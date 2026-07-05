use crate::types::{Context, Error};
use std::sync::atomic::Ordering;

/// Pause all match reports
#[poise::command(slash_command)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().polling_paused.store(true, Ordering::Relaxed);
    ctx.say("Polling paused.").await?;
    Ok(())
}
