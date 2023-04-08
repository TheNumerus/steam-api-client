use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub steamid: String,
    pub profileurl: String,
    pub personaname: String,
    pub avatarfull: String,
}
