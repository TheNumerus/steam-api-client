use serde::{Deserialize, Serialize};
use crate::AppId;

/// Entity representing game or application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// ID of the app
    pub appid: AppId,
    /// Optional name
    pub name: Option<String>,
    /// Optional icon url part
    pub img_icon_url: Option<String>,
    /// Total playtime in minutes
    pub playtime_forever: usize,
    /// Timestamp for most recent play session
    #[serde(alias = "rtime_last_played")]
    pub timestamp_last_played: u64,
}

/// Entity representing recently player game or application
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RecentGame {
    /// ID of the app
    pub appid: AppId,
    /// Optional name
    pub name: Option<String>,
    /// Optional icon url part
    pub img_icon_url: Option<String>,
    /// Total playtime in minutes
    pub playtime_forever: i32,
    /// Total playtime in last two weeks in minutes
    pub playtime_2weeks: i32,
}
