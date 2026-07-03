use std::env;

use serenity::prelude::*;

use poise::serenity_prelude as serenity_poise;

use crate::db::Database;

mod riot_api;

mod db;

// fundamental types for discord bot
struct Data {
    henrik_client: riot_api::valorant::HenrikClient,
    db: Mutex<Database>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn signup(
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

    ctx.say(format!("{} just signed up!", ctx.author().name))
        .await?;
    Ok(())
}

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
            commands: vec![signup()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let henrik_api_key =
                    env::var("HENRIK_API_KEY").expect("Expected HENRIK_API_KEY in environment");
                let db_path = env::var("DB_PATH").unwrap_or_else(|_| "accounts.json".to_string());
                let db = Database::load(&db_path)?;

                Ok(Data {
                    henrik_client: riot_api::valorant::HenrikClient::new(henrik_api_key),
                    db: Mutex::new(db),
                })
            })
        })
        .build();

    let client = Client::builder(&token, intents).framework(framework).await;
    client.unwrap().start().await.unwrap();
}
