mod audio;
mod database;
mod filesystem;
mod screen;

use std::{
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use wie_backend::{Filesystem, Instant, Platform, Screen};

use screen::CaptureScreen;
pub use screen::CapturedFrame;

/// 게임이 요청한 진동 (duration_ms, intensity 0~100).
#[derive(Clone, Copy)]
pub struct VibrationRequest {
    pub duration_ms: u64,
    pub intensity: u8,
}

/// Host platform for mobile targets (and headless testing). All persistent
/// state lives under `base_path` (filesDir on Android, Documents on iOS).
pub struct MobilePlatform {
    screen: CaptureScreen,
    filesystem: filesystem::FsFilesystem,
    database_repository: database::FsDatabaseRepository,
    audio_engine: audio::AudioEngine,
    /// 최근 진동 요청. 호스트가 폴링으로 소비한다 (콜백 없는 설계 유지).
    vibration: Mutex<Option<VibrationRequest>>,
    /// 게임이 종료를 요청했는지. 호스트가 폴링으로 감지해 세션을 정리한다.
    exit_requested: AtomicBool,
}

impl MobilePlatform {
    pub fn new(base_path: PathBuf, width: u32, height: u32, soundfont_path: Option<PathBuf>) -> Self {
        Self {
            screen: CaptureScreen::new(width, height),
            filesystem: filesystem::FsFilesystem::new(base_path.join("fs")),
            database_repository: database::FsDatabaseRepository::new(base_path.join("db")),
            audio_engine: audio::AudioEngine::new(soundfont_path.as_deref()),
            vibration: Mutex::new(None),
            exit_requested: AtomicBool::new(false),
        }
    }

    pub fn screen_capture(&self) -> &CaptureScreen {
        &self.screen
    }

    /// 보류 중인 진동 요청을 가져오고 지운다.
    pub fn take_vibration(&self) -> Option<VibrationRequest> {
        self.vibration.lock().unwrap().take()
    }

    /// 게임의 종료 요청이 있었는지 확인하고 플래그를 지운다.
    pub fn take_exit_requested(&self) -> bool {
        self.exit_requested.swap(false, Ordering::SeqCst)
    }

    /// 볼륨 설정 (0.0~1.0, 0이면 음소거) — PCM(효과음)/MIDI(배경음악) 분리, 호스트 사운드 설정용
    pub fn set_volume(&self, pcm: f32, midi: f32) {
        self.audio_engine.set_volume(pcm, midi);
    }
}

/// MobilePlatform을 Arc로 공유하기 위한 래퍼 — 에뮬레이터가 Box<dyn Platform>을
/// 소유하는 동안에도 호스트(UI/테스트 하니스)가 화면 캡처에 접근할 수 있게 함
pub struct SharedPlatform(pub std::sync::Arc<MobilePlatform>);

impl Platform for SharedPlatform {
    fn screen(&self) -> &dyn Screen {
        self.0.screen()
    }
    fn now(&self) -> Instant {
        self.0.now()
    }
    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        self.0.database_repository()
    }
    fn filesystem(&self) -> &dyn Filesystem {
        self.0.filesystem()
    }
    fn audio_sink(&self) -> Box<dyn wie_backend::AudioSink> {
        self.0.audio_sink()
    }
    fn write_stdout(&self, buf: &[u8]) {
        self.0.write_stdout(buf)
    }
    fn write_stderr(&self, buf: &[u8]) {
        self.0.write_stderr(buf)
    }
    fn exit(&self) {
        self.0.exit()
    }
    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        self.0.vibrate(duration_ms, intensity)
    }
}

impl Platform for MobilePlatform {
    fn screen(&self) -> &dyn Screen {
        &self.screen
    }

    fn now(&self) -> Instant {
        let since_the_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();

        Instant::from_epoch_millis(since_the_epoch.as_millis() as _)
    }

    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        &self.database_repository
    }

    fn filesystem(&self) -> &dyn Filesystem {
        &self.filesystem
    }

    fn audio_sink(&self) -> Box<dyn wie_backend::AudioSink> {
        Box::new(self.audio_engine.sink())
    }

    fn write_stdout(&self, buf: &[u8]) {
        tracing::info!("stdout: {}", String::from_utf8_lossy(buf));
    }

    fn write_stderr(&self, buf: &[u8]) {
        tracing::warn!("stderr: {}", String::from_utf8_lossy(buf));
    }

    fn exit(&self) {
        tracing::info!("app requested exit");
        self.exit_requested.store(true, Ordering::SeqCst);
    }

    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        tracing::info!("vibrate({duration_ms}ms, {intensity}%)");
        *self.vibration.lock().unwrap() = Some(VibrationRequest { duration_ms, intensity });
    }
}
