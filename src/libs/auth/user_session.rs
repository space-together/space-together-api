use aes_gcm::aead::{Aead, AeadCore, OsRng}; // Import AeadCore for generate_nonce
use aes_gcm::{Aes256Gcm, Key, KeyInit};
use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, DateTime};
use mongodb::options::IndexOptions;
use mongodb::IndexModel;
use rand::{distributions::Alphanumeric, Rng};
use std::sync::Arc;

use crate::config::application_conf::{AppConfig, SESSION_EXPIRATION_SECONDS};
use crate::models::auth::session_model::{
    UserSessionModelGet, VerificationToken, VerificationTokenNew,
};
use crate::models::user_model::user_model_model::UserModel;
use crate::{
    models::auth::session_model::{UserSessionModel, UserSessionModelNew},
    AppState,
};

use super::user_account::update_user_account_expires;

fn get_secret_key() -> Result<String, String> {
    AppConfig::from_env()
        .map(|res| res.secret_key.app_secret)
        .map_err(|e| e.to_string())
}

fn encrypt_data(data: &str) -> Result<String, String> {
    let key_str = get_secret_key()?;
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted = cipher
        .encrypt(&nonce, data.as_bytes())
        .map_err(|_| "Encryption failed".to_string())?;

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&encrypted);
    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(123)
        .map(char::from)
        .collect()
}

pub fn user_session_expires() -> DateTime {
    DateTime::from_millis(
        (Utc::now() + Duration::seconds(SESSION_EXPIRATION_SECONDS as i64)).timestamp_millis(),
    )
}

pub async fn create_user_session(
    user: UserModel,
    state: Arc<AppState>,
) -> Result<UserSessionModel, String> {
    let token = generate_token();
    let session_data = format!(
        "{}|{}|{}|{:#?}|{}",
        user.id.unwrap_or_default(),
        user.username.unwrap_or_default(),
        user.email,
        user.role,
        user.image.clone().unwrap_or_default(),
    );

    let encrypted_session = encrypt_data(&session_data)?;
    let data = UserSessionModelNew {
        session_token: encrypted_session,
        user_id: user.id.unwrap_or_default(),
        expires_at: user_session_expires(),
        token,
    };

    let index_token = IndexModel::builder()
        .keys(doc! {"token": 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    state
        .db
        .user_session
        .collection
        .create_index(index_token)
        .await
        .map_err(|e| e.to_string())?;

    let session_id = state
        .db
        .user_session
        .create(
            UserSessionModel::new(data),
            Some("user_session".to_string()),
        )
        .await
        .map_err(|e| e.to_string())?;

    state
        .db
        .user_session
        .get_one_by_id(session_id, Some("user_session".to_string()))
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_user_session(
    token: &str,
    state: Arc<AppState>,
) -> Result<UserSessionModelGet, String> {
    state
        .db
        .user_session
        .collection
        .find_one(doc! {"token": token})
        .await
        .map_err(|e| e.to_string())?
        .map(UserSessionModel::format)
        .ok_or("Session not found".to_string())
}

pub async fn delete_user_session(token: &str, state: Arc<AppState>) -> Result<String, String> {
    state
        .db
        .user_session
        .collection
        .delete_one(doc! {"token": token})
        .await
        .map_err(|e| e.to_string())?;

    Ok("Session deleted".to_string())
}

pub async fn update_session_by_id(
    state: &Arc<AppState>,
    id: &ObjectId,
) -> Result<UserSessionModelGet, String> {
    let session = state
        .db
        .user_session
        .collection
        .find_one_and_update(doc! {"_id": id}, doc! {"$set" : UserSessionModel::put()})
        .await
        .map_err(|e| e.to_string())?;

    match session {
        Some(d) => get_session_by_user_id(state.clone(), &d.user_id)
            .await
            .map(UserSessionModel::format),
        None => Err("Session not found".to_string()),
    }
}

pub async fn update_user_session_expires(
    token: &str,
    state: &Arc<AppState>,
) -> Result<UserSessionModelGet, String> {
    let session = state
        .db
        .user_session
        .collection
        .find_one_and_update(
            doc! {"token": token},
            doc! {"$set" : UserSessionModel::put_expires()},
        )
        .await
        .map_err(|e| e.to_string())?;

    match session {
        Some(d) => {
            let get_session = get_session_by_user_id(state.clone(), &d.user_id).await?;
            update_user_account_expires(state, &d.user_id).await?;
            Ok(UserSessionModel::format(get_session))
        }
        None => Err("Session not found".to_string()),
    }
}

pub async fn get_session_by_user_id(
    state: Arc<AppState>,
    user_id: &ObjectId,
) -> Result<UserSessionModel, String> {
    let session = state
        .db
        .user_session
        .collection
        .find_one(doc! {"user_id": user_id})
        .await
        .map_err(|e| e.to_string())?;

    session.ok_or_else(|| "Session not found".to_string())
}

// verification oauth token
pub async fn create_verification_token(
    state: &Arc<AppState>,
    state_data: &Option<String>,
    code_verification: &Option<String>,
) -> Result<VerificationToken, String> {
    let index = IndexModel::builder()
        .keys(doc! {"code_verifier" : 1 , "state" : 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if let Err(err) = state
        .db
        .verification_token
        .collection
        .create_index(index)
        .await
    {
        return Err(err.to_string());
    }
    let token = VerificationTokenNew {
        state: state_data.clone(),
        code_verifier: code_verification.clone(),
    };

    let create = state
        .db
        .verification_token
        .create(
            VerificationToken::new(token),
            Some("verification_token".to_string()),
        )
        .await
        .map_err(|e| e.to_string())?;
    let data = state
        .db
        .verification_token
        .get_one_by_id(create, Some("verification_token".to_string()))
        .await
        .map_err(|e| e.to_string())?;
    Ok(data)
}

// get verification token

pub async fn get_verification_token_by_state(
    state: &Arc<AppState>,
    data_state: &str,
) -> Result<VerificationToken, String> {
    let token = state
        .db
        .verification_token
        .collection
        .find_one(doc! {"state" : data_state})
        .await
        .map_err(|e| e.to_string())?;

    match token {
        Some(res) => Ok(res),
        None => Err(format!(
            "No verification Token found by [{}], token should be expired or not stored",
            data_state
        )),
    }
}
