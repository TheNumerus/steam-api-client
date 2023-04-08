use std::fmt::Debug;

use bytes::Bytes;
use reqwest::StatusCode;
use serde_json::{self, Value};

use tracing::{error, info_span};
use tracing_futures::Instrument;

use crate::{
    entity::{Game, Player},
    error::SasError,
};

mod resolver;
use resolver::CachingVanityUrlResolver;

mod endpoint;
use endpoint::SteamEndpoint;

pub mod entity;
use entity::*;

use self::endpoint::SteamImageEndpoint;

pub struct SteamClient {
    api_key: String,
    client: reqwest::Client,
    vanity_resolver: CachingVanityUrlResolver,
}

impl SteamClient {
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("STEAM_API_KEY").unwrap_or_else(|_| "".to_owned()),
            client: reqwest::Client::new(),
            vanity_resolver: CachingVanityUrlResolver::new(),
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            vanity_resolver: CachingVanityUrlResolver::new(),
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_owned_games(
        &self,
        id: &str,
        include_appinfo: bool,
        include_free_games: bool,
    ) -> Result<Vec<Game>, SasError> {
        let id = self.vanity_resolver.resolve(id, &self.client).await?;

        let url = SteamEndpoint::GetOwnedGames {
            id: &id,
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

    #[tracing::instrument(skip(self))]
    pub async fn get_achievements_for_game(
        &self,
        id: &str,
        appid: &str,
        lang: Option<&str>,
    ) -> Result<Option<Vec<AchievementPlayerStatsSchema>>, SasError> {
        let id = self.vanity_resolver.resolve(id, &self.client).await?;

        let url = SteamEndpoint::GetPlayerAchievements { id: &id, appid, lang }.url(&self.api_key);

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

    #[tracing::instrument(skip(self))]
    pub async fn get_player_info(&self, id: &str) -> Result<Option<Player>, SasError> {
        let id = self.vanity_resolver.resolve(id, &self.client).await?;

        let url = SteamEndpoint::GetPlayerSummaries { steam_id: &id }.url(&self.api_key);

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

    #[tracing::instrument(skip(self))]
    pub async fn get_achievement_rarity(
        &self,
        appid: &str,
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

    #[tracing::instrument(skip(self))]
    pub async fn get_schema_for_game(&self, appid: &str) -> Result<Option<GameSchema>, SasError> {
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

    #[tracing::instrument(skip(self))]
    pub async fn validate_api_key(&self) -> Result<(), SasError> {
        // appid 400 is portal
        self.get_schema_for_game("400").await.map(|_e| ())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_profile_pic(&self, player: &Player) -> Result<Bytes, SasError> {
        let res = self.client.get(&player.avatarfull).send().instrument(info_span!("Image request")).await?;

        Ok(res.bytes().await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_game_small_capsule(&self, appid: &str) -> Result<Bytes, SasError> {
        let url = SteamImageEndpoint::SmallCapsule { appid }.url();

        let res = self.client.get(&url).send().instrument(info_span!("Image request")).await?;

        Ok(res.bytes().await?)
    }
}

impl Debug for SteamClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SteamClient").finish()
    }
}
