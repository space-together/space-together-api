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
}
