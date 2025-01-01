use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct UserRoleModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub role: String,
    pub create_on: DateTime,
    pub update_on: Option<DateTime>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UserRoleModelNew {
    pub role: String,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UserRoleModelPut {
    pub role: String,
    pub update_on: Option<DateTime>,
}

impl UserRoleModel {
    pub fn new(role: UserRoleModelNew) -> Self {
        UserRoleModel {
            id: None,
            role: role.role,
            create_on: DateTime::now(),
            update_on: None,
        }
    }

    pub fn put(role: UserRoleModelNew) -> Document {
        doc! {
            "role": role.role,
            "update_on": DateTime::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleModelGet {
    pub id: String,
    pub role: String,
    pub create_on: String,
    pub update_on: Option<String>,
}

impl UserRoleModelGet {
    pub fn format(role: UserRoleModel) -> Self {
        UserRoleModelGet {
            id: role.id.map_or("".to_string(), |id| id.to_string()),
            role: role.role,
            create_on: role
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            update_on: role.update_on.map(|update_on| {
                update_on
                    .try_to_rfc3339_string()
                    .unwrap_or_else(|_| "".to_string())
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleModelUpdate {
    pub role: Option<String>,
}
