use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::Config;

const TMPFS_DIR: &str = "/tmp/synobot";
const STATUS_FILE: &str = "status.json";
const CONFIG_FILE: &str = "config_read.json";
const CONFIG_UPDATE_FILE: &str = "config_update.json";

#[derive(Clone)]
pub struct BridgeState {
    pub config: Arc<RwLock<Config>>,
    pub config_path: PathBuf,
    pub watch_folder: PathBuf,
}

#[derive(Serialize)]
struct StatusData {
    bot_running: bool,
    version: String,
    watch_folder: String,
}

#[derive(Serialize)]
struct ConfigData {
    bot_token_masked: String,
    authorized_users: Vec<u64>,
    watch_folder: String,
}

fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        return "***".to_string();
    }
    format!("{}...{}", &token[..6], &token[token.len() - 4..])
}

/// Create /tmp/synobot and symlink it from the 3rdparty directory.
pub async fn setup_tmpfs_bridge(thirdparty_dir: &Path) -> Result<(), String> {
    let tmpfs = Path::new(TMPFS_DIR);
    tokio::fs::create_dir_all(tmpfs)
        .await
        .map_err(|e| format!("create {}: {}", TMPFS_DIR, e))?;

    let link_path = thirdparty_dir.join("data");
    let _ = tokio::fs::remove_file(&link_path).await;
    let _ = tokio::fs::remove_dir(&link_path).await;

    #[cfg(unix)]
    tokio::fs::symlink(tmpfs, &link_path)
        .await
        .map_err(|e| format!("symlink: {}", e))?;

    log::info!("tmpfs bridge: {} -> {}", link_path.display(), TMPFS_DIR);
    Ok(())
}

/// Write status and config to tmpfs periodically.
pub async fn run_status_writer(state: BridgeState) {
    let tmpfs = Path::new(TMPFS_DIR);
    loop {
        let config = state.config.read().await;
        let status = StatusData {
            bot_running: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            watch_folder: config.watch.folder.clone(),
        };
        let config_data = ConfigData {
            bot_token_masked: mask_token(&config.telegram.bot_token),
            authorized_users: config.telegram.authorized_users.clone(),
            watch_folder: config.watch.folder.clone(),
        };
        drop(config);

        if let Ok(j) = serde_json::to_string(&status) { let _ = tokio::fs::write(tmpfs.join(STATUS_FILE), j).await; }
        if let Ok(j) = serde_json::to_string(&config_data) { let _ = tokio::fs::write(tmpfs.join(CONFIG_FILE), j).await; }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

/// Watch for config_update.json in the watch folder.
pub async fn run_config_watcher(state: BridgeState) {
    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let path = state.watch_folder.join(CONFIG_UPDATE_FILE);
        if !path.exists() { continue; }
        log::info!("Config update detected");
        let content = match tokio::fs::read_to_string(&path).await { Ok(c) => c, Err(_) => { let _ = tokio::fs::remove_file(&path).await; continue; } };
        let _ = tokio::fs::remove_file(&path).await;
        let update: serde_json::Value = match serde_json::from_str(&content) { Ok(v) => v, Err(_) => continue };
        let mut config = state.config.write().await;
        if let Some(t) = update.get("bot_token").and_then(|v| v.as_str()) { if !t.is_empty() && !t.contains("...") { config.telegram.bot_token = t.to_string(); } }
        if let Some(u) = update.get("authorized_users").and_then(|v| v.as_array()) { let p: Vec<u64> = u.iter().filter_map(|v| v.as_u64()).collect(); if !p.is_empty() { config.telegram.authorized_users = p; } }
        if let Some(f) = update.get("watch_folder").and_then(|v| v.as_str()) { if !f.is_empty() { config.watch.folder = f.to_string(); } }
        if let Err(e) = config.save(&state.config_path) { log::error!("Save failed: {}", e); } else { log::info!("Config saved"); }
    }
}
