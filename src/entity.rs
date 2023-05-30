use serde::{Deserialize, Deserializer, Serialize};

mod game;
mod player;

pub use game::{Game, RecentGame};
pub use player::Player;

/// Game schema information
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameSchema {
    /// Game name
    #[serde(alias = "gameName")]
    pub game_name: String,
    /// Game version
    #[serde(alias = "gameVersion")]
    pub game_version: String,
    /// Game stats and achievement
    #[serde(alias = "availableGameStats")]
    pub stats: GameStats,
}

/// Game stats and achievements
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameStats {
    /// Game achievements
    #[serde(default)]
    pub achievements: Vec<AchievementSchema>,
    /// Game stats
    #[serde(default)]
    pub stats: Vec<StatSchema>,
}

/// Entity containing basic achievement information.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AchievementSchema {
    /// Achievement name used throughout API
    pub name: String,
    /// Localized achievement name
    #[serde(alias = "displayName")]
    pub display_name: String,
    /// Achievement set as hidden
    #[serde(deserialize_with = "num_to_bool")]
    pub hidden: bool,
    /// Optional localized description
    pub description: Option<String>,
    /// Achieved icon url
    pub icon: String,
    /// Missing icon url
    pub icongray: String,
}

/// Entity containing basic stat information.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StatSchema {
    /// Stat name used throughout API
    pub name: String,
    /// Default stat value
    #[serde(alias = "defaultvalue")]
    pub default_value: f64,
    /// Localized stat name
    #[serde(alias = "displayName")]
    pub display_name: Option<String>,
}

/// Entity containing achievement completion percentage information.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AchievementPercentageSchema {
    /// Achievement name used throughout API
    pub name: String,
    /// Completion percentage
    pub percent: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum PlayerStatsSchema {
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

/// Entity containing player info of achievement
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AchievementPlayerStatsSchema {
    /// Achievement name used throughout API
    #[serde(alias = "apiname")]
    pub api_name: String,
    /// Is achieved by player
    #[serde(deserialize_with = "num_to_bool")]
    pub achieved: bool,
    /// Achieved unlock timestamp
    #[serde(alias = "unlocktime")]
    pub unlock_time: i64,
}

fn num_to_bool<'de, D>(d: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|e: i32| e == 1)
}
