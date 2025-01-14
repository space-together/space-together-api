use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SectionModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub create_on: DateTime,
    pub update_on: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SectionModelNew {
    pub name: String,
}
