//! PCM은 rodio(AAudio), MIDI는 rustysynth 소프트신스로 재생.
//! rodio 객체는 전용 오디오 스레드가 소유하고, 에뮬레이터 스레드는
//! 채널(PCM)과 Mutex<Synthesizer>(MIDI)로만 접근한다.

use std::{
    fs::File,
    num::NonZero,
    path::Path,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use rodio::{DeviceSinkBuilder, Player, Source, buffer::SamplesBuffer, conversions::SampleTypeConverter};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

const SYNTH_SAMPLE_RATE: u32 = 44100;
/// 한 번에 렌더링하는 프레임 수 (~12ms @ 44.1kHz)
const SYNTH_CHUNK_FRAMES: usize = 512;

/// 오디오 스레드로 보내는 명령. PCM 재생 외에 호스트 설정(마스터 볼륨)도 이 채널로 전달한다.
enum AudioCommand {
    Pcm {
        channel: u8,
        sampling_rate: u32,
        wave_data: Vec<i16>,
    },
    /// 호스트 볼륨 (0.0~1.0). PCM(효과음)과 MIDI(배경음악)를 분리 조절 —
    /// 사운드폰트 음량과 게임 내장 샘플 음량이 게임마다 달라 밸런스 보정이 필요 (웹버전과 동일).
    SetVolume {
        pcm: f32,
        midi: f32,
    },
}

pub struct AudioEngine {
    tx: Sender<AudioCommand>,
    synth: Option<Arc<Mutex<Synthesizer>>>,
}

impl AudioEngine {
    pub fn new(soundfont_path: Option<&Path>) -> Self {
        tracing::info!("AudioEngine::new(soundfont: {soundfont_path:?})");

        let synth = soundfont_path.and_then(|path| match Self::create_synthesizer(path) {
            Ok(synth) => {
                tracing::info!("soundfont loaded");
                Some(Arc::new(Mutex::new(synth)))
            }
            Err(e) => {
                tracing::warn!("Failed to load soundfont {path:?}: {e} - MIDI will be silent");
                None
            }
        });

        let (tx, rx) = channel();
        {
            let synth = synth.clone();
            std::thread::Builder::new()
                .name("wie-audio".into())
                .spawn(move || Self::audio_thread(rx, synth))
                .unwrap();
        }

        Self { tx, synth }
    }

    fn create_synthesizer(path: &Path) -> anyhow::Result<Synthesizer> {
        let mut file = File::open(path)?;
        let sound_font = Arc::new(SoundFont::new(&mut file)?);
        let settings = SynthesizerSettings::new(SYNTH_SAMPLE_RATE as i32);

        Ok(Synthesizer::new(&sound_font, &settings)?)
    }

    fn audio_thread(rx: Receiver<AudioCommand>, synth: Option<Arc<Mutex<Synthesizer>>>) {
        let output = match DeviceSinkBuilder::open_default_sink() {
            Ok(x) => x,
            Err(e) => {
                tracing::warn!("Failed to open audio output: {e} - audio disabled");
                while rx.recv().is_ok() {}
                return;
            }
        };
        tracing::info!("audio output opened");

        let pcm_player = Player::connect_new(output.mixer());

        // 신스는 무한 소스로 믹서에 상시 연결
        let midi_player = synth.map(|synth| {
            let player = Player::connect_new(output.mixer());
            player.append(SynthSource::new(synth));
            player
        });

        while let Ok(command) = rx.recv() {
            match command {
                AudioCommand::Pcm {
                    channel,
                    sampling_rate,
                    wave_data,
                } => {
                    let (Some(channel_count), Some(sample_rate)) = (NonZero::new(channel.into()), NonZero::new(sampling_rate)) else {
                        continue;
                    };

                    let buffer = SamplesBuffer::new(
                        channel_count,
                        sample_rate,
                        SampleTypeConverter::new(wave_data.into_iter()).collect::<Vec<_>>(),
                    );

                    // TODO 다중 PCM 동시 재생 (wie_cli와 동일한 한계)
                    pcm_player.append(buffer);
                }
                AudioCommand::SetVolume { pcm, midi } => {
                    pcm_player.set_volume(pcm.clamp(0.0, 1.0));
                    if let Some(midi_player) = &midi_player {
                        midi_player.set_volume(midi.clamp(0.0, 1.0));
                    }
                }
            }
        }
    }

    /// 볼륨 설정 (0.0~1.0, 0이면 음소거) — PCM(효과음)/MIDI(배경음악) 분리
    pub fn set_volume(&self, pcm: f32, midi: f32) {
        let _ = self.tx.send(AudioCommand::SetVolume { pcm, midi });
    }

    pub fn sink(&self) -> AudioSink {
        AudioSink {
            pcm_tx: self.tx.clone(),
            synth: self.synth.clone(),
        }
    }
}

/// rustysynth 출력을 rodio 소스로 노출 (스테레오 인터리브, 무한)
struct SynthSource {
    synth: Arc<Mutex<Synthesizer>>,
    left: Vec<f32>,
    right: Vec<f32>,
    interleaved: Vec<f32>,
    pos: usize,
}

impl SynthSource {
    fn new(synth: Arc<Mutex<Synthesizer>>) -> Self {
        Self {
            synth,
            left: vec![0.0; SYNTH_CHUNK_FRAMES],
            right: vec![0.0; SYNTH_CHUNK_FRAMES],
            interleaved: Vec::new(),
            pos: 0,
        }
    }
}

impl Iterator for SynthSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.interleaved.len() {
            self.synth.lock().unwrap().render(&mut self.left, &mut self.right);

            self.interleaved.clear();
            for i in 0..SYNTH_CHUNK_FRAMES {
                self.interleaved.push(self.left[i]);
                self.interleaved.push(self.right[i]);
            }
            self.pos = 0;
        }

        let sample = self.interleaved[self.pos];
        self.pos += 1;
        Some(sample)
    }
}

impl Source for SynthSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SYNTH_SAMPLE_RATE).unwrap()
    }

    fn total_duration(&self) -> Option<core::time::Duration> {
        None
    }
}

pub struct AudioSink {
    pcm_tx: Sender<AudioCommand>,
    synth: Option<Arc<Mutex<Synthesizer>>>,
}

impl AudioSink {
    fn midi(&self, f: impl FnOnce(&mut Synthesizer)) {
        if let Some(synth) = &self.synth {
            f(&mut synth.lock().unwrap());
        }
    }
}

impl wie_backend::AudioSink for AudioSink {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        let _ = self.pcm_tx.send(AudioCommand::Pcm {
            channel,
            sampling_rate,
            wave_data: wave_data.to_vec(),
        });
    }

    fn midi_note_on(&self, channel_id: u8, note: u8, velocity: u8) {
        self.midi(|s| s.note_on(channel_id as i32, note as i32, velocity as i32));
    }

    fn midi_note_off(&self, channel_id: u8, note: u8, _velocity: u8) {
        self.midi(|s| s.note_off(channel_id as i32, note as i32));
    }

    fn midi_program_change(&self, channel_id: u8, program: u8) {
        self.midi(|s| s.process_midi_message(channel_id as i32, 0xC0, program as i32, 0));
    }

    fn midi_control_change(&self, channel_id: u8, control: u8, value: u8) {
        self.midi(|s| s.process_midi_message(channel_id as i32, 0xB0, control as i32, value as i32));
    }

    fn midi_pitch_bend(&self, channel_id: u8, value: u16) {
        self.midi(|s| s.process_midi_message(channel_id as i32, 0xE0, (value & 0x7f) as i32, ((value >> 7) & 0x7f) as i32));
    }

    fn midi_sysex(&self, data: &[u8]) {
        tracing::debug!("midi_sysex({} bytes) - ignored", data.len());
    }
}
