// Reference example — not wired into the crate.
// Add to Cargo.toml:
//   serde = { version = "1", features = ["derive"] }
//   serde_json = "1"
//   chrono = { version = "0.4", features = ["serde"] }

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One tracked Valorant account linked to a Discord user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiotAccount {
    pub discord_user_id: u64,
    pub puuid: String,
    pub name: String,
    pub tag: String,
    pub region: String,
    pub platform: String, // usually "pc"
    pub added_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum DbError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    Duplicate { puuid: String },
    NotFound { puuid: String },
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Io(e) => write!(f, "io error: {e}"),
            DbError::Serde(e) => write!(f, "json error: {e}"),
            DbError::Duplicate { puuid } => write!(f, "account already tracked: {puuid}"),
            DbError::NotFound { puuid } => write!(f, "account not found: {puuid}"),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::Io(e) => Some(e),
            DbError::Serde(e) => Some(e),
            _ => None,
        }
    }
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

/// In-memory store backed by a JSON file on disk.
pub struct Database {
    path: PathBuf,
    accounts: Vec<RiotAccount>,
}

impl Database {
    /// Load from disk, or start empty if the file doesn't exist yet.
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

    /// Write current state to disk (atomic: tmp file + rename).
    pub fn save(&self) -> Result<(), DbError> {
        let contents = serde_json::to_string_pretty(&self.accounts)?;
        let tmp_path = self.path.with_extension("json.tmp");

        fs::write(&tmp_path, contents)?;
        fs::rename(tmp_path, &self.path)?;

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn all_accounts(&self) -> &[RiotAccount] {
        &self.accounts
    }

    pub fn accounts_for_user(&self, discord_user_id: u64) -> Vec<&RiotAccount> {
        self.accounts
            .iter()
            .filter(|a| a.discord_user_id == discord_user_id)
            .collect()
    }

    pub fn find_by_puuid(&self, puuid: &str) -> Option<&RiotAccount> {
        self.accounts.iter().find(|a| a.puuid == puuid)
    }

    /// Add a new account. Errors if that puuid is already tracked.
    pub fn add_account(&mut self, account: RiotAccount) -> Result<(), DbError> {
        if self.find_by_puuid(&account.puuid).is_some() {
            return Err(DbError::Duplicate {
                puuid: account.puuid,
            });
        }

        self.accounts.push(account);
        Ok(())
    }

    /// Remove an account only if it belongs to the given Discord user.
    pub fn remove_account(&mut self, discord_user_id: u64, puuid: &str) -> Result<(), DbError> {
        let index = self
            .accounts
            .iter()
            .position(|a| a.discord_user_id == discord_user_id && a.puuid == puuid)
            .ok_or_else(|| DbError::NotFound {
                puuid: puuid.to_string(),
            })?;

        self.accounts.remove(index);
        Ok(())
    }
}
