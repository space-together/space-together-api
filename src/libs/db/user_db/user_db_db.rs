use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::user_error::user_error_::{UserError, UserResult},
    models::user_model::user_model_model::{UserModel, UserModelNew},
};

#[derive(Debug)]
pub struct UserDb {
    pub user: Collection<UserModel>,
}

impl UserDb {
    pub async fn create_user(&self, user: UserModelNew) -> UserResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
            "em" : 1,
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let one_index = self.user.create_index(index).await;
        if one_index.is_err() {
            return Err(UserError::UserIsReadyExit);
        };
        let new = UserModel::new(user);

        match new {
            Ok(ok) => {
                let create = self.user.insert_one(ok).await;
                match create {
                    Ok(res) => Ok(res),
                    Err(err) => Err(UserError::CanNotCreateUser {
                        err: err.to_string(),
                    }),
                }
            }
            Err(err) => Err(err),
        }
    }
    pub async fn get_user_by_id(&self, id: String) -> UserResult<UserModel> {
        let obj_id = ObjectId::from_str(&id);
        if obj_id.is_err() {
            return Err(UserError::InvalidId);
        };
        let get = self.user.find_one(doc! {"_id" : obj_id.unwrap()}).await;
        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound),
            Err(err) => Err(UserError::CanNotFindUser {
                err: err.to_string(),
            }),
        }
    }

    pub async fn get_user_by_email(&self, email: String) -> UserResult<UserModel> {
        let get = self.user.find_one(doc! {"em" : email}).await;
        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound),
            Err(err) => Err(UserError::CanNotFindUser {
                err: err.to_string(),
            }),
        }
    }
}
