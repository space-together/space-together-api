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

use crate::models::auth::session_model::UserSessionModelGet;
use crate::models::user_model::user_model_model::UserModel;
use crate::{
    models::auth::session_model::{UserSessionModel, UserSessionModelNew},
    AppState,
};

const SECRET_KEY: &[u8; 32] = b"super_secret_key_123456789012343";
pub const SESSION_EXPIRATION_SECONDS: u64 = 60 * 60 * 24 * 7;

fn encrypt_data(data: &str) -> String {
    let key = Key::<Aes256Gcm>::from_slice(SECRET_KEY);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // ✅ Now correctly importing AeadCore
    let encrypted = cipher
        .encrypt(&nonce, data.as_bytes())
        .expect("Encryption failed");

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&encrypted);

    general_purpose::STANDARD.encode(combined) // ✅ Encode nonce + encrypted data
}

// fn decrypt_data(encrypted: &str) -> String {
//     let key = Key::<Aes256Gcm>::from_slice(SECRET_KEY);
//     let cipher = Aes256Gcm::new(key);
//     let decoded_data = general_purpose::STANDARD
//         .decode(encrypted)
//         .expect("Base64 decode failed");

//     if decoded_data.len() < 12 {
//         panic!("Invalid encrypted data format");
//     }

//     let (nonce_bytes, cipher_text) = decoded_data.split_at(12);
//     let nonce = Nonce::from_slice(nonce_bytes);

//     let decrypted = cipher
//         .decrypt(nonce, cipher_text)
//         .expect("Decryption failed");
//     String::from_utf8(decrypted).expect("UTF-8 conversion failed")
// }

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

    let encrypted_session = encrypt_data(&session_data);
    let data = UserSessionModelNew {
        session_token: encrypted_session,
        user_id: user.id.unwrap_or_default(),
        expires_at: user_session_expires(),
        token,
    };

    // create index token
    let index_token = IndexModel::builder()
        .keys(doc! {"token": 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = state
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
    let session = state
        .db
        .user_session
        .collection
        .find_one(doc! {"token": token})
        .await
        .map_err(|e| e.to_string())?;

    match session {
        Some(session) => Ok(UserSessionModel::format(session)),
        None => Err("Session not found".to_string()),
    }
}

pub async fn delete_user_session(token: &str, state: Arc<AppState>) -> Result<String, String> {
    let _ = state
        .db
        .user_session
        .collection
        .delete_one(doc! {"token": token})
        .await
        .map_err(|e| e.to_string())?;

    Ok("Session deleted".to_string())
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

    match session {
        Some(session) => Ok(session),
        None => Err("Session not found".to_string()),
    }
}

pub async fn update_session_by_id(
    state: &Arc<AppState>,
    id: ObjectId,
) -> Result<UserSessionModel, String> {
    let session = state
        .db
        .user_session
        .collection
        .find_one_and_update(doc! {"_id": id}, doc! {"$set" : UserSessionModel::put()})
        .await
        .map_err(|e| e.to_string())?;

    match session {
        Some(session) => Ok(session),
        None => Err("Session not found".to_string()),
    }
}
