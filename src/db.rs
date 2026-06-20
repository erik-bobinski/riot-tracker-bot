/* db for discord user's riot accounts will be accounts.json storing something like:
  [
    {
      "discord_user_id": "123456789",
      "riot_id": "SummonerName#NA1",
      "puuid": null,
      "added_at": "2026-06-14T12:00:00Z"
    }
  ]
*/
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseAccount {
    pub discord_user_id: u64,
    pub valorant_name: String,
    pub valorant_tag: String,
    pub valorant_region: String,
    pub valorant_platform: String,
    pub last_seen_valorant_match_id: Option<Uuid>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum DbError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    Duplicate { puuid: String },
    NotFound { puuid: String },
}
impl From<std::io::Error> for DbError {
    fn from(err: std::io::Error) -> Self {
        DbError::Io(err)
    }
}
impl From<serde_json::Error> for DbError {
    fn from(err: serde_json::Error) -> Self {
        DbError::Serde(err)
    }
}

pub struct Database {
    path: PathBuf,
    accounts: Vec<DatabaseAccount>,
}
impl Database {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, DbError> {
        let path = path.into();

        let accounts = if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents)?
        } else {
            Vec::new()
        };

        Ok(Self { path, accounts })
    }

    // rewrite all contents into temp file first in case of runtime errors during file
    pub fn save(&self) -> Result<(), DbError> {
        let contents = serde_json::to_string_pretty(&self.accounts)?;
        let tmp_path = &self.path.with_extension("json.tmp");

        fs::write(&tmp_path, contents)?;
        fs::rename(tmp_path, &self.path)?;

        Ok(())
    }
}
