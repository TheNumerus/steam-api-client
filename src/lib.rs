#![warn(missing_docs)]

//!
//! Library for communicating with Steam Web APIs.
//!

/// Entity specifications.
pub mod entity;
mod error;
mod steam_client;

use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
pub use steam_client::SteamClient;
pub use error::SasError;

/// Newtype for app ids
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct AppId(pub usize);

impl Display for AppId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}