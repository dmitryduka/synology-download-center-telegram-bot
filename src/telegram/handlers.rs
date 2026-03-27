use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::Document;

use crate::config::Config;
use crate::synology::TorrentDropper;

pub struct AppState {
    pub config: Config,
    pub dropper: TorrentDropper,
    pub http: reqwest::Client,
}

pub async fn handle_help(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    let text =
        "**Synology Download Bot**\n\n\
         Send me a `.torrent` file and I'll add it to Download Station.\n\n\
         /help — Show this message";

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

pub async fn handle_document(
    bot: &Bot,
    msg: &Message,
    doc: &Document,
    state: &Arc<AppState>,
) -> ResponseResult<()> {
    let file_name = doc.file_name.as_deref().unwrap_or("unknown");

    if !file_name.ends_with(".torrent") {
        bot.send_message(msg.chat.id, "Only .torrent files are supported.")
            .await?;
        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        format!("📥 Receiving: {}...", file_name),
    )
    .await?;

    let file = bot.get_file(&doc.file.id).await?;
    let download_url = format!(
        "https://api.telegram.org/file/bot{}/{}",
        state.config.telegram.bot_token, file.path
    );
    let buf = match state.http.get(&download_url).send().await {
        Ok(resp) => resp.bytes().await.map(|b| b.to_vec()).unwrap_or_default(),
        Err(e) => {
            bot.send_message(
                msg.chat.id,
                format!("Failed to download from Telegram: {}", e),
            )
            .await?;
            return Ok(());
        }
    };

    if buf.is_empty() {
        bot.send_message(msg.chat.id, "Downloaded file is empty.")
            .await?;
        return Ok(());
    }

    match state.dropper.drop_torrent(file_name, &buf).await {
        Ok(path) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "✅ Sent to Download Station!\n📁 {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
            )
            .await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Failed: {}", e))
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_text_message(
    bot: &Bot,
    msg: &Message,
    _text: &str,
    _state: &Arc<AppState>,
) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        "Send me a .torrent file to start a download.\n\nType /help for more info.",
    )
    .await?;
    Ok(())
}

pub fn user_id_from_msg(msg: &Message) -> Option<u64> {
    msg.from.as_ref().map(|u| u.id.0)
}

pub fn check_authorized(config: &Config, msg: &Message) -> bool {
    if let Some(uid) = user_id_from_msg(msg) {
        config.telegram.authorized_users.contains(&uid)
    } else {
        false
    }
}
