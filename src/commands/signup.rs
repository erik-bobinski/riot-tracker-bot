use crate::db;
use crate::types::{Context, Error};

/// Get your riot account's match results reported
#[poise::command(slash_command)]
pub async fn signup(
    ctx: Context<'_>,
    #[description = "Riot Name"] riot_name: String,
    #[description = "Riot Tag after the # (e.g. NA1)"] riot_tag: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let valorant_account = ctx
        .data()
        .henrik_client
        .get_account(&riot_name, &riot_tag)
        .await
        .ok();

    let lol_account = ctx
        .data()
        .riot_client
        .get_account(&riot_name, &riot_tag)
        .await
        .ok();

    if valorant_account.is_none() && lol_account.is_none() {
        ctx.say("Couldn't find recent account data for that Riot ID :(")
            .await?;
        return Ok(());
    }

    let (val_puuid, val_region, last_seen_val_match_id) =
        if let Some(valorant_account) = valorant_account {
            // unlike lol, henrik's account lookup already tells us the region, so no
            // brute-force detection needed; just baseline to the newest match like lol does
            let val_matches = ctx
                .data()
                .henrik_client
                .get_matches(&riot_name, &riot_tag, &valorant_account.region)
                .await?;
            let last_seen_val_match_id = val_matches
                .first()
                .map(|m| m.metadata.matchid.parse())
                .transpose()?;

            (
                valorant_account.puuid,
                Some(valorant_account.region),
                last_seen_val_match_id,
            )
        } else {
            (String::new(), None, None)
        };

    let (lol_puuid, lol_region, last_seen_lol_match_id) = if let Some(lol_account) = lol_account {
        // baseline to the newest match on signup instead of backfilling full match
        // data for everyone's existing history, to avoid a burst of api calls
        let detected = ctx
            .data()
            .riot_client
            .detect_region(&lol_account.puuid)
            .await?;
        let (lol_region, last_seen_lol_match_id) = match detected {
            Some((region, match_ids)) => (Some(region), match_ids.into_iter().next()),
            None => (None, None),
        };

        (lol_account.puuid, lol_region, last_seen_lol_match_id)
    } else {
        (String::new(), None, None)
    };

    let mut db = ctx.data().db.lock().await;

    db.add_account(db::DatabaseAccount {
        discord_user_id: ctx.author().id.get(),
        discord_name: ctx.author().name.clone(),
        riot_name,
        riot_tag,
        val_puuid,
        val_region,
        last_seen_val_match_id,
        lol_puuid,
        lol_region,
        last_seen_lol_match_id,
        added_at: chrono::Utc::now(),
        ..Default::default()
    })?;

    ctx.say(format!("**{}** just signed up!", ctx.author().name))
        .await?;
    Ok(())
}
