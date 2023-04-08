use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub appid: u32,
    pub name: Option<String>,
    pub img_icon_url: Option<String>,
    pub img_logo_url: Option<String>,
    #[serde(default)]
    pub playtime_2weeks: i32,
    pub playtime_forever: i32,
}
