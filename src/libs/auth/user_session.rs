use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key}; //Nonce
use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, DateTime};
use mongodb::options::IndexOptions;
use mongodb::IndexModel;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::application_conf::{AppConfig, SESSION_EXPIRATION_SECONDS};
use crate::models::auth::session_model::{
    UserSessionModelGet, VerificationToken, VerificationTokenNew,
};
use crate::models::user_model::user_model_model::{UserModel, UserRole};
use crate::{
    models::auth::session_model::{UserSessionModel, UserSessionModelNew},
    AppState,
};

use super::user_account::update_user_account_expires;

fn get_secret_key() -> Result<[u8; 32], String> {
    let key_str = AppConfig::from_env()
        .map(|res| res.secret_key.app_secret)
        .map_err(|e| e.to_string())?;

    let mut key_bytes = [0u8; 32];
    let key_slice = key_str.as_bytes();

    if key_slice.len() >= 32 {
        key_bytes.copy_from_slice(&key_slice[..32]); // Truncate
    } else {
        key_bytes[..key_slice.len()].copy_from_slice(key_slice); // Pad with zeros
    }

    Ok(key_bytes)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserSessionModelDecode {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub image: String,
}

fn encrypt_user_session_data(data: &str) -> Result<String, String> {
    let key = get_secret_key()?; // Now it's exactly 32 bytes
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    match cipher.encrypt(&nonce, data.as_bytes()) {
        Ok(encrypted) => {
            let mut combined = nonce.to_vec();
            combined.extend_from_slice(&encrypted);
            Ok(general_purpose::STANDARD.encode(combined))
        }
        Err(_) => Err("Encryption failed".to_string()),
    }
}

// fn decrypt_user_session_data_raw(encrypted_data: &str) -> Result<String, String> {
//     let key = get_secret_key()?;
//     let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

//     let decoded = general_purpose::STANDARD
//         .decode(encrypted_data)
//         .map_err(|_| "Base64 decoding failed".to_string())?;

//     if decoded.len() < 12 {
//         return Err("Invalid user encrypted data".to_string());
//     }

//     let (nonce_bytes, ciphertext) = decoded.split_at(12);
//     let nonce = Nonce::from_slice(nonce_bytes);

//     cipher
//         .decrypt(nonce, ciphertext)
//         .map_err(|_| "Decryption failed".to_string())
//         .and_then(|decrypted| String::from_utf8(decrypted).map_err(|_| "Invalid UTF-8".to_string()))
// }

// pub fn decrypt_user_session_data(encrypted_data: &str) -> Result<UserSessionModelDecode, String> {
//     let decrypted = decrypt_user_session_data_raw(encrypted_data)?;
//     let parts: Vec<&str> = decrypted.split('|').collect();

//     if parts.len() != 5 {
//         return Err("Invalid decrypted user session format".to_string());
//     }

//     let role = parts[3]
//         .replace("Some(", "")
//         .replace(")", "")
//         .replace("\n", "")
//         .trim()
//         .trim_end_matches(',')
//         .to_string();

//     Ok(UserSessionModelDecode {
//         user_id: parts[0].to_string(),
//         username: parts[1].to_string(),
//         email: parts[2].to_string(),
//         role: role
//             .parse::<UserRole>()
//             .map_err(|_| "Invalid user role".to_string())?,
//         image: parts[4].to_string(),
//     })
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
    let user_id = user.id.ok_or("User ID is missing")?;
    let token = generate_token();
    let session_data = format!(
        "{}|{}|{}|{:#?}|{}",
        user_id,
        user.username.unwrap_or_default(),
        user.email,
        user.role,
        user.image.clone().unwrap_or_default(),
    );

    let encrypted_session = encrypt_user_session_data(&session_data)?;
    let data = UserSessionModelNew {
        session_token: encrypted_session,
        user_id,
        expires_at: user_session_expires(),
        token,
    };

    let index_token = IndexModel::builder()
        .keys(doc! {"session_token": 1, "token" : 1})
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

pub async fn get_session_by_user_id(
    state: Arc<AppState>,
    user_id: &ObjectId,
) -> Result<UserSessionModel, String> {
    state
        .db
        .user_session
        .collection
        .find_one(doc! {"user_id": user_id})
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Session not found".to_string())
}

pub async fn update_session_by_id(
    state: &Arc<AppState>,
    id: &ObjectId,
) -> Result<UserSessionModelGet, String> {
    let session = state
        .db
        .user_session
        .collection
        .find_one_and_update(
            doc! {"_id": id},
            doc! {"$set" : {"expires_at": user_session_expires(), "update_at" : DateTime::now()}},
        )
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

pub async fn get_user_session(
    token: &str,
    state: &Arc<AppState>,
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
