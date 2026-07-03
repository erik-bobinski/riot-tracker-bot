use serde::Deserialize;

pub struct HenrikClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}
impl HenrikClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: String::from("https://api.henrikdev.xyz"),
            api_key,
        }
    }

    pub async fn get_account(&self, name: &str, tag: &str) -> Result<AccountData, reqwest::Error> {
        let url = format!("{}/valorant/v2/account/{}/{}", self.base_url, name, tag);

        Ok(self
            .http
            .get(url)
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json::<HenrikResponse<AccountData>>()
            .await?
            .data)
    }

    pub async fn get_matches(
        &self,
        name: &str,
        tag: &str,
        region: &str,
    ) -> Result<Vec<MatchSummary>, reqwest::Error> {
        let url = format!(
            "{}/valorant/v3/matches/{}/{}/{}",
            self.base_url, region, name, tag
        );

        Ok(self
            .http
            .get(url)
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json::<HenrikResponse<Vec<MatchSummary>>>()
            .await?
            .data)
    }
}

// general henrik response shape
#[derive(Debug, Deserialize)]
pub struct HenrikResponse<T> {
    pub status: u16,
    pub data: T,
}

// response from /valorant/v2/account/{name}/{tag}
#[derive(Debug, Deserialize)]
pub struct AccountData {
    pub puuid: String,
}

//response from /valorant/v3/matches/{region}/{name}/{tag}
#[derive(Debug, Deserialize)]
pub struct MatchSummary {
    metadata: MatchMetadata,
    players: MatchPlayers,
}

#[derive(Debug, Deserialize)]
pub struct MatchMetadata {
    map: String,
    mode: String,
    game_length: u64,
    region: String,
    matchid: String,
}

#[derive(Debug, Deserialize)]
pub struct MatchPlayers {
    all_players: Vec<MatchPlayer>,
}

#[derive(Debug, Deserialize)]
pub struct MatchPlayer {
    puuid: String,
    name: String,
    tag: String,
    team: String,
    character: String,
    currenttier_patched: String,
    player_card: String,
    stats: PlayerStats,
}

#[derive(Debug, Deserialize)]
pub struct PlayerStats {
    kills: u32,
}
