use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::libs::auth::user_session::{generate_token, user_session_expires};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserSessionModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub session_token: String,
    pub token: String,
    pub expires_at: DateTime,
    pub created_at: DateTime,
    pub update_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserSessionModelGet {
    pub id: String,
    pub user_id: String,
    pub session_token: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
    pub update_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserSessionModelNew {
    pub session_token: String,
    pub user_id: ObjectId,
    pub token: String,
    pub expires_at: DateTime,
}

impl UserSessionModel {
    pub fn new(data: UserSessionModelNew) -> Self {
        UserSessionModel {
            id: None,
            session_token: data.session_token,
            token: data.token,
            user_id: data.user_id,
            expires_at: data.expires_at,
            created_at: DateTime::now(),
            update_at: None,
        }
    }
    pub fn format(data: Self) -> UserSessionModelGet {
        UserSessionModelGet {
            id: data.id.map_or("".to_string(), |i| i.to_string()),
            session_token: data.session_token,
            user_id: data.user_id.to_string(),
            expires_at: data.expires_at.try_to_rfc3339_string().unwrap_or_default(),
            update_at: data
                .update_at
                .map(|d| d.try_to_rfc3339_string().unwrap_or_default()),
            created_at: data.created_at.try_to_rfc3339_string().unwrap_or_default(),
            token: data.token,
        }
    }

    pub fn put() -> Document {
        let doc = doc! {
            "token": generate_token(),
            "expires_at": user_session_expires(), // Ensure this returns `mongodb::bson::DateTime`
            "update_at": DateTime::now(),
        };
        doc
    }

    pub fn put_expires() -> Document {
        let doc = doc! {
          "expires_at": user_session_expires(),
         "update_at": DateTime::now(),
        };
        doc
    }
}

// VerificationToken

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VerificationToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub state: Option<String>,
    pub code_verifier: Option<String>,
    pub expires_at: DateTime,
    pub created_at: DateTime,
    pub update_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VerificationTokenNew {
    pub state: Option<String>,
    pub code_verifier: Option<String>,
}

impl VerificationToken {
    pub fn new(token: VerificationTokenNew) -> Self {
        VerificationToken {
            id: None,
            state: token.state,
            code_verifier: token.code_verifier,
            expires_at: user_session_expires(),
            created_at: DateTime::now(),
            update_at: None,
        }
    }
}

pub fn verification_token_expires() -> DateTime {
    DateTime::from_millis(
        (Utc::now() + Duration::seconds(SESSION_EXPIRATION_SECONDS as i64)).timestamp_millis(),
    )
}
