use std::borrow::Cow;

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SasError {
    #[error("Could not connect to Steam API")]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },
    #[error("Steam API Error: {msg}")]
    SteamApiError { msg: Cow<'static, str>, status: StatusCode },
    #[error("Deserialization Error: {source}")]
    SerdeError {
        #[from]
        source: serde_json::error::Error,
    },
    #[error("HTTP Error: {0}")]
    ApiError(String),
    #[error("Internal Error: {0}")]
    InternalError(Cow<'static, str>),
}
