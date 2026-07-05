use crate::types::{Context, Error};
use std::sync::atomic::Ordering;

#[poise::command(slash_command)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().polling_paused.store(false, Ordering::Relaxed);
    ctx.say("Polling resumed.").await?;
    Ok(())
}
