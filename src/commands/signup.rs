use crate::db;
use crate::types::{Context, Error};

#[poise::command(slash_command)]
pub async fn signup(
    ctx: Context<'_>,
    #[description = "Riot Name"] riot_name: String,
    #[description = "Riot Tag"] riot_tag: String,
) -> Result<(), Error> {
    let valorant_account = match ctx
        .data()
        .henrik_client
        .get_account(&riot_name, &riot_tag)
        .await
    {
        Ok(valorant_account) => valorant_account,
        Err(_) => {
            ctx.say("Couldn't find riot account :(").await?;
            return Ok(());
        }
    };

    let lol_account = match ctx
        .data()
        .riot_client
        .get_account(&riot_name, &riot_tag)
        .await
    {
        Ok(lol_account) => lol_account,
        Err(_) => {
            ctx.say("Couldn't find riot account :(").await?;
            return Ok(());
        }
    };

    let detected = ctx
        .data()
        .riot_client
        .detect_region(&lol_account.puuid)
        .await?;
    // baseline to the newest match on signup instead of backfilling full match
    // data for everyone's existing history, to avoid a burst of api calls
    let (lol_region, last_seen_lol_match_id) = match detected {
        Some((region, match_ids)) => (Some(region), match_ids.into_iter().next()),
        None => (None, None),
    };

    let mut db = ctx.data().db.lock().await;

    db.add_account(db::DatabaseAccount {
        discord_user_id: ctx.author().id.get(),
        val_puuid: valorant_account.puuid,
        lol_puuid: lol_account.puuid,
        lol_name: Some(riot_name),
        lol_tag: Some(riot_tag),
        lol_region,
        last_seen_lol_match_id,
        added_at: chrono::Utc::now(),
        ..Default::default()
    })?;

    ctx.say(format!("**{}** just signed up!", ctx.author().name))
        .await?;
    Ok(())
}
