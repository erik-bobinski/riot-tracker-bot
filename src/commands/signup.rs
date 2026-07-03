use crate::db;
use crate::types::{Context, Error};

#[poise::command(slash_command)]
pub async fn signup(
    ctx: Context<'_>,
    #[description = "Riot Name"] riot_name: String,
    #[description = "Riot Tag"] riot_tag: String,
) -> Result<(), Error> {
    let riot_account = match ctx
        .data()
        .henrik_client
        .get_account(&riot_name, &riot_tag)
        .await
    {
        Ok(riot_account) => riot_account,
        Err(_) => {
            ctx.say("Couldn't find riot account :(").await?;
            return Ok(());
        }
    };

    let mut db = ctx.data().db.lock().await;

    db.add_account(db::DatabaseAccount {
        discord_user_id: ctx.author().id.get(),
        puuid: riot_account.puuid,
        added_at: chrono::Utc::now(),
        ..Default::default()
    })?;

    ctx.say(format!("**{}** just signed up!", ctx.author().name))
        .await?;
    Ok(())
}
