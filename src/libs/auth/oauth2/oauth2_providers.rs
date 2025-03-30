use serde::Deserialize;
use serde_json::Value;

use crate::{
    config::application_conf::AppConfig, models::user_model::user_model_model::AccountProviders,
};

pub struct OAuthUrls {
    pub auth: String,
    pub token: String,
    pub user: String,
}

pub struct OAuthClient {
    pub provider: AccountProviders,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub code_verifier: String,
    pub state: String,
    pub urls: OAuthUrls,
    pub user_info: UserInfo,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub parser: fn(Value) -> Result<ParsedUser, String>,
}

#[derive(Debug, Deserialize)]
pub struct ParsedUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub image: Option<String>,
}

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
        id: user.id.clone(),
        name: user.global_name.unwrap_or(user.username),
        email: user.email,
        image: user
            .avatar
            .map(|i| format!("https://cdn.discordapp.com/avatars/{}/${}.png", user.id, i)),
    })
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
}

pub fn create_discord_oath2_provider(state: &str, code_verifier: &str) -> OAuthClient {
    let provider = AccountProviders::Discord;
    let config = AppConfig::from_env().unwrap();
    let client_id = config.oath.discord_client_id;
    let client_secret = config.oath.discord_client_secret;
    let redirect_uri = format!("{}/discord", config.oath.redirect_url_web);
    let scopes = vec!["identify".to_string(), "email".to_string()];

    OAuthClient {
        provider,
        client_id,
        client_secret,
        redirect_uri,
        scopes,
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
    }
}
