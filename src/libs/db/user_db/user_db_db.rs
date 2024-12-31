use std::str::FromStr;

use futures::stream::StreamExt;

use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::user_error::user_error_err::{UserError, UserResult},
    libs::functions::characters_fn::{is_valid_name, is_valid_username},
    models::user_model::user_model_model::{
        UserModel, UserModelGet, UserModelNew, UserModelPut, UsersUpdateManyModel,
    },
};

#[derive(Debug)]
pub struct UserDb {
    pub user: Collection<UserModel>,
}

enum UpdateDeleteValueType {
    ObjectId(ObjectId),
    String(String),
}

impl UserDb {
    async fn find_one_by_field(&self, field: &str, value: String) -> UserResult<Option<UserModel>> {
        self.user
            .find_one(doc! { field: value})
            .await
            .map_err(|err| UserError::CanNotFindUser {
                err: err.to_string(),
            })
    }

    pub async fn get_user_by_email(&self, email: String) -> UserResult<UserModel> {
        match self.find_one_by_field("em", email).await {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound {
                field: "email".to_string(),
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn get_user_by_username(&self, username: String) -> UserResult<UserModel> {
        match self.find_one_by_field("un", username).await {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound {
                field: "username".to_string(),
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn create_user(&self, user: UserModelNew) -> UserResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
            "em" : 1,
            "un" : 1
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        if let Err(err) = self.user.create_index(index).await {
            return Err(UserError::CanNotCreateUser {
                err: err.to_string(),
            });
        }

        if let Err(err) = is_valid_name(&user.nm) {
            return Err(UserError::InvalidName { err });
        }

        if self.get_user_by_email(user.em.clone()).await.is_ok() {
            return Err(UserError::UserIsReadyExit {
                field: "Email".to_string(),
                value: user.em.clone(),
            });
        }

        let create = self.user.insert_one(UserModel::new(user)).await;
        match create {
            Ok(res) => Ok(res),
            Err(err) => Err(UserError::CanNotCreateUser {
                err: err.to_string(),
            }),
        }
    }

    pub async fn get_user_by_id(&self, id: ObjectId) -> UserResult<UserModel> {
        let get = self.user.find_one(doc! {"_id" : id}).await;
        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound {
                field: "_id".to_string(),
            }),
            Err(err) => Err(UserError::CanNotFindUser {
                err: err.to_string(),
            }),
        }
    }

    pub async fn get_all_users(&self) -> UserResult<Vec<UserModelGet>> {
        let cursor = self
            .user
            .find(doc! {})
            .await
            .map_err(|err| UserError::CanNotGetAllUsers {
                field: "all".to_string(),
                err: err.to_string(),
            });
        let mut users: Vec<UserModelGet> = Vec::new();
        match cursor {
            Ok(mut res) => {
                while let Some(result) = res.next().await {
                    match result {
                        Ok(doc) => users.push(UserModelGet::format(doc)),
                        Err(err) => {
                            return Err(UserError::CanNotGetAllUsers {
                                field: "all".to_string(),
                                err: err.to_string(),
                            })
                        }
                    }
                }
                Ok(users)
            }
            Err(err) => Err(err),
        }
    }

    async fn find_many_by_field(
        &self,
        field: &str,
        value: ObjectId,
    ) -> UserResult<Vec<UserModelGet>> {
        let mut cursor = self.user.find(doc! { field: value }).await.map_err(|err| {
            UserError::CanNotGetAllUsers {
                err: err.to_string(),
                field: field.to_string().clone(),
            }
        })?;

        let mut users = Vec::new();
        while let Some(data) = cursor.next().await {
            match data {
                Ok(doc) => users.push(UserModelGet::format(doc)),
                Err(err) => {
                    return Err(UserError::CanNotGetAllUsers {
                        err: err.to_string(),
                        field: field.to_string(),
                    });
                }
            }
        }
        Ok(users)
    }

    async fn update_by_field(
        &self,
        user: UserModelPut,
        field: &str,
        value: String,
    ) -> UserResult<UserModel> {
        if let Some(name) = user.nm.clone() {
            if let Err(err) = is_valid_name(&name) {
                return Err(UserError::InvalidName { err });
            }
        }

        if let Some(username) = user.un.clone() {
            if let Err(err) = is_valid_username(&username) {
                return Err(UserError::InvalidUsername { err });
            }
        }

        if let Some(email) = user.em.clone() {
            if self.get_user_by_email(email.clone()).await.is_ok() {
                return Err(UserError::UserIsReadyExit {
                    field: "email".to_string(),
                    value: email.clone(),
                });
            }
        }

        if let Some(username) = user.un.clone() {
            if self.get_user_by_username(username.clone()).await.is_ok() {
                return Err(UserError::UserIsReadyExit {
                    field: "username".to_string(),
                    value: username.clone(),
                });
            }
        }

        let field_value = if field == "_id" {
            match ObjectId::from_str(&value) {
                Ok(object_id) => UpdateDeleteValueType::ObjectId(object_id),
                Err(_) => return Err(UserError::InvalidId),
            }
        } else {
            UpdateDeleteValueType::String(value.clone())
        };

        let field_value_to_use = match field_value {
            UpdateDeleteValueType::ObjectId(object_id) => bson::Bson::ObjectId(object_id),
            UpdateDeleteValueType::String(string) => bson::Bson::String(string),
        };

        match self
            .user
            .find_one_and_update(
                doc! {field: field_value_to_use},
                doc! {"$set" : UserModel::put(user)},
            )
            .await
        {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound {
                field: field.to_string(),
            }),
            Err(err) => Err(UserError::CanNotDoActionUser {
                action: "update".to_string(),
                err: err.to_string(),
            }),
        }
    }

    pub async fn update_user_by_id(
        &self,
        user: UserModelPut,
        id: ObjectId,
    ) -> UserResult<UserModel> {
        self.update_by_field(user, "_id", id.to_string()).await
    }

    pub async fn update_user_by_username(
        &self,
        user: UserModelPut,
        username: String,
    ) -> UserResult<UserModel> {
        self.update_by_field(user, "un", username).await
    }

    pub async fn get_users_by_rl(&self, role: ObjectId) -> UserResult<Vec<UserModelGet>> {
        self.find_many_by_field("rl", role).await
    }

    async fn delete_user_by_field(&self, field: &str, value: String) -> UserResult<UserModel> {
        let field_value = if field == "_id" {
            match ObjectId::from_str(&value) {
                Ok(object_id) => UpdateDeleteValueType::ObjectId(object_id),
                Err(_) => return Err(UserError::InvalidId),
            }
        } else {
            UpdateDeleteValueType::String(value.clone())
        };

        let field_value_to_use = match field_value {
            UpdateDeleteValueType::ObjectId(object_id) => bson::Bson::ObjectId(object_id),
            UpdateDeleteValueType::String(string) => bson::Bson::String(string),
        };

        match self
            .user
            .find_one_and_delete(doc! {field: field_value_to_use})
            .await
        {
            Ok(Some(doc)) => Ok(doc),
            Ok(None) => Err(UserError::UserNotFound {
                field: field.to_string(),
            }),
            Err(err) => Err(UserError::CanNotDoActionUser {
                action: "delete".to_string(),
                err: err.to_string(),
            }),
        }
    }

    pub async fn delete_user_by_id(&self, id: ObjectId) -> UserResult<UserModel> {
        self.delete_user_by_field("_id", id.to_string()).await
    }

    pub async fn delete_user_by_username(&self, username: String) -> UserResult<UserModel> {
        self.delete_user_by_field("un", username).await
    }

    pub async fn delete_users(&self, ids: Vec<ObjectId>) -> UserResult<Vec<UserModelGet>> {
        let mut users = Vec::new();
        for id in ids {
            match self.delete_user_by_id(id).await {
                Ok(user) => users.push(UserModelGet::format(user)),
                Err(err) => return Err(err),
            }
        }
        Ok(users)
    }

    pub async fn update_many(
        &self,
        users: Vec<UsersUpdateManyModel>,
    ) -> UserResult<Vec<UserModelGet>> {
        let mut result_updates = Vec::new();
        for user in users {
            match self
                .update_user_by_id(
                    user.user,
                    ObjectId::from_str(&user.id).expect("Invalid id to disable many users"),
                )
                .await
            {
                Ok(user) => result_updates.push(user),
                Err(err) => return Err(err),
            }
        }

        Ok(result_updates
            .into_iter()
            .map(UserModelGet::format)
            .collect())
    }
}
