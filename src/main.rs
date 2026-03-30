mod config;
mod notifier;
mod synology;
mod telegram;
mod web;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const THIRDPARTY_BASE: &str = "/usr/syno/synoman/webman/3rdparty/SynoTelegramBot";

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    log::info!("Loading config from {:?}", config_path);

    let config = match config::Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    log::info!(
        "Config loaded. Destinations: {:?}, Authorized users: {:?}",
        config.destinations.aliases.keys().collect::<Vec<_>>(),
        config.telegram.authorized_users,
    );

    let thirdparty_dir = PathBuf::from(THIRDPARTY_BASE);
    if thirdparty_dir.exists() {
        if let Err(e) = web::setup_tmpfs_bridge(&thirdparty_dir).await {
            log::warn!("tmpfs bridge failed: {}", e);
        }
    }

    let shared_config = Arc::new(RwLock::new(config.clone()));
    let dsm = synology::DsmApi::new();
    let watch_folder = config.watch.as_ref().map(|w| w.folder.as_str()).unwrap_or("/volume1/watch");
    let dropper = synology::TorrentDropper::new(watch_folder);

    let bridge = web::BridgeState {
        config: shared_config.clone(),
        config_path,
        dsm: dsm.clone(),
    };

    tokio::spawn(web::run_status_writer(bridge.clone()));
    tokio::spawn(web::run_config_watcher(bridge));

    telegram::bot::run_bot(config, shared_config, dsm, dropper).await;
}
