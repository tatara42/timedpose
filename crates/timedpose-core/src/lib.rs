//! Core domain logic for Timed Pose:
//! - Drift-free monotonic countdown timer
//! - Fisher-Yates deck shuffle with history and bookmarking
//! - Session scheduling (fixed intervals, progressive art class warmups, rest breaks)

pub mod deck;
pub mod preset;
pub mod session;
pub mod timer;

pub use deck::{Deck, DeckFilter, Orientation, PoseItem};
pub use preset::{PresetMode, SessionPreset, StageConfig};
pub use session::{SessionController, SessionEvent, SessionStatus, SessionSummary};
pub use timer::Timer;
