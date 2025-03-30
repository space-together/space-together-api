use config::ConfigError;
use serde::Deserialize;
#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
}

#[derive(Deserialize)]
pub struct OathConfig {
    pub redirect_url_web: String,
    pub discord_client_id: String,
    pub discord_client_secret: String,
}

#[derive(Deserialize)]
pub struct SecretKeysConfig {
    pub app_secret: String,
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub oath: OathConfig,
    pub secret_key: SecretKeysConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into()
    }
}

pub const VERIFICATION_TOKEN_EXPIRATION_SECONDS: u64 = 60 * 10;
pub const SESSION_EXPIRATION_SECONDS: u64 = 60 * 60 * 24 * 7;
