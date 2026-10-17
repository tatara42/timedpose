use crate::canvas::{process_image_filters, render_canvas, CanvasFilterSettings};
use crate::home::{HomeAction, HomeRenderer};
use crate::hotkeys::{HotkeyManager, UserAction};
use crate::hud::HudRenderer;
use crate::review::{ReviewAction, ReviewRenderer};
use eframe::App;
use egui::{CentralPanel, Color32, TextureOptions, Vec2};
use std::path::PathBuf;
use timedpose_core::{
    Deck, DeckFilter, PoseItem, SessionController, SessionEvent, SessionPreset, SessionSummary,
};
use timedpose_infra::{AppConfig, AudioService, ConfigManager, DecodedImage, ImagePreloader, ImageScanner};

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Home,
    ActiveSession,
    Review(SessionSummary),
}

pub struct TimedPoseApp {
    config: AppConfig,
    audio: AudioService,
    preloader: ImagePreloader,
    presets: Vec<SessionPreset>,
    selected_preset_idx: usize,
    deck_filter: DeckFilter,
    scanned_poses: Vec<PoseItem>,
    session: Option<SessionController>,
    active_texture: Option<egui::TextureHandle>,
    active_decoded: Option<DecodedImage>,
    canvas_filters: CanvasFilterSettings,
    pending_texture_update: Option<egui::ColorImage>,
    pan: Vec2,
    zoom: f32,
    screen: AppScreen,
    always_on_top: bool,
    zen_mode: bool,
}

impl TimedPoseApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = ConfigManager::load();
        let audio = AudioService::new(config.enable_audio, config.volume);
        let preloader = ImagePreloader::new(4);
        let presets = SessionPreset::default_presets();

        let selected_preset_idx = presets
            .iter()
            .position(|p| p.id == config.default_preset_id)
            .unwrap_or(4); // Default to Academic Warmup

        let mut app = Self {
            config: config.clone(),
            audio,
            preloader,
            presets,
            selected_preset_idx,
            deck_filter: DeckFilter::All,
            scanned_poses: Vec::new(),
            session: None,
            active_texture: None,
            active_decoded: None,
            canvas_filters: CanvasFilterSettings::default(),
            pending_texture_update: None,
            pan: Vec2::ZERO,
            zoom: 1.0,
            screen: AppScreen::Home,
            always_on_top: config.always_on_top,
            zen_mode: false,
        };

        // Scan folder if previously saved
        if let Some(folder) = &config.last_folder {
            if folder.exists() {
                app.scanned_poses = ImageScanner::scan_directory(folder);
            }
        }

        // Apply dark style
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        app
    }

    fn start_session_with_items(&mut self, items: Vec<PoseItem>) {
        if items.is_empty() {
            return;
        }

        let mut deck = Deck::new(items);
        deck.set_filter(self.deck_filter);

        let preset = self
            .presets
            .get(self.selected_preset_idx)
            .cloned()
            .unwrap_or_else(SessionPreset::fixed_60s);

        let mut session = SessionController::new(deck, preset);
        let events = session.start();
        self.session = Some(session);
        self.screen = AppScreen::ActiveSession;
        self.handle_session_events(events);
    }

    fn handle_session_events(&mut self, events: Vec<SessionEvent>) {
        for event in events {
            match event {
                SessionEvent::PoseChanged { pose, .. } => {
                    self.load_pose_image(&pose.path);
                    // Preload next image in history/deck
                    if let Some(session) = &self.session {
                        if let Some(next_pose) = session.deck().current() {
                            self.preloader.queue_preload(next_pose.path.clone());
                        }
                    }
                }
                SessionEvent::WarningCue { .. } => {
                    self.audio.play_warning_tick();
                }
                SessionEvent::TransitionCue => {
                    self.audio.play_transition_chime();
                }
                SessionEvent::BreakStarted { .. } => {
                    self.audio.play_break_chime();
                }
                SessionEvent::BreakEnded => {
                    self.audio.play_transition_chime();
                }
                SessionEvent::SessionCompleted { summary } => {
                    self.screen = AppScreen::Review(summary);
                }
            }
        }
    }

    fn load_pose_image(&mut self, path: &PathBuf) {
        if let Ok(decoded) = self.preloader.get_or_load_sync(path) {
            self.active_decoded = Some(decoded.clone());
            self.refresh_texture();
            self.pan = Vec2::ZERO;
            self.zoom = 1.0;
        }
    }

    fn refresh_texture(&mut self) {
        if let Some(decoded) = &self.active_decoded {
            let color_image = process_image_filters(decoded, &self.canvas_filters);
            // We'll update the texture in the update loop where ctx is available
            self.pending_texture_update = Some(color_image);
        }
    }

    // Helper property to hold newly processed images for upload
    fn upload_pending_texture(&mut self, ctx: &egui::Context) {
        if let Some(color_img) = self.pending_texture_update.take() {
            self.active_texture = Some(ctx.load_texture("pose_texture", color_img, TextureOptions::LINEAR));
        }
    }

    fn execute_user_action(&mut self, action: UserAction, ctx: &egui::Context) {
        match action {
            UserAction::TogglePause => {
                if let Some(session) = &mut self.session {
                    session.toggle_pause();
                }
            }
            UserAction::NextPose => {
                if let Some(session) = &mut self.session {
                    let events = session.next_pose();
                    self.handle_session_events(events);
                }
            }
            UserAction::PrevPose => {
                if let Some(session) = &mut self.session {
                    let events = session.prev_pose();
                    self.handle_session_events(events);
                }
            }
            UserAction::ToggleFlipHorizontal => {
                self.canvas_filters.flip_horizontal = !self.canvas_filters.flip_horizontal;
            }
            UserAction::ToggleUpsideDown => {
                self.canvas_filters.invert_upside_down = !self.canvas_filters.invert_upside_down;
            }
            UserAction::ToggleGrayscale => {
                self.canvas_filters.grayscale = !self.canvas_filters.grayscale;
                if self.canvas_filters.grayscale {
                    self.canvas_filters.threshold = false;
                }
                self.refresh_texture();
            }
            UserAction::ToggleThreshold => {
                self.canvas_filters.threshold = !self.canvas_filters.threshold;
                if self.canvas_filters.threshold {
                    self.canvas_filters.grayscale = false;
                }
                self.refresh_texture();
            }
            UserAction::ToggleSquintBlur => {
                self.canvas_filters.squint_blur = !self.canvas_filters.squint_blur;
                self.refresh_texture();
            }
            UserAction::ToggleGrid => {
                self.canvas_filters.grid_overlay = !self.canvas_filters.grid_overlay;
            }
            UserAction::ToggleBookmark => {
                if let Some(session) = &mut self.session {
                    session.toggle_bookmark();
                }
            }
            UserAction::ToggleAlwaysOnTop => {
                self.always_on_top = !self.always_on_top;
                self.config.always_on_top = self.always_on_top;
                let _ = ConfigManager::save(&self.config);
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if self.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                }));
            }
            UserAction::ToggleZenMode => {
                self.zen_mode = !self.zen_mode;
            }
            UserAction::ResetPanZoom => {
                self.pan = Vec2::ZERO;
                self.zoom = 1.0;
            }
            UserAction::ExitToHome => {
                self.screen = AppScreen::Home;
                self.session = None;
            }
            UserAction::OpenSettings => {}
        }
    }
}

// Full app implementation
impl App for TimedPoseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Safely free textures from previous session when outside an active session.
        // Freeing at the top of the frame before any rendering occurs ensures no
        // draw calls in the current frame reference the freed texture ID.
        if self.screen != AppScreen::ActiveSession && self.active_texture.is_some() {
            self.active_texture = None;
            self.active_decoded = None;
        }

        // 2. Upload any pending processed textures to the GPU
        self.upload_pending_texture(ctx);

        // 3. Sync audio settings
        self.audio.set_enabled(self.config.enable_audio);
        self.audio.set_volume(self.config.volume);

        // 4. Handle tablet hotkeys
        if let Some(action) = HotkeyManager::handle_input(ctx) {
            self.execute_user_action(action, ctx);
        }

        // 5. Periodic session update (advance timers, audio cues, and transitions)
        if self.screen == AppScreen::ActiveSession {
            ctx.request_repaint();
            if let Some(session) = &mut self.session {
                let events = session.update();
                self.handle_session_events(events);
            }
        }

        // 6. Screen routing
        match &self.screen {
            AppScreen::Home => {
                CentralPanel::default().show(ctx, |ui| {
                    if let Some(action) = HomeRenderer::render(
                        ui,
                        &mut self.config,
                        &self.presets,
                        &mut self.selected_preset_idx,
                        &mut self.deck_filter,
                        self.scanned_poses.len(),
                    ) {
                        match action {
                            HomeAction::PickFolder => {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    ConfigManager::add_recent_folder(&mut self.config, folder.clone());
                                    let _ = ConfigManager::save(&self.config);
                                    self.scanned_poses = ImageScanner::scan_directory(&folder);
                                }
                            }
                            HomeAction::SelectRecentFolder(folder) => {
                                self.config.last_folder = Some(folder.clone());
                                let _ = ConfigManager::save(&self.config);
                                self.scanned_poses = ImageScanner::scan_directory(&folder);
                            }
                            HomeAction::StartSession => {
                                let poses = self.scanned_poses.clone();
                                self.start_session_with_items(poses);
                            }
                        }
                    }
                });
            }
            AppScreen::ActiveSession => {
                CentralPanel::default()
                    .frame(egui::Frame::none().fill(Color32::from_rgb(18, 18, 20)))
                    .show(ctx, |ui| {
                        // Render Canvas
                        if let Some(tex) = &self.active_texture {
                            render_canvas(ui, tex, &self.canvas_filters, &mut self.pan, &mut self.zoom);
                        }

                        // Render HUD Overlay
                        if let Some(session) = &mut self.session {
                            if let Some(action) = HudRenderer::render(
                                ui,
                                session,
                                &mut self.canvas_filters,
                                self.always_on_top,
                                self.zen_mode,
                            ) {
                                self.execute_user_action(action, ctx);
                            }
                        }
                    });
            }
            AppScreen::Review(summary) => {
                let summary_clone = summary.clone();
                CentralPanel::default().show(ctx, |ui| {
                    if let Some(action) = ReviewRenderer::render(ui, &summary_clone) {
                        match action {
                            ReviewAction::StartBookmarkedOnly(bookmarked_paths) => {
                                let items = timedpose_core::Deck::from_paths(&bookmarked_paths)
                                    .bookmarked_items()
                                    .into_iter()
                                    .cloned()
                                    .collect();
                                self.start_session_with_items(items);
                            }
                            ReviewAction::RestartPreset => {
                                let poses = self.scanned_poses.clone();
                                self.start_session_with_items(poses);
                            }
                            ReviewAction::ReturnHome => {
                                self.screen = AppScreen::Home;
                            }
                        }
                    }
                });
            }
        }
    }
}
