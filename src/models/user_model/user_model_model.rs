use std::str::FromStr;

use mongodb::bson::{self, doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

use crate::libs::functions::characters_fn::{generate_salt, generate_username, hash_password};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum UserRole {
    STUDENT,
    TEACHER,
    SCHOOLSTAFF,
    ADMIN,
    PARENT,
}

#[allow(clippy::inherent_to_string)]
impl Gender {
    pub(crate) fn to_string(&self) -> String {
        match self {
            Gender::Female => "Female".to_string(),
            Gender::Male => "Male".to_string(),
            Gender::Other => "Other".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,             // name
    pub email: String,            // email
    pub password: Option<String>, // password
    pub salt: Option<u64>,        // slat
    pub role: Option<UserRole>,   // role
    pub username: Option<String>, // username
    pub image: Option<String>,
    pub bio: Option<String>,
    pub phone: Option<String>,       //phone number
    pub gender: Option<Gender>,      // gender
    pub age: Option<DateTime>,       // age
    pub disable: Option<bool>,       // disable
    pub create_at: Option<DateTime>, // created on
    pub update_at: Option<DateTime>, // updated on
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelNew {
    pub name: String,
    pub email: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelPut {
    pub password: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub image: Option<String>,
    pub gender: Option<Gender>,
    pub age: Option<String>,
    pub bio: Option<String>,
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
        let salt = generate_salt();
        UserModel {
            id: None,
            role: Some(UserRole::STUDENT),
            name: user.name.clone(),
            email: user.email,
            salt: user.password.clone().map(|_| salt),
            gender: None,
            age: None,
            image: None,
            phone: None,
            bio: None,
            disable: Some(false),
            username: Some(
                user.username
                    .unwrap_or_else(|| generate_username(&user.name)),
            ),
            password: user
                .password
                .clone()
                .map(|p| hash_password(&p, user.password.map(|_| salt).unwrap_or_default())),
            create_at: Some(DateTime::now()),
            update_at: None,
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

        insert_if_some("role", user.role.map(bson::Bson::String));
        insert_if_some("bio", user.bio.map(bson::Bson::String));

        insert_if_some("image", user.image.map(bson::Bson::String));
        insert_if_some(
            "age",
            user.age
                .map(|age| bson::Bson::DateTime(DateTime::parse_rfc3339_str(&age).unwrap())),
        );
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
            set_doc.insert("update_at", bson::Bson::DateTime(DateTime::now()));
        }

        set_doc
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserModelGet {
    pub id: String,
    pub role: Option<UserRole>,
    pub name: String,
    pub image: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub disable: Option<bool>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub gender: Option<Gender>,
    pub bio: Option<String>,
    pub age: Option<String>,
    pub create_at: Option<String>,
    pub update_at: Option<String>,
}

impl UserModelGet {
    pub fn format(user: UserModel) -> Self {
        UserModelGet {
            id: user.id.map_or("".to_string(), |id| id.to_string()),
            role: user.role,
            name: user.name,
            username: user.username,
            email: user.email,
            image: user.image,
            gender: user.gender,
            age: user.age.map(|age| {
                age.try_to_rfc3339_string()
                    .unwrap_or_else(|_| "".to_string())
            }),
            bio: user.bio,
            phone: user.phone,
            disable: user.disable,
            password: user.password,
            update_at: user.update_at.map(|up| {
                up.try_to_rfc3339_string()
                    .unwrap_or_else(|_| "".to_string())
            }),
            create_at: user.create_at.map(|create| {
                create
                    .try_to_rfc3339_string()
                    .unwrap_or_else(|_| "".to_string())
            }),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AccountProviders {
    Credentials,
}

#[allow(clippy::inherent_to_string)]
impl AccountProviders {
    pub(crate) fn to_string(&self) -> String {
        match self {
            AccountProviders::Credentials => "Credentials".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAccount {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub session_id: Option<ObjectId>,
    pub provider: AccountProviders,
    pub expires_at: Option<u64>,
    pub create_at: DateTime,
    pub updated_at: Option<DateTime>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAccountNew {
    pub user_id: String,
    pub provider: AccountProviders,
    pub expires_at: Option<u64>,
    pub session_id: Option<String>,
}

impl UserAccount {
    pub fn new(data: UserAccountNew) -> Self {
        UserAccount {
            id: None,
            user_id: ObjectId::from_str(&data.user_id).unwrap(),
            provider: data.provider,
            expires_at: data.expires_at,
            session_id: data.session_id.map(|i| ObjectId::from_str(&i).unwrap()),
            create_at: DateTime::now(),
            updated_at: None,
        }
    }
}
