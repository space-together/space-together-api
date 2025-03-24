use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionModel {
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
pub struct SessionModelNew {
    pub session_token: String,
    pub user_id: ObjectId,
    pub token: String,
    pub expires_at: DateTime,
}

impl SessionModel {
    pub fn new(data: SessionModelNew) -> Self {
        SessionModel {
            id: None,
            session_token: data.session_token,
            token: data.token,
            user_id: data.user_id,
            expires_at: data.expires_at,
            created_at: DateTime::now(),
            update_at: None,
        }
    }
}
