use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct UserRoleModel {
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
            rl: role.rl,
            co: DateTime::now(),
        }
    }
}
