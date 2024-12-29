use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileImageModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub ui: ObjectId, // user id
    pub src: String,
    pub co: DateTime, // create on
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileImageModelGet {
    pub id: String,
    pub ui: String, // user id
    pub src: String,
    pub co: String, // create on
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileImageModelNew {
    pub src: String,
    pub ui: ObjectId,
}

impl ProfileImageModel {
    pub fn new(avatar: ProfileImageModelNew) -> Self {
        ProfileImageModel {
            id: None,
            ui: avatar.ui,
            src: avatar.src,
            co: DateTime::now(),
        }
    }

    pub fn format(avatar: Self) -> ProfileImageModelGet {
        ProfileImageModelGet {
            id: avatar.id.map_or("".to_string(), |id| id.to_string()),
            ui: avatar.ui.to_string(),
            src: avatar.src,
            co: avatar.co.try_to_rfc3339_string().unwrap_or("".to_string()),
        }
    }
}
