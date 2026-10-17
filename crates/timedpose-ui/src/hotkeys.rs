use egui::Key;

/// High-level user actions triggered by single-handed keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAction {
    TogglePause,
    NextPose,
    PrevPose,
    ToggleFlipHorizontal,
    ToggleUpsideDown,
    ToggleGrayscale,
    ToggleThreshold,
    ToggleSquintBlur,
    ToggleGrid,
    ToggleBookmark,
    ToggleAlwaysOnTop,
    ToggleZenMode,
    ResetPanZoom,
    OpenSettings,
    ExitToHome,
}

pub struct HotkeyManager;

impl HotkeyManager {
    /// Evaluates egui keyboard input and returns the triggered action, if any.
    /// Hotkeys are designed for single-handed left-hand use while right hand holds a drawing stylus.
    pub fn handle_input(ctx: &egui::Context) -> Option<UserAction> {
        ctx.input(|input| {
            // Navigation & Flow
            if input.key_pressed(Key::Space) {
                return Some(UserAction::TogglePause);
            }
            if input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::D) {
                return Some(UserAction::NextPose);
            }
            if input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::A) {
                return Some(UserAction::PrevPose);
            }

            // Perceptual Drawing Aids
            if input.key_pressed(Key::F) {
                return Some(UserAction::ToggleFlipHorizontal);
            }
            if input.key_pressed(Key::U) {
                return Some(UserAction::ToggleUpsideDown);
            }
            if input.key_pressed(Key::G) {
                return Some(UserAction::ToggleGrayscale);
            }
            if input.key_pressed(Key::S) && !input.modifiers.command {
                return Some(UserAction::ToggleThreshold);
            }
            if input.key_pressed(Key::B) {
                return Some(UserAction::ToggleSquintBlur);
            }
            if input.key_pressed(Key::R) && !input.modifiers.command {
                return Some(UserAction::ToggleGrid);
            }

            // Bookmarks & Pinning
            if input.key_pressed(Key::K) {
                return Some(UserAction::ToggleBookmark);
            }
            if input.key_pressed(Key::T) {
                return Some(UserAction::ToggleAlwaysOnTop);
            }
            if input.key_pressed(Key::Tab) {
                return Some(UserAction::ToggleZenMode);
            }
            if input.key_pressed(Key::Num0) || input.key_pressed(Key::Z) {
                return Some(UserAction::ResetPanZoom);
            }

            // Modal / Exit
            if input.key_pressed(Key::Escape) {
                return Some(UserAction::ExitToHome);
            }
            if input.key_pressed(Key::O) && input.modifiers.command {
                return Some(UserAction::OpenSettings);
            }

            None
        })
    }
}
