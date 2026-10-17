use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for a specific segment in a progressive session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageConfig {
    /// A set of drawing drills with a given pose count and duration per pose.
    PoseDrill {
        pose_count: usize,
        duration: Duration,
        label: String,
    },
    /// A rest period for wrist stretching and eye fatigue prevention.
    RestBreak {
        duration: Duration,
        prompt: String,
    },
}

/// The mode defining how poses are timed and progressed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetMode {
    /// Constant time per pose with endless cycling.
    Fixed(Duration),
    /// Structured multi-stage session with varying durations and rest breaks.
    Progressive(Vec<StageConfig>),
    /// Untimed session with manual user advancement.
    Endless,
}

/// A session template configuring drawing durations, stages, and breaks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mode: PresetMode,
}

impl SessionPreset {
    /// Creates a fixed-duration preset.
    pub fn fixed(duration: Duration, name: &str, description: &str) -> Self {
        Self {
            id: format!("fixed_{}s", duration.as_secs()),
            name: name.to_string(),
            description: description.to_string(),
            mode: PresetMode::Fixed(duration),
        }
    }

    /// 30-second rapid gesture preset.
    pub fn fixed_30s() -> Self {
        Self::fixed(
            Duration::from_secs(30),
            "Rapid Gesture (30s)",
            "Capture pure line of action and kinetic rhythm.",
        )
    }

    /// 60-second standard gesture preset.
    pub fn fixed_60s() -> Self {
        Self::fixed(
            Duration::from_secs(60),
            "Standard Gesture (60s)",
            "Balance primary rhythm with major masses and angles.",
        )
    }

    /// 120-second form study preset.
    pub fn fixed_120s() -> Self {
        Self::fixed(
            Duration::from_secs(120),
            "Form & Anatomy (2m)",
            "Construct box/cylinder volumes and anatomical landmarks.",
        )
    }

    /// 300-second value study preset.
    pub fn fixed_300s() -> Self {
        Self::fixed(
            Duration::from_secs(300),
            "Detailed Study (5m)",
            "In-depth contours, light patterns, and chiaroscuro values.",
        )
    }

    /// The classic academic life drawing class warmup progression.
    pub fn academic_warmup() -> Self {
        Self {
            id: "academic_warmup".to_string(),
            name: "Academic Class Warmup".to_string(),
            description: "Traditional atelier progression: 10x 30s, 5x 1m, 3m rest break, 2x 5m, 1x 10m."
                .to_string(),
            mode: PresetMode::Progressive(vec![
                StageConfig::PoseDrill {
                    pose_count: 10,
                    duration: Duration::from_secs(30),
                    label: "Warmup: Line of Action".to_string(),
                },
                StageConfig::PoseDrill {
                    pose_count: 5,
                    duration: Duration::from_secs(60),
                    label: "Structure & Masses".to_string(),
                },
                StageConfig::RestBreak {
                    duration: Duration::from_secs(180),
                    prompt: "Model Rest: Put down your stylus, stretch your wrists, and rest your eyes."
                        .to_string(),
                },
                StageConfig::PoseDrill {
                    pose_count: 2,
                    duration: Duration::from_secs(300),
                    label: "Anatomy & Landmarks".to_string(),
                },
                StageConfig::PoseDrill {
                    pose_count: 1,
                    duration: Duration::from_secs(600),
                    label: "Deep Tonal Study".to_string(),
                },
            ]),
        }
    }

    /// Quick 10-minute daily warmup routine.
    pub fn quick_warmup() -> Self {
        Self {
            id: "quick_warmup".to_string(),
            name: "Quick 10m Warmup".to_string(),
            description: "5x 30s quick lines followed by 5x 60s structure poses.".to_string(),
            mode: PresetMode::Progressive(vec![
                StageConfig::PoseDrill {
                    pose_count: 5,
                    duration: Duration::from_secs(30),
                    label: "Quick Gestures".to_string(),
                },
                StageConfig::PoseDrill {
                    pose_count: 5,
                    duration: Duration::from_secs(60),
                    label: "Major Forms".to_string(),
                },
            ]),
        }
    }

    /// Untimed casual sketching mode with manual advance.
    pub fn endless() -> Self {
        Self {
            id: "endless".to_string(),
            name: "Endless / Zen Sketching".to_string(),
            description: "No time limit. Advance to the next pose manually when ready.".to_string(),
            mode: PresetMode::Endless,
        }
    }

    /// Returns the default list of standard factory presets.
    pub fn default_presets() -> Vec<Self> {
        vec![
            Self::fixed_30s(),
            Self::fixed_60s(),
            Self::fixed_120s(),
            Self::fixed_300s(),
            Self::academic_warmup(),
            Self::quick_warmup(),
            Self::endless(),
        ]
    }
}
