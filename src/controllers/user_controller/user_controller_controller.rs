use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::user_error::user_error_::{UserError, UserResult},
    libs::functions::object_id::change_insertoneresult_into_object_id,
    models::user_model::user_model_model::{UserModelGet, UserModelNew, UserModelPut},
    AppState,
};

pub async fn controller_create_user(
    user: UserModelNew,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    match state.db.user.create_user(user).await {
        Ok(_id) => {
            match state
                .db
                .user
                .get_user_by_id(change_insertoneresult_into_object_id(_id))
                .await
            {
                Ok(res) => Ok(UserModelGet::format(res)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_user_update_by_id(
    user: UserModelPut,
    id: ObjectId,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    if let Some(ref email) = user.em {
        if (state.db.user.get_user_by_email(email.to_string()).await).is_ok() {
            return Err(UserError::EmailIsReadyExit);
        }
    }
    match state.db.user.update_user_by_id(user, id).await {
        Err(err) => Err(err),
        Ok(doc) => match state.db.user.get_user_by_id(doc.id.unwrap()).await {
            Ok(res) => Ok(UserModelGet::format(res)),
            Err(err) => Err(err),
        },
    }
}

pub async fn controller_get_user_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> UserResult<UserModelGet> {
    match state.db.user.get_user_by_id(id).await {
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

pub async fn controller_users_get_all_by_role(
    state: Arc<AppState>,
    role: String,
) -> UserResult<Vec<UserModelGet>> {
    match state.db.user_role.get_user_role_by_rl(role).await {
        Err(_) => Err(UserError::CanNotGetRole),
        Ok(res) => match state.db.user.get_users_by_rl(res.id.unwrap()).await {
            Ok(res) => Ok(res.into_iter().map(UserModelGet::format).collect()),
            Err(err) => Err(err),
        },
    }
}
