use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application appearance theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CanvasTheme {
    #[default]
    Dark,
    NeutralGray50,
    Light,
}

/// Persistent user configuration and preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub last_folder: Option<PathBuf>,
    pub recent_folders: Vec<PathBuf>,
    pub default_preset_id: String,
    pub volume: f32,
    pub enable_audio: bool,
    pub warning_threshold_secs: u32,
    pub auto_rest_break_interval_mins: Option<u32>,
    pub window_width: f32,
    pub window_height: f32,
    pub always_on_top: bool,
    pub theme: CanvasTheme,
    pub squint_blur_radius: f32,
    pub threshold_cutoff: f32,
    pub ghost_opacity: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_folder: None,
            recent_folders: Vec::new(),
            default_preset_id: "academic_warmup".to_string(),
            volume: 0.7,
            enable_audio: true,
            warning_threshold_secs: 5,
            auto_rest_break_interval_mins: Some(25),
            window_width: 800.0,
            window_height: 600.0,
            always_on_top: false,
            theme: CanvasTheme::Dark,
            squint_blur_radius: 8.0,
            threshold_cutoff: 0.5,
            ghost_opacity: 0.85,
        }
    }
}

/// Manager for loading and saving `AppConfig` in the standard OS configuration directory.
pub struct ConfigManager;

impl ConfigManager {
    const QUALIFIER: &'static str = "com";
    const ORGANIZATION: &'static str = "timedpose";
    const APPLICATION: &'static str = "TimedPose";
    const CONFIG_FILE_NAME: &'static str = "config.toml";

    /// Retrieves the path to the configuration file in standard OS config directory:
    /// - Linux: ~/.config/TimedPose/config.toml
    /// - macOS: ~/Library/Application Support/com.timedpose.TimedPose/config.toml
    /// - Windows: %APPDATA%\timedpose\TimedPose\config.toml
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from(Self::QUALIFIER, Self::ORGANIZATION, Self::APPLICATION)
            .map(|dirs| dirs.config_dir().join(Self::CONFIG_FILE_NAME))
    }

    /// Loads the configuration from disk, returning default values if file does not exist or errors.
    pub fn load() -> AppConfig {
        let Some(path) = Self::config_path() else {
            return AppConfig::default();
        };

        if !path.exists() {
            return AppConfig::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|err| {
                eprintln!("Warning: Failed to parse config file: {err}. Using defaults.");
                AppConfig::default()
            }),
            Err(err) => {
                eprintln!("Warning: Failed to read config file: {err}. Using defaults.");
                AppConfig::default()
            }
        }
    }

    /// Saves the configuration to disk, creating parent directories if necessary.
    pub fn save(config: &AppConfig) -> std::io::Result<()> {
        let Some(path) = Self::config_path() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not locate OS config directory",
            ));
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(config).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        fs::write(path, toml_string)
    }

    /// Records a newly selected folder into recent folders list.
    pub fn add_recent_folder(config: &mut AppConfig, folder: PathBuf) {
        config.last_folder = Some(folder.clone());
        config.recent_folders.retain(|f| f != &folder);
        config.recent_folders.insert(0, folder);
        if config.recent_folders.len() > 10 {
            config.recent_folders.truncate(10);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let mut config = AppConfig::default();
        config.volume = 0.5;
        config.always_on_top = true;
        config.theme = CanvasTheme::NeutralGray50;
        config.last_folder = Some(PathBuf::from("/test/poses"));

        let serialized = toml::to_string(&config).expect("failed to serialize");
        let deserialized: AppConfig = toml::from_str(&serialized).expect("failed to deserialize");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_recent_folders_cap() {
        let mut config = AppConfig::default();
        for i in 0..15 {
            ConfigManager::add_recent_folder(&mut config, PathBuf::from(format!("/folder_{i}")));
        }
        assert_eq!(config.recent_folders.len(), 10);
        assert_eq!(config.recent_folders[0], PathBuf::from("/folder_14"));
    }
}
