use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct UserRoleModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub rl: String,
    pub co: DateTime,
    pub uo: Option<DateTime>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UserRoleModelNew {
    pub rl: String,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UserRoleModelPut {
    pub rl: String,
    pub uo: Option<DateTime>,
}

impl UserRoleModel {
    pub fn new(role: UserRoleModelNew) -> Self {
        UserRoleModel {
            id: None,
            rl: role.rl,
            co: DateTime::now(),
            uo: None,
        }
    }

    pub fn put(role: UserRoleModelNew) -> Document {
        doc! {
            "rl": role.rl,
            "uo": DateTime::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleModelGet {
    pub id: String,
    pub rl: String,
    pub co: String,
    pub uo: Option<String>,
}

impl UserRoleModelGet {
    pub fn format(role: UserRoleModel) -> Self {
        UserRoleModelGet {
            id: role.id.map_or("".to_string(), |id| id.to_string()),
            rl: role.rl,
            co: role
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            uo: role.uo.map(|uo| {
                uo.try_to_rfc3339_string()
                    .unwrap_or_else(|_| "".to_string())
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleModelUpdate {
    pub rl: Option<String>,
}
