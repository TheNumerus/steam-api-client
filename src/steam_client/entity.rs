use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameSchema {
    #[serde(alias = "gameName")]
    pub game_name: String,
    #[serde(alias = "gameVersion")]
    pub game_version: String,
    #[serde(alias = "availableGameStats")]
    pub stats: GameStats,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameStats {
    #[serde(default)]
    pub achievements: Vec<AchievementSchema>,
    // contains stats, don't care about those for now
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AchievementSchema {
    pub name: String,
    #[serde(alias = "displayName")]
    pub display_name: String,
    #[serde(deserialize_with = "num_to_bool")]
    pub hidden: bool,
    pub description: Option<String>,
    pub icon: String,
    pub icongray: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AchievementPercentageSchema {
    pub name: String,
    pub percent: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PlayerStatsSchema {
    Success {
        #[serde(alias = "steamID")]
        steam_id: String,
        #[serde(alias = "gameName")]
        game_name: String,
        achievements: Vec<AchievementPlayerStatsSchema>,
    },
    Error {
        error: String,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AchievementPlayerStatsSchema {
    pub apiname: String,
    pub achieved: u8,
    pub unlocktime: i64,
}

fn num_to_bool<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|e: i32| e == 1)
}
