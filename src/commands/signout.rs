use crate::types::{Context, Error};

/// Stop tracking your riot account
#[poise::command(slash_command)]
pub async fn signout(ctx: Context<'_>) -> Result<(), Error> {
    let mut db = ctx.data().db.lock().await;

    db.delete_account(ctx.author().id.get())?;

    ctx.say(format!("**{}** just signed out!", ctx.author().name))
        .await?;
    Ok(())
}
