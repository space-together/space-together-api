use std::sync::Arc;

use crate::{
    error::user_error::user_role_error::{UserRoleError, UserRoleResult},
    libs::functions::object_id::change_object_id_into_string,
    models::user_model::user_role_model::{UserRoleModelGet, UserRoleModelNew},
    AppState,
};

pub async fn controller_create_user_model(
    role: UserRoleModelNew,
    state: Arc<AppState>,
) -> UserRoleResult<UserRoleModelGet> {
    match state.db.user_role.create_user_role(role).await {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(UserRoleError::InvalidId)
                .unwrap();

            let get = state.db.user_role.get_user_role_by_id(id).await;

            match get {
                Ok(role) => Ok(UserRoleModelGet::format(role)),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn controller_get_user_role(
    id: String,
    state: Arc<AppState>,
) -> UserRoleResult<UserRoleModelGet> {
    let get = state.db.user_role.get_user_role_by_id(id).await;
    match get {
        Ok(role) => Ok(UserRoleModelGet::format(role)),
        Err(err) => Err(err),
    }
}

pub async fn controller_get_user_role_name(
    name: String,
    state: Arc<AppState>,
) -> UserRoleResult<UserRoleModelGet> {
    match state.db.user_role.get_user_role_by_rl(name).await {
        Ok(role) => Ok(UserRoleModelGet::format(role)),
        Err(err) => Err(err),
    }
}

pub async fn controller_user_role_delete(
    id: String,
    state: Arc<AppState>,
) -> UserRoleResult<UserRoleModelGet> {
    match state.db.user_role.delete_user_role(id).await {
        Ok(role) => Ok(UserRoleModelGet::format(role)),
        Err(err) => Err(err),
    }
}

pub async fn controller_user_role_update(
    id: String,
    role: UserRoleModelNew,
    state: Arc<AppState>,
) -> UserRoleResult<UserRoleModelGet> {
    match state.db.user_role.update_user_role(id, role).await {
        Ok(res) => match state
            .db
            .user_role
            .get_user_role_by_id(change_object_id_into_string(res.id.unwrap()))
            .await
        {
            Ok(role) => Ok(UserRoleModelGet::format(role)),
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    }
}

pub async fn controller_get_all_user_roles(
    state: Arc<AppState>,
) -> UserRoleResult<Vec<UserRoleModelGet>> {
    let get_all = state.db.user_role.get_all_user_roles().await;
    match get_all {
        Ok(roles) => Ok(roles),
        Err(err) => Err(err),
    }
}
