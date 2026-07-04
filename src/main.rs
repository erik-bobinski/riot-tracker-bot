use crate::db::Database;
use crate::types::Data;
use serenity::prelude::*;
use std::env;

mod commands;
mod db;
mod riot_api;
mod types;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        dotenv::dotenv().ok();
        env::var("DISCORD_TOKEN").expect("Expected a discord bot env var in environment")
    });

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
                let db = Database::load(&db_path)?;

                Ok(Data {
                    henrik_client: riot_api::valorant::HenrikClient::new(henrik_api_key),
                    riot_client: riot_api::lol::RiotClient::new(riot_api_key),
                    db: Mutex::new(db),
                })
            })
        })
        .build();

    let client = Client::builder(&token, intents).framework(framework).await;
    client.unwrap().start().await.unwrap();
}
