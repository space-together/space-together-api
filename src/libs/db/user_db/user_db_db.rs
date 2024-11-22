use futures::stream::StreamExt;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::user_error::user_error_::{UserError, UserResult},
    models::user_model::user_model_model::{UserModel, UserModelGet, UserModelNew, UserModelPut},
};

#[derive(Debug)]
pub struct UserDb {
    pub user: Collection<UserModel>,
}

impl UserDb {
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

    pub async fn create_user(&self, user: UserModelNew) -> UserResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
            "em" : 1,
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        if let Err(err) = self.user.create_index(index).await {
            return Err(UserError::CanNotCreateUser {
                err: err.to_string(),
            });
        }

        if self.get_user_by_email(user.em.clone()).await.is_ok() {
            return Err(UserError::EmailIsReadyExit);
        }

        match UserModel::new(user) {
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
    pub async fn get_user_by_id(&self, id: ObjectId) -> UserResult<UserModel> {
        let get = self.user.find_one(doc! {"_id" : id}).await;
        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound),
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

    async fn find_many_by_field(&self, field: &str, value: ObjectId) -> UserResult<Vec<UserModel>> {
        let mut cursor = self.user.find(doc! { field: value }).await.map_err(|err| {
            UserError::CanNotGetAllUsers {
                err: err.to_string(),
                field: field.to_string().clone(),
            }
        })?;

        let mut users = Vec::new();
        while let Some(data) = cursor.next().await {
            match data {
                Ok(doc) => users.push(doc),
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

    pub async fn update_user_by_id(
        &self,
        user: UserModelPut,
        id: ObjectId,
    ) -> UserResult<UserModel> {
        match self
            .user
            .find_one_and_update(doc! {"_id" : id}, doc! {"$set" : UserModel::put(user)})
            .await
        {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(UserError::UserNotFound),
            Err(err) => Err(UserError::CanNotUpdateUser {
                err: err.to_string(),
            }),
        }
    }

    pub async fn get_users_by_rl(&self, role: ObjectId) -> UserResult<Vec<UserModel>> {
        self.find_many_by_field("rl", role).await
    }
}
