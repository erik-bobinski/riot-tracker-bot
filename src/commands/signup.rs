use crate::db;
use crate::types::{Context, Error};

/// Get your riot account's match results reported
#[poise::command(slash_command)]
pub async fn signup(
    ctx: Context<'_>,
    #[description = "before the # (e.g. syan)"] riot_name: String,
    #[description = "after the # (e.g. NA1)"] riot_tag: String,
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

    // the reported-match rings start empty; the first poll baselines them to the
    // current match window, so matches finished before signup are never reported
    let (val_puuid, val_region) = if let Some(valorant_account) = valorant_account {
        // unlike lol, henrik's account lookup already tells us the region, so no
        // brute-force detection needed
        (valorant_account.puuid, Some(valorant_account.region))
    } else {
        (String::new(), None)
    };

    let (lol_puuid, lol_region) = if let Some(lol_account) = lol_account {
        let lol_region = ctx
            .data()
            .riot_client
            .detect_region(&lol_account.puuid)
            .await?;

        (lol_account.puuid, lol_region)
    } else {
        (String::new(), None)
    };

    let mut db = ctx.data().db.lock().await;

    db.add_account(db::DatabaseAccount {
        discord_user_id: ctx.author().id.get(),
        discord_name: ctx.author().name.clone(),
        riot_name,
        riot_tag,
        val_puuid,
        val_region,
        lol_puuid,
        lol_region,
        added_at: chrono::Utc::now(),
        ..Default::default()
    })?;

    ctx.say(format!("**{}** just signed up!", ctx.author().name))
        .await?;
    Ok(())
}
