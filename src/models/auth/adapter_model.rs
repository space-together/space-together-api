use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub provider: String,
    pub provider_account_id: String,
    pub create_on: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountModelGet {
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub provider_account_id: String,
    pub create_on: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountModelNew {
    pub provider: String,
    pub provider_account_id: String,
    pub user_id: String,
}

impl AccountModel {
    pub fn new(account: AccountModelNew) -> Self {
        AccountModel {
            id: None,
            user_id: ObjectId::from_str(&account.user_id).unwrap(),
            provider: account.provider,
            provider_account_id: account.provider_account_id,
            create_on: DateTime::now(),
        }
    }

    pub fn format(account: Self) -> AccountModelGet {
        AccountModelGet {
            id: account.id.map_or("".to_string(), |id| id.to_string()),
            provider_account_id: account.provider_account_id,
            user_id: account.user_id.to_string(),
            provider: account.provider,
            create_on: account
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub expires: DateTime,
    pub session_token: String,
    pub create_on: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionModelGet {
    pub id: String,
    pub user_id: String,
    pub expires: String,
    pub session_token: String,
    pub create_on: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionModelNew {
    pub user_id: String,
    pub expires: String,
    pub session_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionModelPut {
    pub user_id: Option<String>,
    pub expires: Option<String>,
    pub session_token: Option<String>,
}

impl SessionModel {
    pub fn new(session: SessionModelNew) -> Self {
        SessionModel {
            id: None,
            user_id: ObjectId::from_str(&session.user_id).unwrap(),
            expires: DateTime::parse_rfc3339_str(&session.expires)
                .expect("can not change expires into date"),
            session_token: session.session_token,
            create_on: DateTime::now(),
        }
    }

    pub fn format(session: Self) -> SessionModelGet {
        SessionModelGet {
            id: session.id.map_or("".to_string(), |id| id.to_string()),
            session_token: session.session_token,
            user_id: session.user_id.to_string(),
            expires: session
                .expires
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            create_on: session
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
        }
    }

    pub fn put(session: SessionModelPut) -> Document {
        let mut set_doc = Document::new();

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
            }
        };

        insert_if_some("user_id", session.user_id.map(bson::Bson::String));
        insert_if_some("expires", session.expires.map(bson::Bson::String));
        insert_if_some(
            "session_token",
            session.session_token.map(bson::Bson::String),
        );

        set_doc
    }
}
