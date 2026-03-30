use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::BotCommand;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::synology::{DsmApi, TorrentDropper};
use super::handlers::{self, AppState};

pub async fn run_bot(config: Config, shared_config: Arc<RwLock<Config>>, dsm: DsmApi, dropper: TorrentDropper) {
    let bot = Bot::new(&config.telegram.bot_token);

    let commands = vec![
        BotCommand::new("status", "Active downloads"),
        BotCommand::new("all", "All downloads"),
        BotCommand::new("help", "Show help"),
    ];
    let _ = bot.set_my_commands(commands).await;

    let dsm_notifier = DsmApi::new();
    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("http client");

    let state = Arc::new(AppState {
        config: shared_config,
        dsm,
        dropper,
        http,
        pending: RwLock::new(std::collections::HashMap::new()),
    });

    let notifier_bot = bot.clone();
    let notifier_config = config.clone();
    tokio::spawn(async move {
        crate::notifier::run_notifier(notifier_bot, dsm_notifier, notifier_config).await;
    });

    log::info!("Bot is running.");

    let state_cb = state.clone();
    let handler = dptree::entry()
        .branch(
            Update::filter_message().endpoint(
                move |bot: Bot, msg: Message, state: Arc<AppState>| async move {
                    let config = state.config.read().await;
                    if !handlers::check_authorized(&config, &msg) {
                        return ResponseResult::Ok(());
                    }
                    drop(config);

                    if let Some(text) = msg.text() {
                        let lower = text.trim().to_lowercase();
                        if lower == "/start" || lower == "/help" {
                            handlers::handle_help(&bot, &msg).await?;
                        } else if lower == "/status" {
                            handlers::handle_status(&bot, &msg, &state, false).await?;
                        } else if lower == "/all" {
                            handlers::handle_status(&bot, &msg, &state, true).await?;
                        } else {
                            handlers::handle_text(&bot, &msg).await?;
                        }
                    } else if let Some(doc) = msg.document() {
                        handlers::handle_document(&bot, &msg, doc, &state).await?;
                    }
                    Ok(())
                },
            ),
        )
        .branch(
            Update::filter_callback_query().endpoint(
                move |bot: Bot, q: teloxide::types::CallbackQuery| {
                    let state = state_cb.clone();
                    async move { handlers::handle_callback(&bot, &q, &state).await }
                },
            ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(teloxide::dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
