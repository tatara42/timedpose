use egui::{vec2, Color32, ComboBox, RichText, Ui};
use std::path::PathBuf;
use timedpose_core::{DeckFilter, SessionPreset};
use timedpose_infra::AppConfig;

pub enum HomeAction {
    PickFolder,
    SelectRecentFolder(PathBuf),
    StartSession,
}

pub struct HomeRenderer;

impl HomeRenderer {
    pub fn render(
        ui: &mut Ui,
        config: &mut AppConfig,
        presets: &[SessionPreset],
        selected_preset_idx: &mut usize,
        deck_filter: &mut DeckFilter,
        scanned_count: usize,
    ) -> Option<HomeAction> {
        let mut action = None;

        ui.vertical_centered(|ui| {
            ui.add_space(35.0);
            ui.label(RichText::new("TIMED POSE").size(34.0).strong().color(Color32::WHITE));
            ui.label(RichText::new("Gesture & Figure Drawing Companion").size(14.0).color(Color32::from_white_alpha(160)));
            ui.add_space(25.0);

            let content_width = 500.0_f32.min(ui.available_width() - 40.0);

            ui.allocate_ui_with_layout(
                vec2(content_width, 480.0),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    // 1. Folder Selection Card
                    ui.group(|ui| {
                        ui.set_width(content_width);
                        ui.label(RichText::new("1. Reference Image Library").strong());
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            if ui.button("📂 Choose Folder...").clicked() {
                                action = Some(HomeAction::PickFolder);
                            }

                            if let Some(folder) = &config.last_folder {
                                let folder_name = folder.file_name().and_then(|n| n.to_str()).unwrap_or("Folder");
                                ui.label(RichText::new(format!("{folder_name} ({scanned_count} images found)")).color(Color32::from_rgb(120, 220, 160)));
                            } else {
                                ui.colored_label(Color32::from_rgb(255, 180, 50), "No folder selected yet");
                            }
                        });

                        if !config.recent_folders.is_empty() {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Recent:").size(11.0).color(Color32::GRAY));
                                for recent in config.recent_folders.iter().take(3) {
                                    let name = recent.file_name().and_then(|n| n.to_str()).unwrap_or("dir");
                                    if ui.small_button(name).clicked() {
                                        action = Some(HomeAction::SelectRecentFolder(recent.clone()));
                                    }
                                }
                            });
                        }
                    });

                    ui.add_space(15.0);

                    // 2. Preset Selection Card
                    ui.group(|ui| {
                        ui.set_width(content_width);
                        ui.label(RichText::new("2. Session Format & Timing").strong());
                        ui.add_space(6.0);

                        let current_preset_name = presets
                            .get(*selected_preset_idx)
                            .map(|p| p.name.as_str())
                            .unwrap_or("Select Preset");

                        ComboBox::from_id_salt("preset_combo")
                            .selected_text(current_preset_name)
                            .width(content_width - 30.0)
                            .show_ui(ui, |ui| {
                                for (idx, preset) in presets.iter().enumerate() {
                                    if ui.selectable_value(selected_preset_idx, idx, &preset.name).clicked() {
                                        config.default_preset_id = preset.id.clone();
                                    }
                                }
                            });

                        if let Some(preset) = presets.get(*selected_preset_idx) {
                            ui.add_space(4.0);
                            ui.label(RichText::new(&preset.description).size(12.0).color(Color32::from_white_alpha(180)));
                        }
                    });

                    ui.add_space(15.0);

                    // 3. Orientation & Filter Card
                    ui.group(|ui| {
                        ui.set_width(content_width);
                        ui.label(RichText::new("3. Canvas Aspect Ratio Matching").strong());
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.selectable_value(deck_filter, DeckFilter::All, "All Poses");
                            ui.selectable_value(deck_filter, DeckFilter::PortraitOnly, "Portrait Only (Vertical)");
                            ui.selectable_value(deck_filter, DeckFilter::LandscapeOnly, "Landscape Only (Horizontal)");
                        });
                    });

                    ui.add_space(15.0);

                    // 4. Audio Preferences
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut config.enable_audio, "Sound Cues (5s warning & transition chime)");
                        if config.enable_audio {
                            ui.add(egui::Slider::new(&mut config.volume, 0.0..=1.0).text("Volume"));
                        }
                    });

                    ui.add_space(25.0);

                    // 5. Start Button
                    let can_start = scanned_count > 0;
                    ui.add_enabled_ui(can_start, |ui| {
                        let btn_text = RichText::new("▶  START DRAWING SESSION  ◀")
                            .size(18.0)
                            .strong()
                            .color(if can_start { Color32::WHITE } else { Color32::GRAY });

                        let start_btn = ui.add_sized([content_width, 48.0], egui::Button::new(btn_text));
                        if start_btn.clicked() {
                            action = Some(HomeAction::StartSession);
                        }
                    });

                    if !can_start {
                        ui.add_space(4.0);
                        ui.label(RichText::new("Please choose a folder with images to start.").size(12.0).color(Color32::from_rgb(255, 120, 120)));
                    }
                },
            );
        });

        action
    }
}
