use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub ow: ObjectId,                   // sender by
    pub cont: Option<String>,           // content
    pub cov: ObjectId,                  // conversation
    pub seen_by: Option<Vec<ObjectId>>, // see by users
    pub co: DateTime,                   // created_at
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageModelNew {
    pub ow: String,
    pub cov: String,
    pub cont: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageModelGet {
    pub id: String,
    pub ow: String,
    pub seen_by: Option<Vec<String>>,
    pub cov: String,
    pub cont: Option<String>,
    pub co: String,
}

impl MessageModel {
    pub fn new(message: MessageModelNew) -> MessageModel {
        MessageModel {
            id: None,
            cont: message.cont,
            seen_by: None,
            cov: ObjectId::from_str(&message.cov).unwrap(),
            ow: ObjectId::from_str(&message.ow).unwrap(),
            co: DateTime::now(),
        }
    }

    pub fn format(message: MessageModel) -> MessageModelGet {
        MessageModelGet {
            id: message.id.map_or("".to_string(), |id| id.to_string()),
            cov: message.cov.to_string(),
            cont: message.cont,
            ow: message.ow.to_string(),
            seen_by: message
                .seen_by
                .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
            co: message
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
        }
    }
}
