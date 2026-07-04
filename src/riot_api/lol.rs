use serde::Deserialize;

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

    pub async fn get_account(
        &self,
        name: &str,
        tag: &str,
        region: &str,
    ) -> Result<AccountData, reqwest::Error> {
        let url = format!(
            "https://{}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
            region, name, tag
        );

        self.http
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await?
            .json::<AccountData>()
            .await
    }

    pub async fn get_matches(
        &self,
        puuid: &str,
        region: &str,
    ) -> Result<Vec<MatchSummary>, reqwest::Error> {
        let ids_url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids",
            region, puuid
        );

        let match_ids = self
            .http
            .get(ids_url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await?
            .json::<Vec<String>>()
            .await?;

        let mut matches = Vec::with_capacity(match_ids.len());
        for match_id in match_ids {
            let match_url = format!(
                "https://{}.api.riotgames.com/lol/match/v5/matches/{}",
                region, match_id
            );

            let summary = self
                .http
                .get(match_url)
                .header("X-Riot-Token", &self.api_key)
                .send()
                .await?
                .json::<MatchSummary>()
                .await?;

            matches.push(summary);
        }

        Ok(matches)
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
}
