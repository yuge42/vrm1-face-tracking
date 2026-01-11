use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Directory for user VRM models
    pub user_vrm_dir: PathBuf,
    /// Default VRM model filename in user directory
    pub default_vrm_model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let user_vrm_dir = get_user_vrm_dir();
        Self {
            user_vrm_dir,
            default_vrm_model: "model.vrm".to_string(),
        }
    }
}

impl AppConfig {
    /// Load configuration from file, or create default if not exists
    pub fn load_or_create() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = get_config_file_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = get_config_file_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        println!("Configuration saved to: {}", config_path.display());
        Ok(())
    }

    /// Ensure user VRM directory exists
    pub fn ensure_user_vrm_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.user_vrm_dir)?;
        Ok(())
    }
}

/// Get the user's VRM models directory
fn get_user_vrm_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "vrm1-face-tracking") {
        proj_dirs.data_dir().join("vrm_models")
    } else {
        // Fallback to current directory if we can't determine project dirs
        PathBuf::from("user_vrm_models")
    }
}

/// Get the configuration file path
fn get_config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "vrm1-face-tracking") {
        Ok(proj_dirs.config_dir().join("config.toml"))
    } else {
        Err("Could not determine config directory".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.default_vrm_model, "model.vrm");
        assert!(!config.user_vrm_dir.as_os_str().is_empty());
    }
}
