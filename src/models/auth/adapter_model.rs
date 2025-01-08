use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub provider: String,
    pub provider_account_id: String,
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

impl SessionModel {
    pub fn new(session: SessionModelNew) -> Self {
        SessionModel {
            id: None,
            user_id: ObjectId::from_str(&session.user_id).unwrap(),
            expires: session.expires,
            session_token: session.session_token,
        }
    }
}
