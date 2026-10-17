use egui::{Color32, Ui};
use std::path::PathBuf;
use timedpose_core::SessionSummary;

pub enum ReviewAction {
    StartBookmarkedOnly(Vec<PathBuf>),
    ReturnHome,
    RestartPreset,
}

pub struct ReviewRenderer;

impl ReviewRenderer {
    pub fn render(ui: &mut Ui, summary: &SessionSummary) -> Option<ReviewAction> {
        let mut action = None;

        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.heading(egui::RichText::new("🎉 Session Completed!").size(28.0).color(Color32::from_rgb(100, 220, 160)));
            ui.add_space(10.0);

            let mins = summary.total_time_spent.as_secs() / 60;
            let secs = summary.total_time_spent.as_secs() % 60;
            let stats_text = format!(
                "Total Poses: {}   •   Time Spent: {}m {}s   •   Bookmarked: {}",
                summary.total_poses_completed,
                mins,
                secs,
                summary.bookmarked_poses.len()
            );
            ui.label(egui::RichText::new(stats_text).size(16.0).color(Color32::from_white_alpha(200)));

            ui.add_space(20.0);

            // Action Buttons
            ui.horizontal_centered(|ui| {
                if !summary.bookmarked_poses.is_empty() {
                    let btn_label = format!("★ Practice Bookmarked ({})", summary.bookmarked_poses.len());
                    if ui.button(egui::RichText::new(btn_label).color(Color32::from_rgb(255, 215, 0))).clicked() {
                        action = Some(ReviewAction::StartBookmarkedOnly(summary.bookmarked_poses.clone()));
                    }
                }

                if ui.button("↺ Repeat Session").clicked() {
                    action = Some(ReviewAction::RestartPreset);
                }

                if ui.button("⌂ Back to Home").clicked() {
                    action = Some(ReviewAction::ReturnHome);
                }
            });

            ui.add_space(25.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(egui::RichText::new("Session Poses").size(18.0).strong());
            ui.add_space(10.0);

            // Scrollable list of visited poses
            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for (idx, path) in summary.history.iter().enumerate() {
                    let is_bookmarked = summary.bookmarked_poses.contains(path);
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("pose");

                    ui.horizontal(|ui| {
                        ui.label(format!("#{}:", idx + 1));
                        if is_bookmarked {
                            ui.colored_label(Color32::from_rgb(255, 215, 0), "★");
                        }
                        ui.label(file_name);
                        ui.label(egui::RichText::new(path.display().to_string()).size(11.0).color(Color32::GRAY));
                    });
                }
            });
        });

        action
    }
}
