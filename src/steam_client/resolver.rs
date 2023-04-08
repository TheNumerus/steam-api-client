use reqwest::{Client, StatusCode};

use serde_json::{self, Value};

use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{info, info_span, Instrument};

use super::SteamEndpoint;
use crate::{cache::Cache, entity::Player, error::SasError};

pub struct CachingVanityUrlResolver {
    cache: RwLock<Cache<String, Option<String>>>,
    api_key: String,
}

impl CachingVanityUrlResolver {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(Cache::new()),
            api_key: std::env::var("STEAM_API_KEY").unwrap_or_else(|_| "".to_owned()),
        }
    }

    #[tracing::instrument(skip(self, client))]
    pub async fn resolve(&self, id: &str, client: &Client) -> Result<String, SasError> {
        let e = SasError::SteamApiError {
            msg: "Player with given ID does not exist".into(),
            status: StatusCode::NOT_FOUND,
        };

        {
            let mut guard = self.cache.write().await;

            let item = guard.get(id);
            if let Some(inner) = item {
                info!("id {:?} resolved from cache", inner);
                return inner.clone().ok_or(e);
            }
        }

        let resolved = self.find(id, client).await?;

        {
            let mut guard = self.cache.write().await;
            guard.set(id.to_owned(), resolved.clone());
        }

        info!("id {:?} resolved from API", resolved);
        resolved.ok_or(e)
    }

    async fn find(&self, id: &str, client: &Client) -> Result<Option<String>, SasError> {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        #[allow(dead_code)]
        enum GetVanityUrlResponse {
            Ok { steamid: String, success: i32 },
            NotFound { message: String, success: i32 },
        }

        let url = SteamEndpoint::ResolveVanityUrl { url: id }.url(&self.api_key);

        let res = client.get(url).send().instrument(info_span!("API request")).await?;

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

        let url = SteamEndpoint::GetPlayerSummaries { steam_id: id }.url(&self.api_key);

        let res = client.get(url).send().instrument(info_span!("API request")).await?;

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
}
