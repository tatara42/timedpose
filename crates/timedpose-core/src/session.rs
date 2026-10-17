use crate::deck::{Deck, PoseItem};
use crate::preset::{PresetMode, SessionPreset, StageConfig};
use crate::timer::Timer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Summary generated upon completing a drawing session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub total_poses_completed: usize,
    pub total_time_spent: Duration,
    pub bookmarked_poses: Vec<PathBuf>,
    pub history: Vec<PathBuf>,
}

/// Status of the active session.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Idle,
    ActivePose {
        pose: PoseItem,
        stage_label: String,
        pose_index: usize,
        total_poses_in_session: Option<usize>,
    },
    ModelBreak {
        prompt: String,
        duration: Duration,
    },
    Paused,
    Completed(SessionSummary),
}

/// Events emitted by `SessionController` to notify the UI and audio services.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    PoseChanged {
        pose: PoseItem,
        pose_index: usize,
        total_poses: Option<usize>,
        stage_label: String,
        duration: Option<Duration>,
    },
    WarningCue {
        remaining: Duration,
    },
    TransitionCue,
    BreakStarted {
        prompt: String,
        duration: Duration,
    },
    BreakEnded,
    SessionCompleted {
        summary: SessionSummary,
    },
}

/// Orchestrates the Deck, Timer, and SessionPreset into a cohesive drawing session.
#[derive(Debug, Clone)]
pub struct SessionController {
    deck: Deck,
    preset: SessionPreset,
    timer: Option<Timer>,
    status: SessionStatus,
    session_start: Option<Instant>,
    accumulated_session_time: Duration,
    // Progressive session tracking
    current_stage_idx: usize,
    poses_in_current_stage: usize,
    total_poses_drawn: usize,
    warning_threshold: Duration,
}

impl SessionController {
    /// Initializes a new session with a deck of poses and a session preset.
    pub fn new(deck: Deck, preset: SessionPreset) -> Self {
        Self {
            deck,
            preset,
            timer: None,
            status: SessionStatus::Idle,
            session_start: None,
            accumulated_session_time: Duration::ZERO,
            current_stage_idx: 0,
            poses_in_current_stage: 0,
            total_poses_drawn: 0,
            warning_threshold: Duration::from_secs(5),
        }
    }

    /// Sets the countdown warning threshold (default: 5 seconds).
    pub fn set_warning_threshold(&mut self, threshold: Duration) {
        self.warning_threshold = threshold;
    }

    /// Starts or restarts the drawing session.
    pub fn start(&mut self) -> Vec<SessionEvent> {
        self.session_start = Some(Instant::now());
        self.accumulated_session_time = Duration::ZERO;
        self.current_stage_idx = 0;
        self.poses_in_current_stage = 0;
        self.total_poses_drawn = 0;

        self.advance_to_next_step()
    }

    /// Toggles pause state.
    pub fn toggle_pause(&mut self) {
        match self.status {
            SessionStatus::ActivePose { .. } | SessionStatus::ModelBreak { .. } => {
                if let Some(timer) = &mut self.timer {
                    timer.pause();
                }
                self.status = SessionStatus::Paused;
            }
            SessionStatus::Paused => {
                if let Some(timer) = &mut self.timer {
                    timer.resume();
                }
                // Restore status based on current pose or break
                if self.is_in_break() {
                    let prompt = self.current_break_prompt();
                    let duration = self.timer.as_ref().map(|t| t.target_duration()).unwrap_or_default();
                    self.status = SessionStatus::ModelBreak { prompt, duration };
                } else if let Some(pose) = self.deck.current().cloned() {
                    let label = self.current_stage_label();
                    self.status = SessionStatus::ActivePose {
                        pose,
                        stage_label: label,
                        pose_index: self.total_poses_drawn,
                        total_poses_in_session: self.total_poses_planned(),
                    };
                }
            }
            _ => {}
        }
    }

    /// Manually skips to the next pose.
    pub fn next_pose(&mut self) -> Vec<SessionEvent> {
        self.advance_to_next_step()
    }

    /// Navigates back to the previous pose.
    pub fn prev_pose(&mut self) -> Vec<SessionEvent> {
        if let Some(pose) = self.deck.prev().cloned() {
            let label = self.current_stage_label();
            let duration = self.current_target_duration();

            if let Some(dur) = duration {
                if let Some(timer) = &mut self.timer {
                    timer.reset(Some(dur));
                    timer.start();
                }
            }

            self.status = SessionStatus::ActivePose {
                pose: pose.clone(),
                stage_label: label.clone(),
                pose_index: self.total_poses_drawn.saturating_sub(1),
                total_poses_in_session: self.total_poses_planned(),
            };

            vec![SessionEvent::PoseChanged {
                pose,
                pose_index: self.total_poses_drawn.saturating_sub(1),
                total_poses: self.total_poses_planned(),
                stage_label: label,
                duration,
            }]
        } else {
            Vec::new()
        }
    }

    /// Toggles the bookmark status for the current active pose.
    pub fn toggle_bookmark(&mut self) -> bool {
        self.deck.toggle_bookmark_current()
    }

    /// Periodic update loop. Checks for timer expiration and emits warning/transition cues.
    pub fn update(&mut self) -> Vec<SessionEvent> {
        let mut events = Vec::new();

        if let Some(timer) = &mut self.timer {
            if timer.is_active() && timer.check_warning_cue(self.warning_threshold) {
                events.push(SessionEvent::WarningCue {
                    remaining: timer.remaining(),
                });
            }

            if timer.is_finished() {
                events.push(SessionEvent::TransitionCue);
                if self.is_in_break() {
                    events.push(SessionEvent::BreakEnded);
                }
                events.extend(self.advance_to_next_step());
            }
        }

        events
    }

    fn advance_to_next_step(&mut self) -> Vec<SessionEvent> {
        match &self.preset.mode.clone() {
            PresetMode::Fixed(duration) => {
                if let Some(pose) = self.deck.next().cloned() {
                    self.total_poses_drawn += 1;
                    let mut timer = Timer::new(*duration);
                    timer.start();
                    self.timer = Some(timer);

                    self.status = SessionStatus::ActivePose {
                        pose: pose.clone(),
                        stage_label: self.preset.name.clone(),
                        pose_index: self.total_poses_drawn,
                        total_poses_in_session: None,
                    };

                    vec![SessionEvent::PoseChanged {
                        pose,
                        pose_index: self.total_poses_drawn,
                        total_poses: None,
                        stage_label: self.preset.name.clone(),
                        duration: Some(*duration),
                    }]
                } else {
                    self.finish_session()
                }
            }
            PresetMode::Endless => {
                if let Some(pose) = self.deck.next().cloned() {
                    self.total_poses_drawn += 1;
                    self.timer = None;

                    self.status = SessionStatus::ActivePose {
                        pose: pose.clone(),
                        stage_label: "Endless Sketch".to_string(),
                        pose_index: self.total_poses_drawn,
                        total_poses_in_session: None,
                    };

                    vec![SessionEvent::PoseChanged {
                        pose,
                        pose_index: self.total_poses_drawn,
                        total_poses: None,
                        stage_label: "Endless Sketch".to_string(),
                        duration: None,
                    }]
                } else {
                    self.finish_session()
                }
            }
            PresetMode::Progressive(stages) => {
                if self.current_stage_idx >= stages.len() {
                    return self.finish_session();
                }

                match &stages[self.current_stage_idx] {
                    StageConfig::PoseDrill {
                        pose_count,
                        duration,
                        label,
                    } => {
                        if self.poses_in_current_stage < *pose_count {
                            if let Some(pose) = self.deck.next().cloned() {
                                self.poses_in_current_stage += 1;
                                self.total_poses_drawn += 1;

                                let mut timer = Timer::new(*duration);
                                timer.start();
                                self.timer = Some(timer);

                                self.status = SessionStatus::ActivePose {
                                    pose: pose.clone(),
                                    stage_label: label.clone(),
                                    pose_index: self.total_poses_drawn,
                                    total_poses_in_session: self.total_poses_planned(),
                                };

                                vec![SessionEvent::PoseChanged {
                                    pose,
                                    pose_index: self.total_poses_drawn,
                                    total_poses: self.total_poses_planned(),
                                    stage_label: label.clone(),
                                    duration: Some(*duration),
                                }]
                            } else {
                                self.finish_session()
                            }
                        } else {
                            // Stage completed, advance to next stage
                            self.current_stage_idx += 1;
                            self.poses_in_current_stage = 0;
                            self.advance_to_next_step()
                        }
                    }
                    StageConfig::RestBreak { duration, prompt } => {
                        let prompt_str = prompt.clone();
                        let dur = *duration;
                        let mut timer = Timer::new(dur);
                        timer.start();
                        self.timer = Some(timer);

                        self.status = SessionStatus::ModelBreak {
                            prompt: prompt_str.clone(),
                            duration: dur,
                        };

                        // Advance past the break stage so next tick moves to following drill
                        self.current_stage_idx += 1;
                        self.poses_in_current_stage = 0;

                        vec![SessionEvent::BreakStarted {
                            prompt: prompt_str,
                            duration: dur,
                        }]
                    }
                }
            }
        }
    }

    fn finish_session(&mut self) -> Vec<SessionEvent> {
        let total_time = self.session_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO);
        let summary = SessionSummary {
            total_poses_completed: self.total_poses_drawn,
            total_time_spent: total_time,
            bookmarked_poses: self.deck.bookmarked_items().into_iter().map(|i| i.path.clone()).collect(),
            history: self.deck.history_items().into_iter().map(|i| i.path.clone()).collect(),
        };

        self.status = SessionStatus::Completed(summary.clone());
        self.timer = None;

        vec![SessionEvent::SessionCompleted { summary }]
    }

    fn is_in_break(&self) -> bool {
        matches!(self.status, SessionStatus::ModelBreak { .. })
    }

    fn current_break_prompt(&self) -> String {
        if let SessionStatus::ModelBreak { prompt, .. } = &self.status {
            prompt.clone()
        } else {
            "Model Rest".to_string()
        }
    }

    fn current_stage_label(&self) -> String {
        match &self.preset.mode {
            PresetMode::Fixed(_) => self.preset.name.clone(),
            PresetMode::Endless => "Endless".to_string(),
            PresetMode::Progressive(stages) => {
                if let Some(stage) = stages.get(self.current_stage_idx) {
                    match stage {
                        StageConfig::PoseDrill { label, .. } => label.clone(),
                        StageConfig::RestBreak { .. } => "Rest Break".to_string(),
                    }
                } else {
                    "Completed".to_string()
                }
            }
        }
    }

    fn current_target_duration(&self) -> Option<Duration> {
        match &self.preset.mode {
            PresetMode::Fixed(d) => Some(*d),
            PresetMode::Endless => None,
            PresetMode::Progressive(stages) => stages.get(self.current_stage_idx).and_then(|s| match s {
                StageConfig::PoseDrill { duration, .. } => Some(*duration),
                StageConfig::RestBreak { duration, .. } => Some(*duration),
            }),
        }
    }

    /// Total number of planned poses for this session, if bounded.
    pub fn total_poses_planned(&self) -> Option<usize> {
        match &self.preset.mode {
            PresetMode::Progressive(stages) => {
                let total = stages
                    .iter()
                    .filter_map(|s| match s {
                        StageConfig::PoseDrill { pose_count, .. } => Some(*pose_count),
                        StageConfig::RestBreak { .. } => None,
                    })
                    .sum();
                Some(total)
            }
            _ => None,
        }
    }

    /// Access the active timer, if any.
    pub fn timer(&self) -> Option<&Timer> {
        self.timer.as_ref()
    }

    /// Current status of the session.
    pub fn status(&self) -> &SessionStatus {
        &self.status
    }

    /// Reference to the internal deck.
    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    /// Mutable reference to the internal deck.
    pub fn deck_mut(&mut self) -> &mut Deck {
        &mut self.deck
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progressive_warmup_session_flow() {
        let paths: Vec<PathBuf> = (0..5).map(|i| PathBuf::from(format!("p_{i}.jpg"))).collect();
        let deck = Deck::from_paths(&paths);

        let preset = SessionPreset {
            id: "test".to_string(),
            name: "Test Session".to_string(),
            description: "Testing progressive stages".to_string(),
            mode: PresetMode::Progressive(vec![
                StageConfig::PoseDrill {
                    pose_count: 2,
                    duration: Duration::from_millis(50),
                    label: "Stage 1".to_string(),
                },
                StageConfig::RestBreak {
                    duration: Duration::from_millis(30),
                    prompt: "Rest now".to_string(),
                },
                StageConfig::PoseDrill {
                    pose_count: 1,
                    duration: Duration::from_millis(50),
                    label: "Stage 2".to_string(),
                },
            ]),
        };

        let mut session = SessionController::new(deck, preset);
        let start_events = session.start();
        assert_eq!(start_events.len(), 1);
        assert!(matches!(start_events[0], SessionEvent::PoseChanged { .. }));

        // Pose 1 is active
        assert_eq!(session.total_poses_drawn, 1);

        // Advance to Pose 2
        let p2_events = session.next_pose();
        assert_eq!(session.total_poses_drawn, 2);
        assert!(p2_events.iter().any(|e| matches!(e, SessionEvent::PoseChanged { .. })));

        // Advance to Break
        let break_events = session.next_pose();
        assert!(break_events.iter().any(|e| matches!(e, SessionEvent::BreakStarted { .. })));
        assert!(matches!(session.status(), SessionStatus::ModelBreak { .. }));

        // Advance past Break to Pose 3 (Stage 2)
        let p3_events = session.next_pose();
        assert_eq!(session.total_poses_drawn, 3);
        assert!(p3_events.iter().any(|e| matches!(e, SessionEvent::PoseChanged { .. })));

        // Advance past Stage 2 -> session finishes
        let finish_events = session.next_pose();
        assert!(finish_events.iter().any(|e| matches!(e, SessionEvent::SessionCompleted { .. })));
        assert!(matches!(session.status(), SessionStatus::Completed(_)));
    }
}
