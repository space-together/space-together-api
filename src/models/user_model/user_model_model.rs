use std::str::FromStr;

use mongodb::bson::{self, doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};
use sha256::digest;

use crate::{
    libs::functions::characters_fn::generate_username,
    models::images_model::profile_images_model::ProfileImageModelGet,
};

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
    pub role: Option<ObjectId>,      // role
    pub name: String,                // name
    pub username: Option<String>,    // username
    pub email: String,               // email
    pub phone: Option<String>,       //phone number
    pub gender: Option<Gender>,      // gender
    pub disable: Option<bool>,       // disable
    pub password: Option<String>,    // password
    pub create_on: DateTime,         // created on
    pub update_on: Option<DateTime>, // updated on
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelNew {
    pub name: String,
    pub username: Option<String>,
    pub role: String,
    pub email: String,
    pub phone: Option<String>,
    pub password: String,
    pub gender: Gender,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelPut {
    pub role: Option<String>,
    pub username: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub image: Option<String>,
    pub gender: Option<Gender>,
    pub disable: Option<bool>,
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
            role: Some(ObjectId::from_str(&user.role).unwrap()),
            name: user.name.clone(),
            email: user.email,
            gender: Some(user.gender),
            phone: user.phone,
            disable: Some(false),
            username: Some(
                user.username
                    .unwrap_or_else(|| generate_username(&user.name)),
            ),
            password: Some(digest(user.password)),
            create_on: DateTime::now(),
            update_on: None,
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
            "role",
            user.role
                .map(|role| bson::Bson::ObjectId(ObjectId::from_str(&role).unwrap())),
        );
        insert_if_some("image", user.image.map(bson::Bson::String));
        insert_if_some("name", user.name.map(bson::Bson::String));
        insert_if_some("disable", user.disable.map(bson::Bson::Boolean));
        insert_if_some("username", user.username.map(bson::Bson::String));
        insert_if_some("email", user.email.map(bson::Bson::String));
        insert_if_some("phone", user.phone.map(bson::Bson::String));
        insert_if_some("password", user.password.map(bson::Bson::String));
        insert_if_some(
            "gender",
            user.gender
                .map(|gender| bson::Bson::String(gender.to_string())),
        );

        if is_updated {
            set_doc.insert("update_on", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelGet {
    pub id: String,
    pub role: String,
    pub name: String,
    pub image: Option<Vec<ProfileImageModelGet>>,
    pub username: Option<String>,
    pub email: String,
    pub disable: Option<bool>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub gender: Option<Gender>,
    pub create_on: String,
    pub update_on: Option<String>,
}

impl UserModelGet {
    pub fn format(user: UserModel) -> Self {
        UserModelGet {
            id: user.id.map_or("".to_string(), |id| id.to_string()),
            role: user.role.map_or("".to_string(), |role| role.to_string()),
            name: user.name,
            username: user.username,
            email: user.email,
            image: None,
            gender: user.gender,
            phone: user.phone,
            disable: user.disable,
            password: user.password,
            create_on: user
                .create_on
                .try_to_rfc3339_string()
                .unwrap_or_else(|_| "".to_string()),
            update_on: user.update_on.map(|up| {
                up.try_to_rfc3339_string()
                    .unwrap_or_else(|_| "".to_string())
            }),
        }
    }
}
