use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{Document, InlineKeyboardButton, InlineKeyboardMarkup};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::synology::dsm_api::{DsmApi, Task, format_size};
use crate::synology::TorrentDropper;

pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub dsm: DsmApi,
    pub dropper: TorrentDropper,
    pub http: reqwest::Client,
    pub pending: RwLock<std::collections::HashMap<String, PendingTorrent>>,
}

pub struct PendingTorrent {
    pub filename: String,
    pub data: Vec<u8>,
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() }
    else { format!("{}…", s.chars().take(max - 1).collect::<String>()) }
}

fn format_task_line(t: &Task) -> String {
    let title = truncate(&t.title, 40);
    let pct = format!("{}%", t.progress());
    let size = format_size(t.size);
    let speed = if t.is_active() && t.speed() > 0 {
        format!("  {}/s", format_size(t.speed()))
    } else { String::new() };
    format!("{} {} — {} of {}{}", t.status_icon(), title, pct, size, speed)
}

pub async fn handle_help(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id,
        "📥 Synology Download Bot\n\n\
         Send a .torrent file to start a download.\n\n\
         /status — Active downloads\n\
         /all — All downloads\n\
         /help — This message"
    ).await?;
    Ok(())
}

pub async fn handle_status(bot: &Bot, msg: &Message, state: &Arc<AppState>, show_all: bool) -> ResponseResult<()> {
    match state.dsm.list_tasks().await {
        Ok(tl) => {
            let tasks: Vec<&Task> = if show_all {
                tl.task.iter().collect()
            } else {
                tl.task.iter().filter(|t| t.is_in_progress()).collect()
            };
            if tasks.is_empty() {
                bot.send_message(msg.chat.id, if show_all { "No downloads." } else { "No active downloads." }).await?;
                return Ok(());
            }
            let lines: Vec<String> = tasks.iter().map(|t| format_task_line(t)).collect();
            bot.send_message(msg.chat.id, lines.join("\n")).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!(
                "Cannot check status: {}\n\nOpen the DSM app and click 'Setup Bot Access'.", e
            )).await?;
        }
    }
    Ok(())
}

pub async fn handle_document(
    bot: &Bot, msg: &Message, doc: &Document, state: &Arc<AppState>,
) -> ResponseResult<()> {
    let file_name = doc.file_name.as_deref().unwrap_or("unknown");
    if !file_name.ends_with(".torrent") {
        bot.send_message(msg.chat.id, "Only .torrent files are supported.").await?;
        return Ok(());
    }

    log::info!("Receiving torrent: {}", file_name);
    bot.send_message(msg.chat.id, format!("📥 Receiving: {}...", file_name)).await?;

    let file = bot.get_file(&doc.file.id).await?;
    let (url, destinations) = {
        let config = state.config.read().await;
        let u = format!("https://api.telegram.org/file/bot{}/{}", config.telegram.bot_token, file.path);
        let d: Vec<(String, String)> = config.destinations.aliases.iter()
            .map(|(k, v)| (k.clone(), v.clone())).collect();
        (u, d)
    };

    let buf = match state.http.get(&url).send().await {
        Ok(r) => r.bytes().await.map(|b| b.to_vec()).unwrap_or_default(),
        Err(e) => { bot.send_message(msg.chat.id, format!("Failed: {}", e)).await?; return Ok(()); }
    };
    if buf.is_empty() { bot.send_message(msg.chat.id, "Empty file.").await?; return Ok(()); }

    // If only default destination or no destinations, use default directly
    if destinations.len() <= 1 {
        let dest = destinations.first().map(|(_, v)| v.as_str()).unwrap_or("downloads");
        return create_and_respond(bot, msg, state, file_name, &buf, dest).await;
    }

    // Store pending torrent and show destination buttons
    let key = format!("{}_{}", msg.chat.id, msg.id);
    {
        let mut pending = state.pending.write().await;
        pending.insert(key.clone(), PendingTorrent { filename: file_name.to_string(), data: buf });
    }

    let buttons: Vec<Vec<InlineKeyboardButton>> = destinations.iter()
        .map(|(name, _path)| {
            vec![InlineKeyboardButton::callback(
                format!("📁 {}", name),
                format!("dl:{}:{}", key, name),
            )]
        })
        .collect();

    bot.send_message(msg.chat.id, "Where to download?")
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .await?;

    Ok(())
}

async fn create_and_respond(
    bot: &Bot, msg: &Message, state: &Arc<AppState>,
    filename: &str, data: &[u8], dest_name: &str,
) -> ResponseResult<()> {
    let config = state.config.read().await;
    let dest_path = config.destinations.resolve(dest_name)
        .unwrap_or("downloads").to_string();
    drop(config);

    match state.dropper.drop_torrent(filename, data).await {
        Ok(saved_name) => {
            bot.send_message(msg.chat.id, format!("✅ {} → {}", truncate(filename, 30), dest_name)).await?;
            // Edit destination in background after DS picks it up
            let dsm = state.dsm.clone();
            let hint = saved_name.replace(".torrent", "");
            tokio::spawn(async move {
                if let Err(e) = dsm.set_task_destination(&hint, &dest_path).await {
                    log::warn!("Failed to set destination: {}", e);
                }
            });
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ {}", e)).await?;
        }
    }
    Ok(())
}

pub async fn handle_callback(bot: &Bot, q: &teloxide::types::CallbackQuery, state: &Arc<AppState>) -> ResponseResult<()> {
    log::info!("Callback received: {:?}", q.data);
    let data = match &q.data { Some(d) => d.clone(), None => return Ok(()) };
    if !data.starts_with("dl:") { return Ok(()); }

    let parts: Vec<&str> = data.splitn(3, ':').collect();
    if parts.len() < 3 { return Ok(()); }
    let key = parts[1];
    let dest_name = parts[2];

    let config = state.config.read().await;
    let dest_path = config.destinations.resolve(dest_name)
        .unwrap_or("downloads")
        .to_string();
    drop(config);

    let pending = {
        let mut map = state.pending.write().await;
        map.remove(key)
    };

    let chat_id = q.message.as_ref().map(|m| m.chat().id).unwrap_or(ChatId(0));

    if let Some(torrent) = pending {
        let config = state.config.read().await;
        let dest_path = config.destinations.resolve(dest_name)
            .unwrap_or("downloads").to_string();
        drop(config);

        match state.dropper.drop_torrent(&torrent.filename, &torrent.data).await {
            Ok(saved_name) => {
                bot.send_message(chat_id, format!("✅ {} → {}", truncate(&torrent.filename, 30), dest_name)).await?;
                let dsm = state.dsm.clone();
                let hint = saved_name.replace(".torrent", "");
                tokio::spawn(async move {
                    if let Err(e) = dsm.set_task_destination(&hint, &dest_path).await {
                        log::warn!("Failed to set destination: {}", e);
                    }
                });
            }
            Err(e) => {
                bot.send_message(chat_id, format!("❌ {}", e)).await?;
            }
        }
    } else {
        bot.send_message(chat_id, "Torrent expired. Please send again.").await?;
    }

    bot.answer_callback_query(&q.id).await?;
    Ok(())
}

pub async fn handle_text(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "Send a .torrent file or use /status, /help").await?;
    Ok(())
}

pub fn user_id_from_msg(msg: &Message) -> Option<u64> {
    msg.from.as_ref().map(|u| u.id.0)
}

pub fn check_authorized(config: &Config, msg: &Message) -> bool {
    user_id_from_msg(msg).map(|uid| config.telegram.authorized_users.contains(&uid)).unwrap_or(false)
}
