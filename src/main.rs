use crate::db::Database;
use crate::types::Data;
use serenity::prelude::*;
use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

mod commands;
mod db;
mod discord;
mod polling;
mod riot_api;
mod types;
mod web;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        dotenv::dotenv().ok();
        env::var("DISCORD_TOKEN").expect("Expected a discord bot env var in environment")
    });

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    tokio::spawn(web::serve(port));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let henrik_api_key =
                    env::var("HENRIK_API_KEY").expect("Expected HENRIK_API_KEY in environment");
                let riot_api_key =
                    env::var("RIOT_API_KEY").expect("Expected RIOT_API_KEY in environment");
                let db_path = env::var("DB_PATH").unwrap_or_else(|_| "accounts.json".to_string());
                let notification_channel_id = env::var("NOTIFICATION_CHANNEL_ID")
                    .expect("Expected NOTIFICATION_CHANNEL_ID in environment")
                    .parse::<u64>()
                    .expect("NOTIFICATION_CHANNEL_ID must be a valid channel id");
                let notification_channel_id = serenity::model::id::ChannelId::new(notification_channel_id);

                let db = Database::load(&db_path)?;
                let db = Arc::new(Mutex::new(db));
                let henrik_client = Arc::new(riot_api::valorant::HenrikClient::new(henrik_api_key));
                let riot_client = Arc::new(riot_api::lol::RiotClient::new(riot_api_key));
                let polling_paused = Arc::new(AtomicBool::new(false));

                tokio::spawn(polling::run(
                    ctx.http.clone(),
                    db.clone(),
                    henrik_client.clone(),
                    riot_client.clone(),
                    notification_channel_id,
                    polling_paused.clone(),
                ));

                Ok(Data {
                    henrik_client,
                    riot_client,
                    db,
                    polling_paused,
                })
            })
        })
        .build();

    let client = Client::builder(&token, intents).framework(framework).await;
    client.unwrap().start().await.unwrap();
}
