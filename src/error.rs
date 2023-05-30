use std::borrow::Cow;

use reqwest::StatusCode;
use thiserror::Error;

/// Error type for this library.
#[derive(Debug, Error)]
pub enum SasError {
    /// Network error
    #[error("Could not connect to Steam API")]
    ReqwestError {
        /// Source of the network error
        #[from]
        source: reqwest::Error,
    },
    /// Error returned from Steam API
    #[error("Steam API Error: {msg}")]
    SteamApiError {
        /// Error message
        msg: Cow<'static, str>,
        /// Error code
        status: StatusCode
    },
    /// Format/structure error
    #[error("Deserialization Error: {source}")]
    SerdeError {
        /// Source of the error
        #[from]
        source: serde_json::error::Error,
    },
    /// General HTTP error
    #[error("HTTP Error: {0}")]
    ApiError(String),
    /// General Error
    #[error("Internal Error: {0}")]
    InternalError(Cow<'static, str>),
}
