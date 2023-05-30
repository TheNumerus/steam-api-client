use std::fmt::Debug;

use bytes::Bytes;
use reqwest::StatusCode;
use serde::{Deserialize};
use serde_json::{self, Value};

use tracing::{error, info_span};
use tracing_futures::Instrument;

use crate::{
    AppId,
    entity::{
        Game, Player, RecentGame, AchievementPlayerStatsSchema, PlayerStatsSchema,
        GameSchema, AchievementPercentageSchema
    },
    error::SasError
};

mod endpoint;
use endpoint::SteamEndpoint;

use self::endpoint::SteamImageEndpoint;

/// Client for interaction with Steam API
pub struct SteamClient {
    api_key: String,
    client: reqwest::Client,
}

impl SteamClient {
    /// Creates new [SteamClient] with api key from `STEAM_API_KEY` environment variable.
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("STEAM_API_KEY").unwrap_or_else(|_| "".to_owned()),
            client: reqwest::Client::new(),
        }
    }

    /// Creates new [SteamClient] with provided api key.
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Sets provided api key.
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    /// Returns list of owned games for given player id.
    ///
    /// Can include free games, and game info.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::SteamClient;
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_owned_games("playerId", true, true).await {
    ///     Ok(games) => {/*List of games*/},
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_owned_games(
        &self,
        id: &str,
        include_appinfo: bool,
        include_free_games: bool,
    ) -> Result<Vec<Game>, SasError> {
        let url = SteamEndpoint::GetOwnedGames {
            id,
            include_appinfo,
            include_free_games,
        }
        .url(&self.api_key);

        let res = self
            .client
            .get(url)
            .send()
            .instrument(info_span!("API request"))
            .await?;

        let status = res.status();
        if !status.is_success() {
            error!(status = status.as_u16());
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().instrument(info_span!("reading from JSON")).await?;

        let games: Vec<Game> = match serde_json::from_value(res["response"]["games"].clone()) {
            Ok(v) => v,
            Err(e) => {
                error!(error = ?e);
                return Err(SasError::InternalError("Invalid data format".into()));
            }
        };

        Ok(games)
    }

    /// Returns list of achievements for given
    ///
    /// Can include localized achievement name and description.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::{AppId, SteamClient};
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_achievements_for_game("playerId", AppId(400), Some("en")).await {
    ///     Ok(Some(games)) => {/*List of achievements*/}
    ///     Ok(None) => {/*Game has no achievements*/}
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_achievements_for_game(
        &self,
        id: &str,
        appid: AppId,
        lang: Option<&str>,
    ) -> Result<Option<Vec<AchievementPlayerStatsSchema>>, SasError> {
        let url = SteamEndpoint::GetPlayerAchievements { id, appid, lang }.url(&self.api_key);

        let res = self
            .client
            .get(url)
            .send()
            .instrument(info_span!("API request"))
            .await?;

        let status = res.status();
        if status.as_u16() == 403 {
            // private profile
            return Err(SasError::SteamApiError {
                msg: "Cannot get stats for private profile".into(),
                status: StatusCode::FORBIDDEN,
            });
        }
        if status.is_server_error() {
            error!(status = status.as_u16());
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().instrument(info_span!("reading from JSON")).await?;

        match res["playerstats"].as_object() {
            Some(playerstats) => {
                let stats: PlayerStatsSchema = match serde_json::from_value(Value::Object(playerstats.clone())) {
                    Ok(stats) => stats,
                    Err(e) => {
                        error!(error = ?e);
                        return Err(SasError::InternalError("Invalid data format".into()));
                    }
                };

                match stats {
                    PlayerStatsSchema::Success { achievements, .. } => Ok(Some(achievements)),
                    PlayerStatsSchema::Error { error } => Err(SasError::SteamApiError {
                        msg: error.into(),
                        status: StatusCode::BAD_GATEWAY,
                    }),
                }
            }
            None => Err(SasError::InternalError("Invalid data format".into())),
        }
    }

    /// Returns info about the player with given id.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::SteamClient;
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_player_info("playerId").await {
    ///     Ok(Some(player)) => {/*Player found*/},
    ///     Ok(None) => {/*No player found*/},
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_player_info(&self, id: &str) -> Result<Option<Player>, SasError> {
        let url = SteamEndpoint::GetPlayerSummaries { steam_id: id }.url(&self.api_key);

        let res = self
            .client
            .get(url)
            .send()
            .instrument(info_span!("API request"))
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().instrument(info_span!("reading from JSON")).await?;

        let players: Vec<Player> = match serde_json::from_value(res["response"]["players"].clone()) {
            Ok(v) => v,
            Err(e) => {
                error!(error = ?e);
                return Err(SasError::InternalError("Invalid data format".into()));
            }
        };

        Ok(players.get(0).cloned())
    }

    /// Returns achievement rarities for given app.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::{AppId, SteamClient};
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_achievement_rarity(AppId(400)).await {
    ///     Ok(Some(id)) => {/*App has achievements*/}
    ///     Ok(None) => {/*App has no achievements*/}
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_achievement_rarity(
        &self,
        appid: AppId,
    ) -> Result<Option<Vec<AchievementPercentageSchema>>, SasError> {
        let url = SteamEndpoint::GetGlobalAchievementPercentagesForApp { appid }.url(&self.api_key);

        let res = self
            .client
            .get(url)
            .send()
            .instrument(info_span!("API request"))
            .await?;

        let status = res.status();
        // no game or achievements
        if status.as_u16() == 403 {
            return Ok(None);
        }

        if !status.is_success() {
            error!(status = status.as_u16());
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().instrument(info_span!("reading from JSON")).await?;

        let achievements: Vec<AchievementPercentageSchema> =
            match serde_json::from_value(res["achievementpercentages"]["achievements"].clone()) {
                Ok(v) => v,
                Err(e) => {
                    error!(error = ?e);
                    return Err(SasError::InternalError("Invalid data format".into()));
                }
            };

        Ok(Some(achievements))
    }

    /// Resolves vanity url.
    ///
    /// Vanity is the custom url player may set on their profile. This function will resolve that url to profile ID.
    /// If profile id is inputted instead, this function will validate that and return it.
    /// # Examples
    /// ```rust
    /// use steam_api_client::SteamClient;
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.resolve_vanity_url("exampleVanityUrl").await {
    ///     Ok(Some(id)) => {/*Id found or validated*/}
    ///     Ok(None) => {/*No id found*/}
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn resolve_vanity_url(&self, vanity: &str) -> Result<Option<String>, SasError> {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        #[allow(dead_code)]
        enum GetVanityUrlResponse {
            Ok { steamid: String, success: i32 },
            NotFound { message: String, success: i32 },
        }

        let url = SteamEndpoint::ResolveVanityUrl { url: vanity }.url(&self.api_key);

        let res = self.client.get(url).send().instrument(info_span!("API request")).await?;

        let status = res.status();
        if !status.is_success() {
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().await?;

        let vanity_res = serde_json::from_value(res["response"].clone())?;

        if let GetVanityUrlResponse::Ok { steamid, .. } = vanity_res {
            return Ok(Some(steamid));
        }

        // now try access id directly

        let url = SteamEndpoint::GetPlayerSummaries { steam_id: vanity }.url(&self.api_key);

        let res = self.client.get(url).send().instrument(info_span!("API request")).await?;

        let status = res.status();
        if !status.is_success() {
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().await?;

        let players: Vec<Player> = serde_json::from_value(res["response"]["players"].clone())?;

        let player = players.get(0);
        match player {
            Some(p) => Ok(Some(p.steamid.clone())),
            None => Ok(None),
        }
    }

    /// Returns game schema, with stats and achievements.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::{AppId, SteamClient};
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_schema_for_game(AppId(400)).await {
    ///     Ok(Some(schema)) => {/*Schema found*/}
    ///     Ok(None) => {/*No schema for given app*/}
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_schema_for_game(&self, appid: AppId) -> Result<Option<GameSchema>, SasError> {
        let url = SteamEndpoint::GetSchemaForGame { appid }.url(&self.api_key);

        let res = self
            .client
            .get(url)
            .send()
            .instrument(info_span!("API request"))
            .await?;

        let status = res.status();
        if !status.is_success() {
            error!(status = status.as_u16());
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().instrument(info_span!("reading from JSON")).await?;

        match res["game"].as_object() {
            Some(obj) if obj.is_empty() => Ok(None),
            Some(obj) => {
                let schema: GameSchema = match serde_json::from_value(Value::Object(obj.clone())) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(error = ?e);
                        return Err(SasError::InternalError("Invalid data format".into()));
                    }
                };

                Ok(Some(schema))
            }
            None => Err(SasError::ApiError("Unexpected response from Steam API".into())),
        }
    }

    /// Returns list of recently played games by given player id.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::{AppId, SteamClient};
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_recent_games("playerId").await {
    ///     Ok(games) => {/*List of games*/},
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_recent_games(&self, id: &str) -> Result<Vec<RecentGame>, SasError> {
        let url = SteamEndpoint::GetRecentlyPlayedGames{ steam_id:id }.url(&self.api_key);

        let res = self
            .client
            .get(url)
            .send()
            .instrument(info_span!("API request"))
            .await?;

        let status = res.status();
        if !status.is_success() {
            error!(status = status.as_u16());
            return Err(SasError::ApiError(format!("Recieved {} from Steam API", status)));
        }

        let res = res.json::<Value>().instrument(info_span!("reading from JSON")).await?;

        let games: Vec<RecentGame> = match serde_json::from_value(res["response"]["games"].clone()) {
            Ok(v) => v,
            Err(e) => {
                error!(error = ?e);
                return Err(SasError::InternalError("Invalid data format".into()));
            }
        };

        Ok(games)
    }

    /// Validates api key given to the client.
    ///
    /// This function tries to get app schema for Portal, which should be stable.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::SteamClient;
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.validate_api_key().await {
    ///     Ok(_) => {/*Api key is valid*/},
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn validate_api_key(&self) -> Result<(), SasError> {
        // appid 400 is portal
        self.get_schema_for_game(AppId(400)).await.map(|_e| ())
    }

    /// Returns profile picture data for given player.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::{AppId, SteamClient};
    /// # use steam_api_client::entity::Player;
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// # let player = Player {
    /// # personaname: String::new(),
    /// # steamid: String::new(),
    /// # avatarfull: String::new(),
    /// # profileurl: String::new(),
    /// # };
    /// match client.get_profile_pic(&player).await {
    ///     Ok(profile_pic) => {/*Profile pic*/}
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_profile_pic(&self, player: &Player) -> Result<Bytes, SasError> {
        let res = self.client.get(&player.avatarfull).send().instrument(info_span!("Image request")).await?;

        Ok(res.bytes().await?)
    }

    /// Returns small image capsule for given app.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::{AppId, SteamClient};
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_game_small_capsule(AppId(400)).await {
    ///     Ok(pic) => {/*Capsule pic*/},
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_game_small_capsule(&self, appid: AppId) -> Result<Bytes, SasError> {
        let url = SteamImageEndpoint::SmallCapsule { appid }.url();

        let res = self.client.get(&url).send().instrument(info_span!("Image request")).await?;

        Ok(res.bytes().await?)
    }

    /// Returns library image capsule for given app.
    ///
    /// # Examples
    /// ```rust
    /// use steam_api_client::{AppId, SteamClient};
    ///
    /// # tokio_test::block_on(async {
    /// let client = SteamClient::new();
    /// match client.get_game_library_capsule(AppId(400)).await {
    ///     Ok(pic) => {/*Capsule pic*/},
    ///     Err(e) => {/*Error*/}
    /// }
    /// # });
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_game_library_capsule(&self, appid: AppId) -> Result<Option<Bytes>, SasError> {
        let url = SteamImageEndpoint::LibraryCapsule { appid }.url();

        let res = self.client.get(&url).send().instrument(info_span!("Image request")).await?;

        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(res.bytes().await?))
    }
}

impl Debug for SteamClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SteamClient").finish()
    }
}
