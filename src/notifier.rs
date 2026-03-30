use std::collections::HashMap;
use std::time::Duration;
use teloxide::prelude::*;

use crate::config::Config;
use crate::synology::dsm_api::{DsmApi, format_size};

pub async fn run_notifier(bot: Bot, dsm: DsmApi, config: Config) {
    let interval = Duration::from_secs(config.notifications.poll_interval_secs);
    let mut known: HashMap<String, i64> = HashMap::new();
    let mut first = true;
    let mut poll_count: u64 = 0;

    log::info!("Notifier started (interval: {}s)", config.notifications.poll_interval_secs);

    loop {
        tokio::time::sleep(interval).await;

        let tasks = match dsm.list_tasks().await {
            Ok(tl) => tl.task,
            Err(e) => {
                log::warn!("Notifier poll failed: {}", e);
                continue;
            }
        };

        let mut current: HashMap<String, i64> = HashMap::new();
        for t in &tasks {
            current.insert(t.id.clone(), t.status);
        }

        if !first {
            for t in &tasks {
                let prev = known.get(&t.id).copied();

                // Task completed: status 5 (finished), 7 (seeding), or 8 (done/filehosting)
                let is_done = matches!(t.status, 5 | 7 | 8);
                let was_done = prev.map(|s| matches!(s, 5 | 7 | 8)).unwrap_or(false);
                if is_done && !was_done {
                    log::info!("Download complete: {}", t.title);
                    let text = format!(
                        "✅ Download complete!\n\n{}\nSize: {}",
                        t.title, format_size(t.size),
                    );
                    for uid in &config.telegram.authorized_users {
                        if let Err(e) = bot.send_message(ChatId(*uid as i64), &text).await {
                            log::error!("Failed to send notification: {}", e);
                        }
                    }
                }

                // Task just errored
                if t.status == 10 && prev != Some(10) {
                    log::info!("Download error: {}", t.title);
                    let text = format!("❌ Download error: {}", t.title);
                    for uid in &config.telegram.authorized_users {
                        let _ = bot.send_message(ChatId(*uid as i64), &text).await;
                    }
                }

                // New task appeared (wasn't in previous poll)
                if prev.is_none() && t.is_active() {
                    log::info!("New download detected: {}", t.title);
                }
            }
        } else {
            log::info!("Notifier: initial scan found {} tasks", tasks.len());
        }

        known = current;
        first = false;
        poll_count += 1;
        if poll_count % 20 == 0 {
            log::info!("Notifier: {} polls, tracking {} tasks", poll_count, known.len());
        }
    }
}
