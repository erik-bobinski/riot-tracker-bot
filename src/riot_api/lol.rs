use serde::Deserialize;

// Match-V5 continental routing values. A player's account lives behind
// exactly one of these; there's no way to know which without asking or probing.
const MATCH_REGIONS: [&str; 4] = ["americas", "asia", "europe", "sea"];

pub struct RiotClient {
    http: reqwest::Client,
    api_key: String,
}
impl RiotClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
        }
    }

    // Account-V1 can be queried from any regional cluster for any account,
    // so we always route through the cluster nearest our deployment (Virginia).
    pub async fn get_account(&self, name: &str, tag: &str) -> Result<AccountData, reqwest::Error> {
        let url = format!(
            "https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
            name, tag
        );

        self.http
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await?
            .json::<AccountData>()
            .await
    }

    pub async fn get_match_ids(
        &self,
        puuid: &str,
        region: &str,
    ) -> Result<Vec<String>, reqwest::Error> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids",
            region, puuid
        );

        self.http
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await?
            .json::<Vec<String>>()
            .await
    }

    pub async fn get_match(
        &self,
        match_id: &str,
        region: &str,
    ) -> Result<MatchSummary, reqwest::Error> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/{}",
            region, match_id
        );

        self.http
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await?
            .json::<MatchSummary>()
            .await
    }

    pub async fn get_matches(
        &self,
        puuid: &str,
        region: &str,
    ) -> Result<Vec<MatchSummary>, reqwest::Error> {
        let match_ids = self.get_match_ids(puuid, region).await?;

        let mut matches = Vec::with_capacity(match_ids.len());
        for match_id in match_ids {
            matches.push(self.get_match(&match_id, region).await?);
        }

        Ok(matches)
    }

    // Unlike account-v1, match-v5 results only come back non-empty from the
    // continental cluster the account's platform actually belongs to, so we
    // have to probe each one. Returns None if the account has no match history
    // in any cluster yet (nothing to detect from).
    pub async fn detect_region(&self, puuid: &str) -> Result<Option<String>, reqwest::Error> {
        for region in MATCH_REGIONS {
            let match_ids = self.get_match_ids(puuid, region).await?;
            if !match_ids.is_empty() {
                return Ok(Some(region.to_string()));
            }
        }

        Ok(None)
    }
}

// response from /riot/account/v1/accounts/by-riot-id/{name}/{tag}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

// response from /lol/match/v5/matches/{matchId}
#[derive(Debug, Deserialize)]
pub struct MatchSummary {
    pub metadata: MatchMetadata,
    pub info: MatchInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchMetadata {
    pub match_id: String,
    pub participants: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchInfo {
    pub game_mode: String,
    pub game_duration: u64,
    pub platform_id: String,
    pub participants: Vec<MatchParticipant>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchParticipant {
    pub puuid: String,
    pub riot_id_game_name: String,
    pub riot_id_tagline: String,
    pub team_id: u32,
    pub champion_name: String,
    pub kills: u32,
    pub deaths: u32,
}
