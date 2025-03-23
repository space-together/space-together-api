use chrono::{Duration, Utc};
use futures::FutureExt;
use mongodb::{
    bson::{oid::ObjectId, DateTime},
    IndexModel,
};
use rand::Rng;
use std::sync::Arc;

use crate::{
    models::{
        auth::session_model::{SessionModel, SessionModelNew},
        user_model::user_model_model::UserRole,
    },
    AppState,
};

pub const SESSION_EXPIRATION_SECONDS: i64 = 60 * 60 * 24 * 7;

#[derive(Debug, Clone)]
pub struct CookieOptions {
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<String>, // "strict" or "lax"
    pub expires: Option<u64>,      // Unix timestamp
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
}

pub trait Cookies {
    fn set(&mut self, key: String, value: String, options: CookieOptions);
    fn get(&self, key: &str) -> Option<Cookie>;
    fn delete(&mut self, key: &str);
}

pub struct UserSession {
    id: String,
    role: UserRole,
    email: String,
    name: String,
    image: Option<String>,
}

pub fn generate_session_id() -> String {
    let mut rng = rand::thread_rng();
    (0..128)
        .map(|_| format!("{:x}", rng.gen::<u8>()))
        .collect::<String>()
}

pub async fn create_user_session(
    user_id: ObjectId,
    cookies: &mut impl Cookies,
    state: Arc<AppState>,
) -> Result<(), String> {
    let session_id = generate_session_id();

    let session_data = SessionModelNew {
        session_token: session_id.clone(),
        user_id,
        expires: DateTime::from_millis(
            (Utc::now() + Duration::seconds(SESSION_EXPIRATION_SECONDS)).timestamp_millis(),
        ),
    };
    let _ = state
        .db
        .user_session
        .create(
            SessionModel::new(session_data),
            Some("user_session".to_string()),
        )
        .await
        .map_err(|e| e.to_string())?;

    set_cookies(session_id, cookies);

    Ok(())
}

fn set_cookies(session_id: String, cookies: &mut impl Cookies) {
    let cookie_options = CookieOptions {
        secure: Some(true),
        http_only: Some(true),
        same_site: Some("Strict".to_string()),
        expires: Some(SESSION_EXPIRATION_SECONDS as u64),
    };
    cookies.set("session_id".to_string(), session_id, cookie_options);
}
