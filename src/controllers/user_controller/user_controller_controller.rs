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
    let find_user_role = state
        .db
        .user_role
        .get_user_role_by_id(user.rl.clone())
        .await;
    if find_user_role.is_err() {
        return Err(UserError::UserRoleIsNotExit);
    }

    let find_email = state.db.user.get_user_by_email(user.em.clone()).await;

    if find_email.is_ok() {
        return Err(UserError::EmailIsReadyExit);
    }

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
