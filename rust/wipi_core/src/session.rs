//! 에뮬레이터 세션 관리 — Android(JNI)/iOS(C ABI) 브리지가 공유.
//!
//! Threading model: start()가 tick 루프를 도는 전용 에뮬레이터 스레드를 만든다.
//! UI 스레드는 채널로 키 이벤트를 보내고 take_frame()으로 최신 프레임을 폴링한다.
//! 콜백 없음 — 브리지 계층에 런타임 의존성이 생기지 않는다.

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread::JoinHandle,
    time::Instant,
};

use wie_backend::{Event, KeyCode};

use crate::platform::{CapturedFrame, MobilePlatform, SharedPlatform, VibrationRequest};

enum KeyEvent {
    Down(KeyCode),
    Up(KeyCode),
}

/// 에러 종류 — 호스트가 사용자 안내 문구를 고르는 기준. message는 영어 진단 원문(로그/제보용).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    /// 게임 로드 실패 (지원하지 않는 형식, 손상된 파일 등)
    LoadFailed,
    /// 실행 중 오류 (미구현 API 등 호환성 문제)
    Runtime,
}

#[derive(Clone, Debug)]
pub struct SessionError {
    pub kind: ErrorKind,
    pub message: String,
}

struct Session {
    key_tx: Sender<KeyEvent>,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    platform: Arc<MobilePlatform>,
    error: Arc<Mutex<Option<SessionError>>>,
    thread: Option<JoinHandle<()>>,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

fn parse_key(key: &str) -> Option<KeyCode> {
    Some(match key {
        "UP" => KeyCode::UP,
        "DOWN" => KeyCode::DOWN,
        "LEFT" => KeyCode::LEFT,
        "RIGHT" => KeyCode::RIGHT,
        "OK" => KeyCode::OK,
        "0" => KeyCode::NUM0,
        "1" => KeyCode::NUM1,
        "2" => KeyCode::NUM2,
        "3" => KeyCode::NUM3,
        "4" => KeyCode::NUM4,
        "5" => KeyCode::NUM5,
        "6" => KeyCode::NUM6,
        "7" => KeyCode::NUM7,
        "8" => KeyCode::NUM8,
        "9" => KeyCode::NUM9,
        "#" => KeyCode::HASH,
        "*" => KeyCode::STAR,
        "CLR" => KeyCode::CLEAR,
        "SOFT_L" => KeyCode::LEFT_SOFT_KEY,
        "SOFT_R" => KeyCode::RIGHT_SOFT_KEY,
        "CALL" => KeyCode::CALL,
        "HANGUP" => KeyCode::HANGUP,
        _ => return None,
    })
}

fn emulator_thread(
    platform: Arc<MobilePlatform>,
    filename: String,
    game_data: Vec<u8>,
    key_rx: Receiver<KeyEvent>,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    error: Arc<Mutex<Option<SessionError>>>,
) {
    let mut emulator = match crate::create_emulator(Box::new(SharedPlatform(platform.clone())), &filename, &game_data) {
        Ok(x) => x,
        Err(e) => {
            *error.lock().unwrap() = Some(SessionError {
                kind: ErrorKind::LoadFailed,
                message: e.to_string(),
            });
            return;
        }
    };

    let mut pressed: HashMap<KeyCode, Instant> = HashMap::new();

    while !stop.load(Ordering::SeqCst) {
        // 일시정지: tick을 멈춰 게임 세계를 얼린다 (RetroArch 등 에뮬 표준의 auto-pause).
        // 키 이벤트는 채널에 남아 재개 후 처리된다.
        if paused.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        while let Ok(event) = key_rx.try_recv() {
            match event {
                KeyEvent::Down(key) => {
                    if let std::collections::hash_map::Entry::Vacant(e) = pressed.entry(key) {
                        emulator.handle_event(Event::Keydown(key));
                        e.insert(Instant::now());
                    }
                }
                KeyEvent::Up(key) => {
                    if pressed.remove(&key).is_some() {
                        emulator.handle_event(Event::Keyup(key));
                    }
                }
            }
        }

        let now = Instant::now();
        for (key, time) in pressed.iter_mut() {
            if now.duration_since(*time).as_millis() > 100 {
                emulator.handle_event(Event::Keyrepeat(*key));
                *time = now;
            }
        }

        if platform.screen_capture().take_redraw_request() {
            emulator.handle_event(Event::Redraw);
        }

        if let Err(e) = emulator.tick() {
            *error.lock().unwrap() = Some(SessionError {
                kind: ErrorKind::Runtime,
                message: e.to_string(),
            });
            break;
        }
    }
}

/// 게임을 로드하고 에뮬레이터 스레드를 시작한다.
/// 단일 세션 앱이므로 이전 세션이 남아 있으면(호스트 정리 누락 등) 오류 대신 정리하고 시작한다.
pub fn start(filename: String, game_data: Vec<u8>, data_dir: PathBuf, soundfont_path: Option<PathBuf>) -> anyhow::Result<()> {
    let mut session = SESSION.lock().unwrap();
    if let Some(mut stale) = session.take() {
        tracing::warn!("session already running - stopping stale session before start");
        stale.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = stale.thread.take() {
            let _ = thread.join();
        }
    }

    let platform = Arc::new(MobilePlatform::new(data_dir, crate::SCREEN_WIDTH, crate::SCREEN_HEIGHT, soundfont_path));
    let (key_tx, key_rx) = channel();
    let stop = Arc::new(AtomicBool::new(false));
    let paused = Arc::new(AtomicBool::new(false));
    let error = Arc::new(Mutex::new(None));

    let thread = {
        let (platform, stop, paused, error) = (platform.clone(), stop.clone(), paused.clone(), error.clone());
        std::thread::Builder::new()
            .name("wie-emulator".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || emulator_thread(platform, filename, game_data, key_rx, stop, paused, error))?
    };

    *session = Some(Session {
        key_tx,
        stop,
        paused,
        platform,
        error,
        thread: Some(thread),
    });

    Ok(())
}

/// 에뮬레이션 일시정지/재개. 일시정지 중엔 tick이 멈춰 게임 세계가 얼어붙는다.
pub fn set_paused(paused: bool) {
    let session = SESSION.lock().unwrap();
    if let Some(session) = session.as_ref() {
        session.paused.store(paused, Ordering::SeqCst);
    }
}

/// 볼륨 (0.0~1.0, 0이면 음소거) — PCM(효과음)/MIDI(배경음악) 분리, 호스트 사운드 설정용
pub fn set_volume(pcm: f32, midi: f32) {
    let session = SESSION.lock().unwrap();
    if let Some(session) = session.as_ref() {
        session.platform.set_volume(pcm, midi);
    }
}

/// 마지막 paint 이후 새 프레임이 있으면 반환한다.
pub fn take_frame() -> Option<CapturedFrame> {
    let session = SESSION.lock().unwrap();
    session.as_ref()?.platform.screen_capture().take_frame()
}

/// 게임이 요청한 보류 중인 진동을 가져온다 (호스트가 실제 진동 하드웨어로 전달).
pub fn take_vibration() -> Option<VibrationRequest> {
    let session = SESSION.lock().unwrap();
    session.as_ref()?.platform.take_vibration()
}

/// 게임이 종료를 요청했는지 확인한다 (호스트가 세션을 정리하고 UI를 되돌린다).
pub fn take_exit_requested() -> bool {
    let session = SESSION.lock().unwrap();
    session.as_ref().is_some_and(|s| s.platform.take_exit_requested())
}

pub fn key_down(key: &str) {
    send_key(key, KeyEvent::Down)
}

pub fn key_up(key: &str) {
    send_key(key, KeyEvent::Up)
}

fn send_key(key: &str, make: fn(KeyCode) -> KeyEvent) {
    let Some(keycode) = parse_key(key) else {
        tracing::warn!("unknown key: {key}");
        return;
    };

    let session = SESSION.lock().unwrap();
    if let Some(session) = session.as_ref() {
        let _ = session.key_tx.send(make(keycode));
    }
}

/// 보류 중인 오류(종류+진단 원문)를 가져오고 지운다.
pub fn take_error() -> Option<SessionError> {
    let session = SESSION.lock().unwrap();
    session.as_ref().and_then(|s| s.error.lock().unwrap().take())
}

/// 에뮬레이터 스레드를 정지하고 세션을 정리한다.
pub fn stop() {
    let mut session = SESSION.lock().unwrap();
    if let Some(mut session) = session.take() {
        session.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = session.thread.take() {
            let _ = thread.join();
        }
    }
}
