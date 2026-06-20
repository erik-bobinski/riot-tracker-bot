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

    //TODO:
    pub fn get_account(&self, name: &str, tag: &str) -> Result<AccountData, reqwest::Error> {
        todo!()
    }

    //TODO:
    pub fn get_matches(
        &self,
        name: &str,
        tag: &str,
        region: &str,
    ) -> Result<MatchData, reqwest::Error> {
        todo!()
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
    pub region: String,
    pub account_level: u64,
    pub name: String,
    pub tag: String,
    pub platforms: Vec<String>,
}

//TODO: response from /valorant/v3/matches/{region}/{name}/{tag}
#[derive(Debug, Deserialize)]
pub struct MatchData {}
