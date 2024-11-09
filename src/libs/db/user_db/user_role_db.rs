use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::user_error::user_role_error::{UserRoleError, UserRoleResult},
    models::user_model::user_role_model::{UserRoleModel, UserRoleModelNew},
};

#[derive(Debug)]
pub struct UserRoleDb {
    pub role: Collection<UserRoleModel>,
}

impl UserRoleDb {
    pub async fn create_user_role(
        &self,
        role: UserRoleModelNew,
    ) -> UserRoleResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
                "rl" : 1,
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let one_index = self.role.create_index(index).await;
        if one_index.is_err() {
            return Err(UserRoleError::RoleIsReadyExit);
        };

        let create = self.role.insert_one(UserRoleModel::new(role)).await;
        match create {
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
}
