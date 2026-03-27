use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub telegram: TelegramConfig,
    pub watch: WatchConfig,
    #[serde(default)]
    pub notifications: NotificationsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub authorized_users: Vec<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WatchConfig {
    /// Path to the folder Download Station watches for .torrent files.
    pub folder: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NotificationsConfig {
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
}

fn default_poll_interval() -> u64 {
    30
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: default_poll_interval(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
