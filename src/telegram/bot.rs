use std::sync::Arc;
use teloxide::prelude::*;

use crate::config::Config;
use crate::synology::TorrentDropper;
use super::handlers::{self, AppState};

pub async fn run_bot(config: Config) {
    let bot = Bot::new(&config.telegram.bot_token);

    let dropper = TorrentDropper::new(&config.watch.folder);

    let http = reqwest::Client::builder()
        .build()
        .expect("failed to create HTTP client");

    let state = Arc::new(AppState {
        config: config.clone(),
        dropper,
        http,
    });

    log::info!("Bot is running. Watch folder: {}", config.watch.folder);

    let handler = Update::filter_message().endpoint(
        move |bot: Bot, msg: Message, state: Arc<AppState>| async move {
            if !handlers::check_authorized(&state.config, &msg) {
                log::warn!(
                    "Unauthorized access attempt from user {:?}",
                    handlers::user_id_from_msg(&msg)
                );
                return ResponseResult::Ok(());
            }

            if let Some(text) = msg.text() {
                let cmd = text.trim().to_lowercase();
                if cmd == "/start" || cmd == "/help" {
                    handlers::handle_help(&bot, &msg).await?;
                } else {
                    handlers::handle_text_message(&bot, &msg, text, &state).await?;
                }
            } else if let Some(doc) = msg.document() {
                handlers::handle_document(&bot, &msg, doc, &state).await?;
            }

            Ok(())
        },
    );

    Dispatcher::builder(bot, handler)
        .dependencies(teloxide::dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
