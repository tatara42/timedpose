use crate::canvas::CanvasFilterSettings;
use crate::hotkeys::UserAction;
use egui::{pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Rounding, Stroke, Ui};
use std::f32::consts::TAU;
use std::time::Duration;
use timedpose_core::{SessionController, SessionStatus};

/// Renders the floating HUD controls, countdown badge, and status banners.
pub struct HudRenderer;

impl HudRenderer {
    /// Renders the HUD overlay on top of the canvas.
    pub fn render(
        ui: &mut Ui,
        session: &mut SessionController,
        filters: &mut CanvasFilterSettings,
        always_on_top: bool,
        zen_mode: bool,
    ) -> Option<UserAction> {
        let mut triggered_action = None;
        let screen_rect = ui.max_rect();

        // 1. Model Rest Break Overlay (if currently in break)
        if let SessionStatus::ModelBreak { prompt, .. } = session.status().clone() {
            Self::render_break_overlay(ui, screen_rect, &prompt, session, &mut triggered_action);
            return triggered_action;
        }

        // 2. Minimalist Timer Badge (Top Right)
        if let Some(timer) = session.timer() {
            let remaining = timer.remaining();
            let progress = timer.progress();
            let is_warning = remaining <= Duration::from_secs(5);
            Self::render_timer_badge(ui, screen_rect, remaining, progress, is_warning, timer.is_paused());
        }

        // In Zen mode, we omit all other controls to give pure distraction-free drawing
        if zen_mode {
            return triggered_action;
        }

        // 3. Stage & Pose Tracker (Top Left)
        Self::render_session_tracker(ui, screen_rect, session);

        // 4. Bottom Control Bar
        Self::render_bottom_toolbar(
            ui,
            screen_rect,
            session,
            filters,
            always_on_top,
            &mut triggered_action,
        );

        triggered_action
    }

    fn render_timer_badge(
        ui: &mut Ui,
        screen_rect: Rect,
        remaining: Duration,
        progress: f32,
        is_warning: bool,
        is_paused: bool,
    ) {
        let center = pos2(screen_rect.right() - 55.0, screen_rect.top() + 55.0);
        let radius = 32.0;
        let painter = ui.painter_at(screen_rect);

        // Background circle
        painter.circle_filled(center, radius, Color32::from_black_alpha(180));

        // Progress Arc
        let arc_color = if is_paused {
            Color32::from_rgb(255, 180, 50)
        } else if is_warning {
            Color32::from_rgb(255, 80, 80)
        } else {
            Color32::from_rgb(80, 200, 255)
        };

        let start_angle = -TAU * 0.25;
        let end_angle = start_angle + TAU * (1.0 - progress);

        // Draw segmented arc
        let segments = 32;
        let mut prev_pt: Option<Pos2> = None;
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = start_angle + (end_angle - start_angle) * t;
            let pt = center + vec2(angle.cos() * (radius - 3.0), angle.sin() * (radius - 3.0));
            if let Some(prev) = prev_pt {
                painter.line_segment([prev, pt], Stroke::new(3.0_f32, arc_color));
            }
            prev_pt = Some(pt);
        }

        // Timer text
        let secs = remaining.as_secs();
        let text = if is_paused {
            "PAUSED".to_string()
        } else if secs >= 60 {
            format!("{}:{:02}", secs / 60, secs % 60)
        } else {
            format!("{secs}s")
        };

        let font_size = if is_paused { 11.0 } else { 16.0 };
        painter.text(
            center,
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(font_size),
            Color32::WHITE,
        );
    }

    fn render_session_tracker(ui: &mut Ui, screen_rect: Rect, session: &SessionController) {
        let top_left = pos2(screen_rect.left() + 20.0, screen_rect.top() + 20.0);
        let painter = ui.painter_at(screen_rect);

        if let SessionStatus::ActivePose {
            stage_label,
            pose_index,
            total_poses_in_session,
            ..
        } = session.status()
        {
            let text = if let Some(total) = total_poses_in_session {
                format!("{stage_label}  •  Pose {pose_index} of {total}")
            } else {
                format!("{stage_label}  •  Pose #{pose_index}")
            };

            let font = FontId::proportional(14.0);
            let text_size = ui.fonts(|f| f.layout_no_wrap(text.clone(), font.clone(), Color32::WHITE).size());
            let badge_rect = Rect::from_min_size(top_left, text_size + vec2(20.0, 10.0));

            painter.rect_filled(badge_rect, Rounding::same(6.0), Color32::from_black_alpha(180));
            painter.text(badge_rect.center(), Align2::CENTER_CENTER, text, font, Color32::from_white_alpha(220));
        }
    }

    fn render_bottom_toolbar(
        ui: &mut Ui,
        screen_rect: Rect,
        session: &mut SessionController,
        filters: &mut CanvasFilterSettings,
        always_on_top: bool,
        triggered: &mut Option<UserAction>,
    ) {
        let bar_width = 720.0_f32.min(screen_rect.width() - 40.0);
        let bar_height = 44.0;
        let bar_rect = Rect::from_center_size(
            pos2(screen_rect.center().x, screen_rect.bottom() - 35.0),
            vec2(bar_width, bar_height),
        );

        let painter = ui.painter_at(screen_rect);
        painter.rect_filled(bar_rect, Rounding::same(8.0), Color32::from_black_alpha(200));
        painter.rect_stroke(bar_rect, Rounding::same(8.0), Stroke::new(1.0_f32, Color32::from_white_alpha(40)));

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(bar_rect), |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(8.0);

                if ui.button("⌂ Home").clicked() {
                    *triggered = Some(UserAction::ExitToHome);
                }

                ui.separator();

                if ui.button("◀ Prev (A)").clicked() {
                    *triggered = Some(UserAction::PrevPose);
                }

                let is_paused = session.timer().map(|t| t.is_paused()).unwrap_or(false);
                let pause_label = if is_paused { "▶ Resume (Space)" } else { "⏸ Pause (Space)" };
                if ui.button(pause_label).clicked() {
                    *triggered = Some(UserAction::TogglePause);
                }

                if ui.button("Next (D) ▶").clicked() {
                    *triggered = Some(UserAction::NextPose);
                }

                let is_bookmarked = session.deck().current().map(|i| i.bookmarked).unwrap_or(false);
                let bookmark_label = if is_bookmarked { "★ Bookmarked (K)" } else { "☆ Bookmark (K)" };
                if ui.button(bookmark_label).clicked() {
                    *triggered = Some(UserAction::ToggleBookmark);
                }

                ui.separator();

                // Perceptual tools
                ui.toggle_value(&mut filters.flip_horizontal, "Flip (F)");
                ui.toggle_value(&mut filters.invert_upside_down, "Upside (U)");
                ui.toggle_value(&mut filters.grayscale, "Gray (G)");
                ui.toggle_value(&mut filters.threshold, "Silhouette (S)");
                ui.toggle_value(&mut filters.squint_blur, "Squint (B)");
                ui.toggle_value(&mut filters.grid_overlay, "Grid (R)");

                ui.separator();

                let pin_text = if always_on_top { "📌 Pinned (T)" } else { "📍 Pin (T)" };
                if ui.button(pin_text).clicked() {
                    *triggered = Some(UserAction::ToggleAlwaysOnTop);
                }

                if ui.button("Zen (Tab)").clicked() {
                    *triggered = Some(UserAction::ToggleZenMode);
                }
            });
        });
    }

    fn render_break_overlay(
        ui: &mut Ui,
        screen_rect: Rect,
        prompt: &str,
        session: &mut SessionController,
        triggered: &mut Option<UserAction>,
    ) {
        let painter = ui.painter_at(screen_rect);
        painter.rect_filled(screen_rect, Rounding::ZERO, Color32::from_rgba_unmultiplied(15, 20, 30, 235));

        let center = screen_rect.center();
        let remaining_secs = session.timer().map(|t| t.remaining().as_secs()).unwrap_or(0);

        painter.text(
            center - vec2(0.0, 70.0),
            Align2::CENTER_CENTER,
            "🧘 MODEL REST BREAK",
            FontId::proportional(26.0),
            Color32::from_rgb(120, 220, 180),
        );

        painter.text(
            center - vec2(0.0, 25.0),
            Align2::CENTER_CENTER,
            prompt,
            FontId::proportional(16.0),
            Color32::from_white_alpha(220),
        );

        let time_str = format!("Break ends in {}:{:02}", remaining_secs / 60, remaining_secs % 60);
        painter.text(
            center + vec2(0.0, 20.0),
            Align2::CENTER_CENTER,
            time_str,
            FontId::proportional(20.0),
            Color32::WHITE,
        );

        let button_rect = Rect::from_center_size(center + vec2(0.0, 80.0), vec2(200.0, 38.0));
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(button_rect), |ui| {
            if ui.button("Skip Break & Resume Sketching ▶").clicked() {
                *triggered = Some(UserAction::NextPose);
            }
        });
    }
}
