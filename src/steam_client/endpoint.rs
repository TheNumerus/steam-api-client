use crate::AppId;

const BASE_URL: &str = "https://api.steampowered.com";
const BASE_IMAGE_URL: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

pub enum SteamEndpoint<'a> {
    GetOwnedGames {
        id: &'a str,
        include_appinfo: bool,
        include_free_games: bool,
    },
    GetPlayerAchievements {
        id: &'a str,
        appid: AppId,
        lang: Option<&'a str>,
    },
    ResolveVanityUrl {
        url: &'a str,
    },
    GetPlayerSummaries {
        steam_id: &'a str,
    },
    GetGlobalAchievementPercentagesForApp {
        appid: AppId,
    },
    GetSchemaForGame {
        appid: AppId,
    },
    GetRecentlyPlayedGames {
        steam_id: &'a str,
    },
}

impl<'a> SteamEndpoint<'a> {
    pub fn url(self, key: &str) -> String {
        let resource = match &self {
            Self::GetOwnedGames { .. } => "/IPlayerService/GetOwnedGames/v0001/",
            Self::GetPlayerAchievements { .. } => "/ISteamUserStats/GetPlayerAchievements/v0001/",
            Self::ResolveVanityUrl { .. } => "/ISteamUser/ResolveVanityURL/v1/",
            Self::GetPlayerSummaries { .. } => "/ISteamUser/GetPlayerSummaries/v2/",
            Self::GetGlobalAchievementPercentagesForApp { .. } => {
                "/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v2/"
            }
            Self::GetSchemaForGame { .. } => "/ISteamUserStats/GetSchemaForGame/v0002/",
            Self::GetRecentlyPlayedGames { .. } => "/IPlayerService/GetRecentlyPlayedGames/v1/"
        };

        match self {
            Self::GetOwnedGames {
                id,
                include_appinfo,
                include_free_games,
            } => format!(
                "{}{}?key={}&steamid={}&include_appinfo={}&include_played_free_games={}",
                BASE_URL, resource, key, id, include_appinfo, include_free_games
            ),
            Self::GetPlayerAchievements { id, appid, lang } => match lang {
                Some(l) => format!(
                    "{}{}?key={}&steamid={}&appid={}&l={}",
                    BASE_URL, resource, key, id, appid, l
                ),
                None => format!("{}{}?key={}&steamid={}&appid={}", BASE_URL, resource, key, id, appid),
            },
            Self::ResolveVanityUrl { url } => {
                format!("{}{}?key={}&vanityurl={}", BASE_URL, resource, key, url)
            }
            Self::GetPlayerSummaries { steam_id } => {
                format!("{}{}?key={}&steamids={}", BASE_URL, resource, key, steam_id)
            }
            Self::GetGlobalAchievementPercentagesForApp { appid } => {
                format!("{}{}?key={}&gameid={}", BASE_URL, resource, key, appid)
            }
            Self::GetSchemaForGame { appid } => {
                format!("{}{}?key={}&appid={}", BASE_URL, resource, key, appid)
            }
            Self::GetRecentlyPlayedGames {steam_id} => {
                format!("{}{}?key={}&steamid={}", BASE_URL, resource, key, steam_id)
            }
        }
    }
}

pub enum SteamImageEndpoint {
    SmallCapsule {
        appid: AppId
    },
    LibraryCapsule {
        appid: AppId
    }
}

impl SteamImageEndpoint {
    pub fn url(self) -> String {
        match self {
            SteamImageEndpoint::SmallCapsule { appid } => format!("{}/{}/capsule_231x87.jpg", BASE_IMAGE_URL, appid),
            SteamImageEndpoint::LibraryCapsule { appid } => format!("{}/{}/library_600x900.jpg", BASE_IMAGE_URL, appid),
        }
    }
}
