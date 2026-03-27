use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

#[derive(Clone)]
pub struct WebState {
    pub config: Arc<RwLock<Config>>,
    pub config_path: PathBuf,
}

#[derive(Serialize)]
struct StatusResponse {
    bot_running: bool,
    version: String,
    watch_folder: String,
}

#[derive(Serialize)]
struct ConfigResponse {
    bot_token_masked: String,
    authorized_users: Vec<u64>,
    watch_folder: String,
}

#[derive(Deserialize)]
struct ConfigUpdateRequest {
    bot_token: Option<String>,
    authorized_users: Option<Vec<u64>>,
    watch_folder: Option<String>,
}

fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        return "***".to_string();
    }
    format!("{}...{}", &token[..6], &token[token.len() - 4..])
}

async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("ui.html"))
}

async fn get_status(State(state): State<WebState>) -> Json<StatusResponse> {
    let config = state.config.read().await;
    Json(StatusResponse {
        bot_running: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        watch_folder: config.watch.folder.clone(),
    })
}

async fn get_config(State(state): State<WebState>) -> Json<ConfigResponse> {
    let config = state.config.read().await;
    Json(ConfigResponse {
        bot_token_masked: mask_token(&config.telegram.bot_token),
        authorized_users: config.telegram.authorized_users.clone(),
        watch_folder: config.watch.folder.clone(),
    })
}

async fn update_config(
    State(state): State<WebState>,
    Json(update): Json<ConfigUpdateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut config = state.config.write().await;

    if let Some(token) = update.bot_token {
        if !token.is_empty() && !token.contains("...") {
            config.telegram.bot_token = token;
        }
    }
    if let Some(users) = update.authorized_users {
        config.telegram.authorized_users = users;
    }
    if let Some(folder) = update.watch_folder {
        if !folder.is_empty() {
            config.watch.folder = folder;
        }
    }

    if let Err(e) = config.save(&state.config_path) {
        log::error!("Failed to save config: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    log::info!("Config updated and saved");
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn run_web_server(state: WebState) {
    let app = Router::new()
        .route("/", get(serve_ui))
        .route("/api/status", get(get_status))
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8008));
    log::info!("Settings UI at http://0.0.0.0:{}", addr.port());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
