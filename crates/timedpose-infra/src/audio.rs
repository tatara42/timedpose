use rodio::source::{SineWave, Source};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::time::Duration;

/// Cross-platform audio synthesizer that generates transition chimes,
/// countdown warning beeps, and rest-break bells directly in code.
pub struct AudioService {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    volume: f32,
    enabled: bool,
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

    /// Plays a soft, short high-frequency tick for countdown warnings (e.g. 5 seconds remaining).
    pub fn play_warning_tick(&self) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }

        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * 0.4);
                let tone = SineWave::new(880.0)
                    .take_duration(Duration::from_millis(90))
                    .amplify(0.8);
                sink.append(tone);
                sink.detach();
            }
        }
    }

    /// Plays a pleasant two-tone chime when the pose advances.
    pub fn play_transition_chime(&self) {
        if !self.enabled || self.volume <= 0.0 {
            return;
        }

        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * 0.6);
                // C5 (523 Hz) followed by G5 (784 Hz)
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

        if let Some(handle) = &self.handle {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.set_volume(self.volume * 0.7);
                let bell = SineWave::new(329.63) // E4
                    .take_duration(Duration::from_millis(600))
                    .amplify(1.0);
                sink.append(bell);
                sink.detach();
            }
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
