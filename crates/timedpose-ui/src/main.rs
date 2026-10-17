mod app;
mod canvas;
mod home;
mod hotkeys;
mod hud;
mod review;

use app::TimedPoseApp;
use eframe::NativeOptions;
use timedpose_infra::ConfigManager;

fn main() -> eframe::Result<()> {
    let config = ConfigManager::load();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Timed Pose - Gesture & Figure Drawing Companion")
        .with_inner_size([config.window_width, config.window_height])
        .with_min_inner_size([480.0, 450.0]);

    if config.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    let native_options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Timed Pose",
        native_options,
        Box::new(|cc| Ok(Box::new(TimedPoseApp::new(cc)))),
    )
}
