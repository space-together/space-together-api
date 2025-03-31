use std::collections::HashMap;

use crate::config::application_conf::AppConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Struct holding OAuth2 URLs
pub struct OAuthUrls {
    pub auth: String,
    pub token: String,
    pub user: String,
}

/// Represents an OAuth2 client
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub code_verifier: String,
    pub state: String,
    pub urls: OAuthUrls,
    pub user_info: UserInfo,
}

/// Holds user information parser
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub parser: fn(Value) -> Result<ParsedUser, String>,
}

/// Parsed user information from OAuth2 provider
#[derive(Debug, Deserialize)]
pub struct ParsedUser {
    pub email: String,
    pub name: String,
    pub image: Option<String>,
}

/// OAuth2 token response
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    #[serde(flatten)]
    pub extra: Option<HashMap<String, serde_json::Value>>, // Capture unexpected fields
}

/// Parses Discord user data from JSON response
fn discord_user_parser(data: Value) -> Result<ParsedUser, String> {
    #[derive(Deserialize)]
    struct DiscordUser {
        pub id: String,
        pub username: String,
        pub global_name: Option<String>,
        pub email: String,
        pub avatar: Option<String>,
    }

    let user: DiscordUser = serde_json::from_value(data).map_err(|e| e.to_string())?;

    Ok(ParsedUser {
        name: user.global_name.unwrap_or(user.username),
        email: user.email,
        image: user
            .avatar
            .map(|i| format!("https://cdn.discordapp.com/avatars/{}/{}.png", user.id, i)),
    })
}

/// Creates an OAuth2 client for Discord
pub fn create_discord_oauth2_provider(
    state: &str,
    code_verifier: &str,
) -> Result<OAuthClient, String> {
    let config = AppConfig::from_env().map_err(|e| e.to_string())?;

    Ok(OAuthClient {
        client_id: config.oath.discord_client_id,
        client_secret: config.oath.discord_client_secret,
        redirect_uri: format!("{}/discord", config.oath.redirect_url_web),
        scopes: vec!["identify".to_string(), "email".to_string()],
        state: state.to_string(),
        urls: OAuthUrls {
            auth: "https://discord.com/oauth2/authorize".to_string(),
            token: "https://discord.com/api/oauth2/token".to_string(),
            user: "https://discord.com/api/users/@me".to_string(),
        },
        user_info: UserInfo {
            parser: discord_user_parser,
        },
        code_verifier: code_verifier.to_string(),
    })
}
