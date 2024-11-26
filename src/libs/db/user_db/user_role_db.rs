use futures::stream::StreamExt;
use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::user_error::user_role_error::{UserRoleError, UserRoleResult},
    models::user_model::user_role_model::{UserRoleModel, UserRoleModelGet, UserRoleModelNew},
};

#[derive(Debug)]
pub struct UserRoleDb {
    pub role: Collection<UserRoleModel>,
}

impl UserRoleDb {
    pub async fn get_user_role_by_rl(&self, role: String) -> UserRoleResult<UserRoleModel> {
        let get = self.role.find_one(doc! {"rl" : role}).await;
        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserRoleError::RoleNotFound),
            Err(err) => Err(UserRoleError::CanNotFindUserRole {
                err: err.to_string(),
            }),
        }
    }

    pub async fn create_user_role(
        &self,
        role: UserRoleModelNew,
    ) -> UserRoleResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {"rl": 1})
            .options(IndexOptions::builder().unique(true).build())
            .build();

        if let Err(err) = self.role.create_index(index).await {
            return Err(UserRoleError::CanNotCreateUserRole {
                err: err.to_string(),
            });
        }

        if let Ok(role) = self.get_user_role_by_rl(role.rl.clone()).await {
            return Err(UserRoleError::RoleIsReadyExit { role: role.rl });
        }

        match self.role.insert_one(UserRoleModel::new(role)).await {
            Ok(res) => Ok(res),
            Err(err) => Err(UserRoleError::CanNotCreateUserRole {
                err: err.to_string(),
            }),
        }
    }

    pub async fn get_user_role_by_id(&self, id: String) -> UserRoleResult<UserRoleModel> {
        let obj_id = ObjectId::from_str(&id);
        if obj_id.is_err() {
            return Err(UserRoleError::InvalidId);
        };
        let get = self.role.find_one(doc! {"_id" : obj_id.unwrap()}).await;
        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserRoleError::RoleNotFound),
            Err(err) => Err(UserRoleError::CanNotFindUserRole {
                err: err.to_string(),
            }),
        }
    }

    pub async fn update_user_role(
        &self,
        id: String,
        role: UserRoleModelNew,
    ) -> UserRoleResult<UserRoleModel> {
        match ObjectId::from_str(&id) {
            Err(_) => Err(UserRoleError::InvalidId),
            Ok(_id) => match self
                .role
                .find_one_and_update(doc! {"_id" : _id}, doc! {"$set" : UserRoleModel::put(role)})
                .await
            {
                Ok(Some(res)) => Ok(res),
                Ok(None) => Err(UserRoleError::RoleNotFound),
                Err(err) => Err(UserRoleError::CanNotUpdateUserRole {
                    err: err.to_string(),
                }),
            },
        }
    }

    pub async fn delete_user_role(&self, id: String) -> UserRoleResult<UserRoleModel> {
        match ObjectId::from_str(&id) {
            Err(_) => Err(UserRoleError::InvalidId),
            Ok(_id) => match self.role.find_one_and_delete(doc! {"_id" : _id}).await {
                Ok(Some(res)) => Ok(res),
                Ok(None) => Err(UserRoleError::RoleNotFound),
                Err(err) => Err(UserRoleError::CanNotUpdateUserRole {
                    err: err.to_string(),
                }),
            },
        }
    }

    pub async fn get_all_user_roles(&self) -> UserRoleResult<Vec<UserRoleModelGet>> {
        let cursor =
            self.role
                .find(doc! {})
                .await
                .map_err(|err| UserRoleError::CanNotFoundAllUserRole {
                    err: err.to_string(),
                });
        let mut roles: Vec<UserRoleModelGet> = Vec::new();

        match cursor {
            Ok(mut res) => {
                while let Some(result) = res.next().await {
                    match result {
                        Ok(doc) => roles.push(UserRoleModelGet::format(doc)),
                        Err(err) => {
                            return Err(UserRoleError::CanNotFoundAllUserRole {
                                err: err.to_string(),
                            })
                        }
                    }
                }
                Ok(roles)
            }
            Err(err) => Err(err),
        }
    }
}
