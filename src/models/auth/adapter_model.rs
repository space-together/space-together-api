use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub provider: String,
    pub provider_account_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountModelNew {
    provider: String,
    provider_account_id: String,
    user_id: String,
}

impl AccountModel {
    pub fn new(account: AccountModelNew) -> Self {
        AccountModel {
            id: None,
            user_id: ObjectId::from_str(&account.user_id).unwrap(),
            provider: account.provider,
            provider_account_id: account.provider_account_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub expires: String,
    pub session_token: String,
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
            expires: session.expires,
            session_token: session.session_token,
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
