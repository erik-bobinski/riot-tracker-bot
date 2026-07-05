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
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DatabaseAccount {
    pub discord_user_id: u64,
    pub discord_name: String,
    pub riot_name: String,
    pub riot_tag: String,
    pub val_puuid: String,
    pub val_region: Option<String>,
    pub last_seen_val_match_id: Option<Uuid>,
    pub lol_puuid: String,
    pub lol_region: Option<String>,
    pub last_seen_lol_match_id: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum DbError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    DuplicatePuuid { puuid: String },
    DuplicateDiscordUserId { discord_user_id: u64 },
    NotFound { discord_user_id: u64 },
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
impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Io(e) => write!(f, "io error: {e}"),
            DbError::Serde(e) => write!(f, "json error: {e}"),
            DbError::DuplicateDiscordUserId { discord_user_id } => {
                write!(f, "discord account already tracked: {discord_user_id}")
            }
            DbError::DuplicatePuuid { puuid } => write!(f, "riot account already tracked: {puuid}"),
            DbError::NotFound { discord_user_id } => {
                write!(f, "discord account not found: {discord_user_id}")
            }
        }
    }
}
impl std::error::Error for DbError {}

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

    // add new discord/riot account to db
    pub fn add_account(&mut self, account: DatabaseAccount) -> Result<(), DbError> {
        // no dupe discord users
        let already_tracked_puuid = self
            .accounts
            .iter()
            .any(|acct| acct.discord_user_id == account.discord_user_id);
        if already_tracked_puuid {
            return Err(DbError::DuplicateDiscordUserId {
                discord_user_id: account.discord_user_id,
            });
        }

        // no dupe valorant users (empty puuid means no valorant account, so never a dupe)
        let already_tracked_val_puuid = !account.val_puuid.is_empty()
            && self
                .accounts
                .iter()
                .any(|acct| acct.val_puuid == account.val_puuid);
        if already_tracked_val_puuid {
            return Err(DbError::DuplicatePuuid {
                puuid: account.val_puuid,
            });
        }

        // no dupe lol users (empty puuid means no lol account, so never a dupe)
        let already_tracked_lol_puuid = !account.lol_puuid.is_empty()
            && self
                .accounts
                .iter()
                .any(|acct| acct.lol_puuid == account.lol_puuid);
        if already_tracked_lol_puuid {
            return Err(DbError::DuplicatePuuid {
                puuid: account.lol_puuid,
            });
        }

        self.accounts.push(account);
        self.save()
    }

    // remove all disord user's data from db
    pub fn delete_account(&mut self, discord_user_id: u64) -> Result<(), DbError> {
        let index = self
            .accounts
            .iter()
            .position(|acct| acct.discord_user_id == discord_user_id);

        match index {
            Some(i) => {
                self.accounts.remove(i);
                self.save()
            }
            None => Err(DbError::NotFound { discord_user_id }),
        }
    }

    pub fn get_accounts(&self) -> Vec<DatabaseAccount> {
        self.accounts.clone()
    }

    // overwrite an existing account's data, matched by discord_user_id
    pub fn update_account(&mut self, account: DatabaseAccount) -> Result<(), DbError> {
        let index = self
            .accounts
            .iter()
            .position(|acct| acct.discord_user_id == account.discord_user_id);

        match index {
            Some(i) => {
                self.accounts[i] = account;
                self.save()
            }
            None => Err(DbError::NotFound {
                discord_user_id: account.discord_user_id,
            }),
        }
    }
}
