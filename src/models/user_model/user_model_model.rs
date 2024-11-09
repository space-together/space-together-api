use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::error::user_error::user_error_::{UserError, UserResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub rl: Option<ObjectId>,
    pub nm: String,
    pub em: String,
    pub ph: Option<String>,
    pub pw: Option<String>,
    pub co: DateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserModelNew {
    pub nm: String,
    pub rl: String,
    pub em: String,
    pub ph: Option<String>,
    pub pw: String,
}

impl UserModel {
    pub fn new(user: UserModelNew) -> UserResult<UserModel> {
        let rl_id = ObjectId::from_str(&user.rl);
        match rl_id {
            Ok(res) => Ok(UserModel {
                id: None,
                rl: Some(res),
                nm: user.nm,
                em: user.em,
                ph: user.ph,
                pw: Some(user.pw),
                co: DateTime::now(),
            }),
            Err(_) => Err(UserError::InvalidId),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserModelGet {
    pub id: String,
    pub rl: String,
    pub nm: String,
    pub em: String,
    pub ph: Option<String>,
    pub pw: Option<String>,
    pub co: String,
}

impl UserModelGet {
    pub fn format(user: UserModel) -> Self {
        UserModelGet {
            id: user.id.map_or("".to_string(), |id| id.to_string()),
            rl: user.rl.map_or("".to_string(), |rl| rl.to_string()),
            nm: user.nm,
            em: user.em,
            ph: user.ph,
            pw: user.pw,
            co: user
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
        }
    }
}
