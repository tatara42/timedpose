//! Infrastructure services for Timed Pose:
//! - Persistent settings and OS standard configuration paths (`config`)
//! - Fast parallel image file discovery with header-only orientation probing (`scanner`)
//! - Procedural synthesized audio chimes, warning ticks, and bells (`audio`)
//! - Asynchronous background image pre-decoder with bounded LRU cache (`image_loader`)

pub mod audio;
pub mod config;
pub mod image_loader;
pub mod scanner;

pub use audio::AudioService;
pub use config::{AppConfig, CanvasTheme, ConfigManager};
pub use image_loader::{DecodedImage, ImagePreloader};
pub use scanner::ImageScanner;
