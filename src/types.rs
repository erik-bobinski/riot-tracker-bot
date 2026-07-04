use crate::db::Database;
use crate::riot_api;
use serenity::prelude::Mutex;

// fundamental types for discord bot
pub struct Data {
    pub henrik_client: riot_api::valorant::HenrikClient,
    pub riot_client: riot_api::lol::RiotClient,
    pub db: Mutex<Database>,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
