use std::str::FromStr;

use mongodb::bson::{self, doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::error::user_error::user_error_::{UserError, UserResult};

#[derive(Debug, Deserialize, Serialize)]
pub enum Gender {
    M,
    F,
    O,
}

#[allow(clippy::inherent_to_string)]
impl Gender {
    pub(crate) fn to_string(&self) -> String {
        match self {
            Gender::F => "F".to_string(),
            Gender::M => "M".to_string(),
            Gender::O => "O".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub rl: Option<ObjectId>,
    pub nm: String,
    pub em: String,
    pub ph: Option<String>,
    pub gd: Option<Gender>,
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
    pub gd: Gender,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserModelPut {
    pub rl: Option<String>,
    pub nm: Option<String>,
    pub em: Option<String>,
    pub ph: Option<String>,
    pub pw: Option<String>,
    pub gd: Option<Gender>,
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
                gd: Some(user.gd),
                ph: user.ph,
                pw: Some(user.pw),
                co: DateTime::now(),
            }),
            Err(_) => Err(UserError::InvalidId),
        }
    }

    pub fn put(user: UserModelPut) -> Document {
        let mut document = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                document.insert(key, v);
                is_updated = true;
            }
        };
        insert_if_some(
            "rl",
            user.rl
                .map(|rl| bson::Bson::ObjectId(ObjectId::from_str(&rl).unwrap())),
        );
        insert_if_some("nm", user.nm.map(bson::Bson::String));
        insert_if_some("em", user.em.map(bson::Bson::String));
        insert_if_some("ph", user.ph.map(bson::Bson::String));
        insert_if_some("pw", user.pw.map(bson::Bson::String));
        insert_if_some(
            "gd",
            user.gd.map(|gender| bson::Bson::String(gender.to_string())),
        );

        if is_updated {
            document.insert("uo", bson::DateTime::now());
        }

        document
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
    pub gd: Option<Gender>,
    pub co: String,
}

impl UserModelGet {
    pub fn format(user: UserModel) -> Self {
        UserModelGet {
            id: user.id.map_or("".to_string(), |id| id.to_string()),
            rl: user.rl.map_or("".to_string(), |rl| rl.to_string()),
            nm: user.nm,
            em: user.em,
            gd: user.gd,
            ph: user.ph,
            pw: user.pw,
            co: user
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
        }
    }
}
