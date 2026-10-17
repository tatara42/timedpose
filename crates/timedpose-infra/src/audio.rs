use rodio::source::{SineWave, Source};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Types of sound cues supported in sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCueType {
    Countdown,
    Transition,
    Break,
    Finished,
}

/// Cross-platform audio service supporting procedural synthesis and custom WAV/OGG files.
pub struct AudioService {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    volume: f32,
    enabled: bool,
    // In-memory cache of custom decoded audio bytes for 0ms disk latency
    custom_countdown: Option<Arc<Vec<u8>>>,
    custom_transition: Option<Arc<Vec<u8>>>,
    custom_break: Option<Arc<Vec<u8>>>,
    custom_finished: Option<Arc<Vec<u8>>>,
}

impl AudioService {
    /// Initializes the audio service, attempting to connect to the default OS sound device.
    /// Operates gracefully in silent mode if no audio device is available (e.g. headless servers).
    pub fn new(enabled: bool, volume: f32) -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("Info: Audio device unavailable ({e}). Operating in silent mode.");
                (None, None)
            }
        };

        Self {
            _stream: stream,
            handle,
            volume: volume.clamp(0.0, 1.0),
            enabled,
            custom_countdown: None,
            custom_transition: None,
            custom_break: None,
            custom_finished: None,
        }
    }

    /// Sets the master volume (0.0 to 1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Enables or disables audio playback.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Loads custom sound files into memory cache from paths.
    /// Supports .wav and .ogg audio files.
    pub fn load_custom_sounds(
        &mut self,
        countdown_path: Option<&PathBuf>,
        transition_path: Option<&PathBuf>,
        break_path: Option<&PathBuf>,
        finished_path: Option<&PathBuf>,
    ) {
        self.custom_countdown = countdown_path.and_then(|p| Self::read_audio_file(p));
        self.custom_transition = transition_path.and_then(|p| Self::read_audio_file(p));
        self.custom_break = break_path.and_then(|p| Self::read_audio_file(p));
        self.custom_finished = finished_path.and_then(|p| Self::read_audio_file(p));
    }

    /// Helper to read and validate a WAV or OGG audio file into memory.
    fn read_audio_file(path: &Path) -> Option<Arc<Vec<u8>>> {
        if !path.exists() {
            return None;
        }

        match fs::read(path) {
            Ok(bytes) => {
                // Verify the buffer can be decoded by rodio (valid WAV or OGG)
                let cursor = Cursor::new(bytes.clone());
                match rodio::Decoder::new(cursor) {
                    Ok(_) => Some(Arc::new(bytes)),
                    Err(e) => {
                        eprintln!("Warning: Failed to decode custom audio at {}: {e}", path.display());
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read custom audio file at {}: {e}", path.display());
                None
            }
        }
    }

    /// Plays custom sound if loaded, returning true if custom sound was dispatched.
    fn play_custom_sound(&self, sound_bytes: &Option<Arc<Vec<u8>>>, volume_scale: f32) -> bool {
        let Some(bytes) = sound_bytes else {
            return false;
        };

        let Some(handle) = &self.handle else {
            return false;
        };

        let cursor = Cursor::new((**bytes).clone());
        if let Ok(decoder) = rodio::Decoder::new(cursor) {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * volume_scale);
                sink.append(decoder);
                sink.detach();
                return true;
            }
        }

        false
    }

    /// Plays a countdown sound for 3, 2, 1 seconds remaining.
    /// Uses custom audio if configured, otherwise plays pitch-scaled synthesized tick (E5 -> G5 -> A5).
    pub fn play_countdown_tick(&self, seconds_remaining: u32) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }

        // Try playing custom countdown audio
        if self.play_custom_sound(&self.custom_countdown, 0.7) {
            return;
        }

        // Fallback to synthesized ascending ticks: 3 = 659Hz, 2 = 784Hz, 1 = 880Hz
        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                let freq = match seconds_remaining {
                    3 => 659.25, // E5
                    2 => 783.99, // G5
                    1 => 880.00, // A5
                    _ => 880.00,
                };
                let duration = if seconds_remaining == 1 {
                    Duration::from_millis(110)
                } else {
                    Duration::from_millis(90)
                };

                sink.set_volume(self.volume * 0.45);
                let tone = SineWave::new(freq)
                    .take_duration(duration)
                    .amplify(0.85);
                sink.append(tone);
                sink.detach();
            }
        }
    }

    /// Legacy warning tick for 5s warning threshold.
    pub fn play_warning_tick(&self) {
        self.play_countdown_tick(3);
    }

    /// Plays transition chime when pose advances.
    pub fn play_transition_chime(&self) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }

        // Try playing custom transition audio
        if self.play_custom_sound(&self.custom_transition, 0.8) {
            return;
        }

        // Fallback to pleasant two-tone chime (C5 -> G5)
        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * 0.6);
                let note1 = SineWave::new(523.25)
                    .take_duration(Duration::from_millis(120))
                    .amplify(0.7);
                let note2 = SineWave::new(783.99)
                    .take_duration(Duration::from_millis(250))
                    .amplify(0.9);
                sink.append(note1);
                sink.append(note2);
                sink.detach();
            }
        }
    }

    /// Plays a deep gong/bell tone marking the start of a model rest break.
    pub fn play_break_chime(&self) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }

        // Try playing custom break audio
        if self.play_custom_sound(&self.custom_break, 0.9) {
            return;
        }

        // Fallback to synthesized deep bell tone (E4 -> B3)
        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * 0.7);
                let bell1 = SineWave::new(329.63) // E4
                    .take_duration(Duration::from_millis(500))
                    .amplify(0.9);
                let bell2 = SineWave::new(246.94) // B3
                    .take_duration(Duration::from_millis(400))
                    .amplify(0.7);
                sink.append(bell1);
                sink.append(bell2);
                sink.detach();
            }
        }
    }

    /// Plays a triumphant finished chime marking session completion.
    pub fn play_finished_sound(&self) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }

        // Try playing custom finished audio
        if self.play_custom_sound(&self.custom_finished, 0.9) {
            return;
        }

        // Fallback to rising celebratory arpeggio: C5 -> E5 -> G5 -> C6
        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * 0.65);
                let c5 = SineWave::new(523.25)
                    .take_duration(Duration::from_millis(100))
                    .amplify(0.7);
                let e5 = SineWave::new(659.25)
                    .take_duration(Duration::from_millis(100))
                    .amplify(0.8);
                let g5 = SineWave::new(783.99)
                    .take_duration(Duration::from_millis(100))
                    .amplify(0.85);
                let c6 = SineWave::new(1046.50)
                    .take_duration(Duration::from_millis(350))
                    .amplify(0.95);
                sink.append(c5);
                sink.append(e5);
                sink.append(g5);
                sink.append(c6);
                sink.detach();
            }
        }
    }

    /// Previews/tests a sound cue directly (e.g. from the UI sound settings).
    pub fn test_sound(&self, cue_type: SoundCueType) {
        match cue_type {
            SoundCueType::Countdown => self.play_countdown_tick(1),
            SoundCueType::Transition => self.play_transition_chime(),
            SoundCueType::Break => self.play_break_chime(),
            SoundCueType::Finished => self.play_finished_sound(),
        }
    }

    /// Plays a subtle click for metronome / pacing rhythm mode.
    pub fn play_metronome_click(&self) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }

        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * 0.25);
                let click = SineWave::new(1200.0)
                    .take_duration(Duration::from_millis(20))
                    .amplify(0.6);
                sink.append(click);
                sink.detach();
            }
        }
    }
}
