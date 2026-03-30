use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub telegram: TelegramConfig,
    #[serde(default)]
    pub destinations: DestinationsConfig,
    #[serde(default)]
    pub notifications: NotificationsConfig,
    /// Watch folder for Download Station auto-download
    #[serde(default)]
    pub watch: Option<WatchConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub authorized_users: Vec<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DestinationsConfig {
    #[serde(flatten)]
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WatchConfig {
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
        Self { poll_interval_secs: default_poll_interval() }
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

    pub fn credentials_folder(&self) -> String {
        // Service credentials are stored in the first destination or a default path
        if let Some(watch) = &self.watch {
            return watch.folder.clone();
        }
        "/volume1/downloads".to_string()
    }
}

impl DestinationsConfig {
    pub fn resolve(&self, alias: &str) -> Option<&str> {
        self.aliases.get(&alias.to_lowercase()).map(|s| s.as_str())
    }

    pub fn default_destination(&self) -> Option<&str> {
        self.aliases.get("default").map(|s| s.as_str())
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.aliases.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    }
}
