use std::time::{Duration, Instant};

/// A monotonic, drift-free countdown and stopwatch timer.
/// Calculates remaining time via duration subtraction against `Instant::now()`,
/// making it completely immune to UI thread lag or dispatcher sleep drift.
#[derive(Debug, Clone)]
pub struct Timer {
    target_duration: Duration,
    started_at: Option<Instant>,
    accumulated_elapsed: Duration,
    is_paused: bool,
    warning_emitted: bool,
    countdown_emitted_mask: u32,
}

impl Timer {
    /// Creates a new timer with the specified target duration.
    pub fn new(target_duration: Duration) -> Self {
        Self {
            target_duration,
            started_at: None,
            accumulated_elapsed: Duration::ZERO,
            is_paused: false,
            warning_emitted: false,
            countdown_emitted_mask: 0,
        }
    }

    /// Starts or resumes the timer.
    pub fn start(&mut self) {
        if self.started_at.is_none() || self.is_paused {
            self.started_at = Some(Instant::now());
            self.is_paused = false;
        }
    }

    /// Pauses the countdown, accumulating the elapsed time so far.
    pub fn pause(&mut self) {
        if let Some(start) = self.started_at.take() {
            if !self.is_paused {
                self.accumulated_elapsed += start.elapsed();
                self.is_paused = true;
            }
        }
    }

    /// Resumes the countdown if paused.
    pub fn resume(&mut self) {
        if self.is_paused {
            self.started_at = Some(Instant::now());
            self.is_paused = false;
        }
    }

    /// Toggles between paused and running states.
    pub fn toggle_pause(&mut self) {
        if self.is_paused {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// Resets the timer to zero elapsed time with an optional new target duration.
    pub fn reset(&mut self, new_duration: Option<Duration>) {
        if let Some(duration) = new_duration {
            self.target_duration = duration;
        }
        self.started_at = None;
        self.accumulated_elapsed = Duration::ZERO;
        self.is_paused = false;
        self.warning_emitted = false;
        self.countdown_emitted_mask = 0;
    }

    /// Returns the exact total elapsed duration.
    pub fn elapsed(&self) -> Duration {
        let current_span = match (self.started_at, self.is_paused) {
            (Some(start), false) => start.elapsed(),
            _ => Duration::ZERO,
        };
        self.accumulated_elapsed + current_span
    }

    /// Returns the remaining time until target duration is reached.
    pub fn remaining(&self) -> Duration {
        self.target_duration.saturating_sub(self.elapsed())
    }

    /// Normalized progress from 0.0 (just started) to 1.0 (completed).
    pub fn progress(&self) -> f32 {
        if self.target_duration.is_zero() {
            return 1.0;
        }
        let elapsed_secs = self.elapsed().as_secs_f32();
        let target_secs = self.target_duration.as_secs_f32();
        (elapsed_secs / target_secs).clamp(0.0, 1.0)
    }

    /// Returns true if the timer target has been reached.
    pub fn is_finished(&self) -> bool {
        self.elapsed() >= self.target_duration
    }

    /// Returns true if the timer is actively ticking (not paused and not finished).
    pub fn is_active(&self) -> bool {
        self.started_at.is_some() && !self.is_paused && !self.is_finished()
    }

    /// Returns whether the timer is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Target duration for this timer.
    pub fn target_duration(&self) -> Duration {
        self.target_duration
    }

    /// Checks if a warning threshold (e.g. 5 seconds remaining) has been reached,
    /// returning true only once per interval to trigger an audio chime.
    pub fn check_warning_cue(&mut self, threshold: Duration) -> bool {
        if !self.warning_emitted && self.is_active() && self.remaining() <= threshold {
            self.warning_emitted = true;
            true
        } else {
            false
        }
    }

    /// Checks if a countdown threshold (e.g. 3, 2, 1 seconds remaining) has been crossed,
    /// returning `Some(seconds_remaining)` exactly once per integer second.
    pub fn check_countdown_cue(&mut self, max_countdown_secs: u32) -> Option<u32> {
        if !self.is_active() {
            return None;
        }
        let rem = self.remaining();
        if rem.is_zero() {
            return None;
        }
        let rem_millis = rem.as_millis();
        let sec = rem_millis.div_ceil(1000) as u32;
        if sec >= 1 && sec <= max_countdown_secs {
            let bit = 1 << sec;
            if (self.countdown_emitted_mask & bit) == 0 {
                self.countdown_emitted_mask |= bit;
                return Some(sec);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_timer_countdown_and_progress() {
        let mut timer = Timer::new(Duration::from_millis(50));
        assert_eq!(timer.elapsed(), Duration::ZERO);
        assert_eq!(timer.remaining(), Duration::from_millis(50));
        assert_eq!(timer.progress(), 0.0);

        timer.start();
        sleep(Duration::from_millis(20));
        assert!(timer.elapsed() >= Duration::from_millis(15));
        assert!(timer.progress() > 0.2);
        assert!(!timer.is_finished());

        sleep(Duration::from_millis(40));
        assert!(timer.is_finished());
        assert_eq!(timer.progress(), 1.0);
        assert_eq!(timer.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_timer_pause_and_resume() {
        let mut timer = Timer::new(Duration::from_millis(100));
        timer.start();
        sleep(Duration::from_millis(20));
        timer.pause();
        assert!(timer.is_paused());

        let paused_elapsed = timer.elapsed();
        sleep(Duration::from_millis(30));
        // While paused, elapsed time should remain strictly unchanged
        assert_eq!(timer.elapsed(), paused_elapsed);

        timer.resume();
        assert!(!timer.is_paused());
        sleep(Duration::from_millis(20));
        assert!(timer.elapsed() > paused_elapsed);
    }

    #[test]
    fn test_warning_cue_fires_once() {
        let mut timer = Timer::new(Duration::from_millis(50));
        timer.start();
        sleep(Duration::from_millis(35));

        // Remaining should be <= 20ms
        assert!(timer.check_warning_cue(Duration::from_millis(25)));
        // Second check must return false (fires once per countdown)
        assert!(!timer.check_warning_cue(Duration::from_millis(25)));
    }

    #[test]
    fn test_countdown_cue_fires_each_second() {
        let mut timer = Timer::new(Duration::from_millis(150));
        timer.start();
        // At start, remaining is ~150ms (< 1s, so sec is 1)
        let cue = timer.check_countdown_cue(3);
        assert_eq!(cue, Some(1));
        // Calling again during the same second must not re-trigger
        assert_eq!(timer.check_countdown_cue(3), None);
    }
}
