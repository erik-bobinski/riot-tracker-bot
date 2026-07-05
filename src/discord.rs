use crate::riot_api::{lol, valorant};

// shared shape both games get adapted into, so the leaderboard
// formatting logic only has to be written once
pub struct PlayerLine {
    pub riot_name: String,
    pub riot_tag: String,
    pub character: String,
    pub kills: u32,
    pub deaths: u32,
    pub is_tracked_user: bool,
}

pub struct MatchResult {
    pub game_name: &'static str,
    pub game_mode: String,
    pub map: Option<String>,
    pub own_team: Vec<PlayerLine>,
    pub enemy_team: Vec<PlayerLine>,
}

pub fn val_match_to_result(
    match_summary: &valorant::MatchSummary,
    tracked_puuid: &str,
) -> MatchResult {
    let tracked_team = match_summary
        .players
        .all_players
        .iter()
        .find(|p| p.puuid == tracked_puuid)
        .map(|p| p.team.clone());

    let (own_team, enemy_team): (Vec<_>, Vec<_>) = match_summary
        .players
        .all_players
        .iter()
        .partition(|p| Some(&p.team) == tracked_team.as_ref());

    let to_line = |p: &valorant::MatchPlayer| PlayerLine {
        riot_name: p.name.clone(),
        riot_tag: p.tag.clone(),
        character: p.character.clone(),
        kills: p.stats.kills,
        deaths: p.stats.deaths,
        is_tracked_user: p.puuid == tracked_puuid,
    };

    MatchResult {
        game_name: "Valorant",
        game_mode: match_summary.metadata.mode.clone(),
        map: Some(match_summary.metadata.map.clone()),
        own_team: own_team.into_iter().map(to_line).collect(),
        enemy_team: enemy_team.into_iter().map(to_line).collect(),
    }
}

pub fn lol_match_to_result(match_summary: &lol::MatchSummary, tracked_puuid: &str) -> MatchResult {
    let tracked_team_id = match_summary
        .info
        .participants
        .iter()
        .find(|p| p.puuid == tracked_puuid)
        .map(|p| p.team_id);

    let (own_team, enemy_team): (Vec<_>, Vec<_>) = match_summary
        .info
        .participants
        .iter()
        .partition(|p| Some(p.team_id) == tracked_team_id);

    let to_line = |p: &lol::MatchParticipant| PlayerLine {
        riot_name: p.riot_id_game_name.clone(),
        riot_tag: p.riot_id_tagline.clone(),
        character: p.champion_name.clone(),
        kills: p.kills,
        deaths: p.deaths,
        is_tracked_user: p.puuid == tracked_puuid,
    };

    MatchResult {
        game_name: "League of Legends",
        game_mode: match_summary.info.game_mode.clone(),
        map: None,
        own_team: own_team.into_iter().map(to_line).collect(),
        enemy_team: enemy_team.into_iter().map(to_line).collect(),
    }
}

pub fn format_match_message(discord_name: &str, result: &MatchResult) -> String {
    let mut message = format!(
        "**{}** just finished a **{}** game\n",
        discord_name, result.game_name
    );

    match &result.map {
        Some(map) => message.push_str(&format!("{} / {}\n\n", result.game_mode, map)),
        None => message.push_str(&format!("{}\n\n", result.game_mode)),
    }

    message.push_str(&format_leaderboard(&result.own_team));
    message.push_str("\n\n");
    message.push_str(&format_leaderboard(&result.enemy_team));

    message
}

fn format_leaderboard(players: &[PlayerLine]) -> String {
    let mut sorted: Vec<&PlayerLine> = players.iter().collect();
    sorted.sort_by(|a, b| kd_ratio(b).partial_cmp(&kd_ratio(a)).unwrap());

    sorted
        .into_iter()
        .map(|p| {
            let name = format!("{}#{}", p.riot_name, p.riot_tag);
            let name = if p.is_tracked_user {
                format!("**{}**", name)
            } else {
                name
            };
            format!("{} ({}) {}/{}", name, p.character, p.kills, p.deaths)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn kd_ratio(p: &PlayerLine) -> f64 {
    p.kills as f64 / p.deaths.max(1) as f64
}
