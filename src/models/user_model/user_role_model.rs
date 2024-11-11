use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct UserRoleModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub rl: String,
    pub co: DateTime,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UserRoleModelNew {
    pub rl: String,
}

impl UserRoleModel {
    pub fn new(role: UserRoleModelNew) -> Self {
        UserRoleModel {
            id: None,
            rl: role.rl,
            co: DateTime::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleModelGet {
    pub id: String,
    pub rl: String,
    pub co: String,
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleModelUpdate {
    pub rl: Option<String>,
}
