use std::sync::Arc;

use mongodb::bson::oid::ObjectId;

use crate::{
    error::user_error::user_error_err::{UserError, UserResult},
    libs::{
        auth::user_session::decrypt_user_session_data,
        functions::object_id::change_insertoneresult_into_object_id,
    },
    models::user_model::user_model_model::{
        UserModelGet, UserModelNew, UserModelPut, UserRole, UsersUpdateManyModel,
    },
    AppState,
};

pub async fn controller_create_user(
    user: UserModelNew,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    let user_id = state.db.user.create_user(user).await?;
    let user_data = state
        .db
        .user
        .get_user_by_id(change_insertoneresult_into_object_id(user_id))
        .await?;
    Ok(UserModelGet::format(user_data))
}
pub async fn controller_user_update_by_id(
    user: UserModelPut,
    id: ObjectId,
    state: Arc<AppState>,
    token: &str,
) -> UserResult<UserModelGet> {
    let user_session =
        decrypt_user_session_data(token).map_err(|e| UserError::SomeError { err: e })?;

    let existing_user =
        state
            .db
            .user
            .get_user_by_id(id)
            .await
            .map_err(|_| UserError::SomeError {
                err: "User does not exist".to_string(),
            })?;

    // Allow update if the user is the same or if they are an admin
    if existing_user.id.unwrap().to_string() != user_session.user_id
        && user_session.role != UserRole::ADMIN
    {
        return Err(UserError::SomeError {
            err: "You are not allowed to update this account".to_string(),
        });
    }

    let updated_user = state.db.user.update_user_by_id(user, id).await?;

    if let Some(user_id) = updated_user.id {
        let user_data = state.db.user.get_user_by_id(user_id).await?;
        Ok(UserModelGet::format(user_data))
    } else {
        Err(UserError::SomeError {
            err: "Failed to retrieve updated user".to_string(),
        })
    }
}

pub async fn controller_user_update_by_username(
    user: UserModelPut,
    username: String,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    let updated_user = state
        .db
        .user
        .update_user_by_username(user, username)
        .await?;
    let user_data = state
        .db
        .user
        .get_user_by_id(updated_user.id.unwrap())
        .await?;
    Ok(UserModelGet::format(user_data))
}

pub async fn controller_get_user_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> UserResult<UserModelGet> {
    let user_data = state.db.user.get_user_by_id(id).await?;
    Ok(UserModelGet::format(user_data))
}

pub async fn controller_user_get_by_username(
    state: Arc<AppState>,
    username: String,
) -> UserResult<UserModelGet> {
    let user_data = state.db.user.get_user_by_username(username).await?;
    Ok(UserModelGet::format(user_data))
}

pub async fn controller_user_get_user_by_email(
    state: Arc<AppState>,
    email: String,
) -> UserResult<UserModelGet> {
    let user_data = state.db.user.get_user_by_email(email.clone()).await?;
    Ok(UserModelGet::format(user_data))
}

pub async fn controller_user_delete_by_id(
    id: ObjectId,
    state: Arc<AppState>,
) -> UserResult<UserModelGet> {
    let user_data = state.db.user.delete_user_by_id(id).await?;
    Ok(UserModelGet::format(user_data))
}

pub async fn controller_user_delete_many(
    user_ids: Vec<ObjectId>,
    state: Arc<AppState>,
) -> UserResult<Vec<UserModelGet>> {
    let users_to_delete = state.db.user.delete_users(user_ids.clone()).await?;
    Ok(users_to_delete)
}

pub async fn controller_user_update_many(
    users: Vec<UsersUpdateManyModel>,
    state: Arc<AppState>,
) -> UserResult<Vec<UserModelGet>> {
    match state.db.user.update_many(users).await {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

pub async fn controller_user_delete_by_username(
    state: Arc<AppState>,
    username: String,
) -> UserResult<UserModelGet> {
    let deleted_user = state.db.user.delete_user_by_username(username).await?;
    Ok(UserModelGet::format(deleted_user))
}

pub async fn controller_get_all_users(state: Arc<AppState>) -> UserResult<Vec<UserModelGet>> {
    let users = state.db.user.get_all_users().await?;

    Ok(users)
}

pub async fn controller_users_get_all_by_role(
    state: Arc<AppState>,
    role: &str,
) -> UserResult<Vec<UserModelGet>> {
    let users = state.db.user.get_users_by_rl(role).await?;
    Ok(users)
}
