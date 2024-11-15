use std::sync::Arc;

use crate::{
    error::user_error::user_error_::{UserError, UserResult},
    models::user_model::user_model_model::{UserModelGet, UserModelNew},
    AppState,
};

pub async fn controller_create_user(
    user: UserModelNew,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    let create = state.db.user.create_user(user).await;
    match create {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(UserError::InvalidId)
                .unwrap();

            let get = state.db.user.get_user_by_id(id).await;

            match get {
                Ok(res) => Ok(UserModelGet::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_get_user_by_id(
    state: Arc<AppState>,
    id: String,
) -> UserResult<UserModelGet> {
    let get = state.db.user.get_user_by_id(id).await;
    match get {
        Ok(res) => Ok(UserModelGet::format(res)),
        Err(err) => Err(err),
    }
}

pub async fn controller_get_all_users(state: Arc<AppState>) -> UserResult<Vec<UserModelGet>> {
    let get_all = state.db.user.get_all_users().await;
    match get_all {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}
