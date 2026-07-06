use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::db::{self, Database};
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
            // henrik can't resolve an account with no match history yet, so a riot id
            // entered at signup may still be unresolved; retry it here now that the
            // player may have finished a game since then
            if account.val_region.is_none() {
                if let Ok(valorant_account) = henrik_client
                    .get_account(&account.riot_name, &account.riot_tag)
                    .await
                {
                    account.val_puuid = valorant_account.puuid;
                    account.val_region = Some(valorant_account.region);
                }
            }

            // user has val_region only if they play valorant
            if let Some(val_region) = account.val_region.clone() {
                let val_matches = match henrik_client
                    .get_matches(&account.val_puuid, &val_region)
                    .await
                {
                    Ok(matches) => matches,
                    Err(_) => Vec::new(),
                };

                if account.reported_val_match_ids.is_empty() {
                    // first fetch for this account: baseline to the current window
                    // instead of reporting the player's existing history
                    account.reported_val_match_ids = val_matches
                        .iter()
                        .map(|m| m.metadata.matchid.clone())
                        .collect();
                    account.reported_val_match_ids.truncate(db::REPORTED_MATCH_CAP);
                } else {
                    let new_val_matches = val_matches
                        .iter()
                        .filter(|m| !account.reported_val_match_ids.contains(&m.metadata.matchid))
                        .collect::<Vec<_>>();

                    for m in new_val_matches {
                        let msg = discord::format_match_message(
                            &account.discord_name,
                            &discord::val_match_to_result(m, &account.val_puuid),
                        );

                        if let Err(e) = notification_channel_id.say(&http, msg).await {
                            eprintln!("failed to send val match notification: {e}");
                        }

                        db::remember_match(&mut account.reported_val_match_ids, &m.metadata.matchid);
                    }
                }
            }

            // lol region can't be detected until the account has match history, so it
            // may still be unresolved from signup; retry it here now that the player
            // may have finished a game since then
            if account.lol_region.is_none() {
                // the account lookup itself may also have failed at signup
                if account.lol_puuid.is_empty() {
                    if let Ok(lol_account) = riot_client
                        .get_account(&account.riot_name, &account.riot_tag)
                        .await
                    {
                        account.lol_puuid = lol_account.puuid;
                    }
                }

                if !account.lol_puuid.is_empty() {
                    if let Ok(Some(region)) =
                        riot_client.detect_region(&account.lol_puuid).await
                    {
                        account.lol_region = Some(region);
                    }
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

                if account.reported_lol_match_ids.is_empty() {
                    // first fetch for this account: baseline to the current window
                    // instead of reporting the player's existing history
                    account.reported_lol_match_ids = lol_match_ids;
                    account.reported_lol_match_ids.truncate(db::REPORTED_MATCH_CAP);
                } else {
                    let new_lol_match_ids = lol_match_ids
                        .iter()
                        .filter(|id| !account.reported_lol_match_ids.contains(id))
                        .collect::<Vec<_>>();

                    for match_id in new_lol_match_ids {
                        let summary = match riot_client.get_match(match_id, lol_region).await {
                            Ok(summary) => summary,
                            // not remembered, so the next poll retries this match
                            Err(_) => continue,
                        };

                        let msg = discord::format_match_message(
                            &account.discord_name,
                            &discord::lol_match_to_result(&summary, &account.lol_puuid),
                        );

                        if let Err(e) = notification_channel_id.say(&http, msg).await {
                            eprintln!("failed to send lol match notification: {e}");
                        }

                        db::remember_match(&mut account.reported_lol_match_ids, match_id);
                    }
                }
            }

            // persist the updated reported-match rings
            if let Err(e) = db.lock().await.update_account(account) {
                eprintln!("failed to persist updated account: {e}");
            }
        }
    }
}
