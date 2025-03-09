use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdType {
    Object(ObjectId),
    String(String),
}

impl IdType {
    pub fn to_object(&self) -> Option<ObjectId> {
        match self {
            IdType::Object(id) => Some(id.clone()),
            IdType::String(s) => ObjectId::parse_str(s).ok(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            IdType::Object(id) => id.to_hex(),
            IdType::String(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DateType {
    DateTime(DateTime),
    String(String),
}
