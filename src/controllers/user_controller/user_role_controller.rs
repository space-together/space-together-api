use std::sync::Arc;

use crate::{
    error::user_error::user_role_error::{UserRoleError, UserRoleResult},
    models::user_model::user_role_model::{UserRoleModelGet, UserRoleModelNew},
    AppState,
};

pub async fn controller_create_user_model(
    role: UserRoleModelNew,
    state: Arc<AppState>,
) -> UserRoleResult<UserRoleModelGet> {
    let find_role = state
        .db
        .user_role
        .get_user_role_by_rl(role.rl.clone())
        .await;

    if find_role.is_ok() {
        return Err(UserRoleError::RoleIsReadyExit);
    }

    let create = state.db.user_role.create_user_role(role).await;

    match create {
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

pub async fn controller_get_all_user_roles(
    state: Arc<AppState>,
) -> UserRoleResult<Vec<UserRoleModelGet>> {
    let get_all = state.db.user_role.get_all_user_roles().await;
    match get_all {
        Ok(roles) => Ok(roles),
        Err(err) => Err(err),
    }
}
