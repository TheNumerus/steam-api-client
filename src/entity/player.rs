use serde::{Deserialize, Serialize};

/// Entity representing the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// ID of the user
    pub steamid: String,
    /// link to user's profile
    pub profileurl: String,
    /// Public username
    pub personaname: String,
    /// Url of profile picture
    pub avatarfull: String,
}
