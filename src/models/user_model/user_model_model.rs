use std::str::FromStr;

use mongodb::bson::{self, doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};
use sha256::digest;

use crate::libs::functions::characters_fn::generate_username;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub rl: Option<ObjectId>, // role
    pub nm: String,           // name
    pub un: Option<String>,   // username
    pub em: String,           // email
    pub ph: Option<String>,   //phone number
    pub gd: Option<Gender>,   // gender
    pub ds: Option<bool>,     // disable
    pub pw: Option<String>,   // password
    pub co: DateTime,         // created on
    pub up: Option<DateTime>, // updated on
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelNew {
    pub nm: String,
    pub un: Option<String>,
    pub rl: String,
    pub em: String,
    pub ph: Option<String>,
    pub pw: String,
    pub gd: Gender,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelPut {
    pub rl: Option<String>,
    pub un: Option<String>,
    pub nm: Option<String>,
    pub em: Option<String>,
    pub ph: Option<String>,
    pub pw: Option<String>,
    pub gd: Option<Gender>,
    pub ds: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsersDeleteManyModelHandle {
    pub users: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsersUpdateManyModel {
    pub id: String,
    pub user: UserModelPut,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsersUpdateManyModelHandle {
    pub users: Vec<UsersUpdateManyModel>,
}

impl UserModel {
    pub fn new(user: UserModelNew) -> Self {
        UserModel {
            id: None,
            rl: Some(ObjectId::from_str(&user.rl).unwrap()),
            nm: user.nm.clone(),
            em: user.em,
            gd: Some(user.gd),
            ph: user.ph,
            ds: Some(false),
            un: Some(user.un.unwrap_or_else(|| generate_username(&user.nm))),
            pw: Some(digest(user.pw)),
            co: DateTime::now(),
            up: None,
        }
    }

    pub fn put(user: UserModelPut) -> Document {
        let mut set_doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                set_doc.insert(key, v);
                is_updated = true;
            }
        };

        insert_if_some(
            "rl",
            user.rl
                .map(|rl| bson::Bson::ObjectId(ObjectId::from_str(&rl).unwrap())),
        );
        insert_if_some("nm", user.nm.map(bson::Bson::String));
        insert_if_some("ds", user.ds.map(bson::Bson::Boolean));
        insert_if_some("un", user.un.map(bson::Bson::String));
        insert_if_some("em", user.em.map(bson::Bson::String));
        insert_if_some("ph", user.ph.map(bson::Bson::String));
        insert_if_some("pw", user.pw.map(bson::Bson::String));
        insert_if_some(
            "gd",
            user.gd.map(|gender| bson::Bson::String(gender.to_string())),
        );

        if is_updated {
            set_doc.insert("uo", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelGet {
    pub id: String,
    pub rl: String,
    pub nm: String,
    pub un: Option<String>,
    pub em: String,
    pub ds: Option<bool>,
    pub ph: Option<String>,
    pub pw: Option<String>,
    pub gd: Option<Gender>,
    pub co: String,
    pub uo: Option<String>,
}

impl UserModelGet {
    pub fn format(user: UserModel) -> Self {
        UserModelGet {
            id: user.id.map_or("".to_string(), |id| id.to_string()),
            rl: user.rl.map_or("".to_string(), |rl| rl.to_string()),
            nm: user.nm,
            un: user.un,
            em: user.em,
            gd: user.gd,
            ph: user.ph,
            ds: user.ds,
            pw: user.pw,
            co: user
                .co
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            uo: user.up.map(|up| {
                up.try_to_rfc3339_string()
                    .unwrap_or_else(|_| "".to_string())
            }),
        }
    }
}
