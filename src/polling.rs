use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::db::Database;
use crate::discord;
use crate::riot_api::{lol::RiotClient, valorant::HenrikClient};

pub async fn run(
    http: Arc<serenity::http::Http>,
    db: Arc<tokio::sync::Mutex<Database>>,
    henrik_client: Arc<HenrikClient>,
    riot_client: Arc<RiotClient>,
    notification_channel_id: serenity::model::id::ChannelId,
    paused: Arc<AtomicBool>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        if paused.load(Ordering::Relaxed) {
            continue;
        }

        let accounts = db.lock().await.get_accounts();

        for mut account in accounts {
            // report new val matches
            let val_matches = match henrik_client
                .get_matches(&account.val_name, &account.val_tag, &account.val_region)
                .await
            {
                Ok(matches) => matches,
                Err(_) => Vec::new(),
            };

            let new_val_matches = val_matches
                .iter()
                .take_while(|m| m.metadata.matchid.parse().ok() != account.last_seen_val_match_id)
                .collect::<Vec<_>>();

            for m in new_val_matches {
                let msg = discord::format_match_message(
                    &account.discord_name,
                    &discord::val_match_to_result(m, &account.val_puuid),
                );

                if let Err(e) = notification_channel_id.say(&http, msg).await {
                    eprintln!("failed to send val match notification: {e}");
                }
            }

            if let Some(newest) = val_matches.first() {
                if let Ok(newest_id) = newest.metadata.matchid.parse() {
                    account.last_seen_val_match_id = Some(newest_id);
                }
            }

            // user has lol_region only if they play lol
            if let Some(lol_region) = &account.lol_region {
                let lol_match_ids = match riot_client
                    .get_match_ids(&account.lol_puuid, lol_region)
                    .await
                {
                    Ok(ids) => ids,
                    Err(_) => Vec::new(),
                };

                let new_lol_match_ids = lol_match_ids
                    .iter()
                    .take_while(|id| account.last_seen_lol_match_id.as_deref() != Some(id.as_str()))
                    .collect::<Vec<_>>();

                for match_id in new_lol_match_ids {
                    let summary = match riot_client.get_match(match_id, lol_region).await {
                        Ok(summary) => summary,
                        Err(_) => continue,
                    };

                    let msg = discord::format_match_message(
                        &account.discord_name,
                        &discord::lol_match_to_result(&summary, &account.lol_puuid),
                    );

                    if let Err(e) = notification_channel_id.say(&http, msg).await {
                        eprintln!("failed to send lol match notification: {e}");
                    }
                }

                if let Some(newest_id) = lol_match_ids.first() {
                    account.last_seen_lol_match_id = Some(newest_id.clone());
                }
            }

            // update last seen match ids
            if let Err(e) = db.lock().await.update_account(account) {
                eprintln!("failed to persist updated account: {e}");
            }
        }
    }
}
