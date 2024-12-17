use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub rl: ObjectId, //role
    pub nm: String,   // name
    pub em: String,   // email
    pub cot: String,  // content
    pub co: DateTime, // create on
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestModelGet {
    pub id: String,
    pub rl: String,
    pub nm: String,
    pub em: String,
    pub co: String,
    pub cot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestModelNew {
    pub rl: String,
    pub nm: String,
    pub em: String,
    pub cot: String,
}

impl RequestModel {
    pub fn new(request: RequestModelNew) -> Self {
        RequestModel {
            id: None,
            rl: ObjectId::from_str(&request.rl).unwrap(),
            nm: request.nm,
            em: request.em,
            cot: request.cot,
            co: DateTime::now(),
        }
    }
    pub fn format(req: RequestModel) -> RequestModelGet {
        RequestModelGet {
            id: req.id.map_or("".to_string(), |id| id.to_string()),
            rl: req.rl.to_string(),
            em: req.em,
            nm: req.nm,
            cot: req.cot,
            co: req.co.try_to_rfc3339_string().unwrap_or("".to_string()),
        }
    }
}
