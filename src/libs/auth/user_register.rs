use std::sync::Arc;
use tracing::error;

use crate::{
    libs::auth::{
        user_account::update_user_account,
        user_session::{create_user_session, get_session_by_user_id, update_session_by_id},
    },
    libs::functions::{
        characters_fn::verify_password, object_id::change_insertoneresult_into_object_id,
    },
    models::{
        auth::session_model::{UserSessionModel, UserSessionModelGet},
        user_model::user_model_model::{
            AccountProviders, UserAccountPut, UserLoginModel, UserModelNew,
        },
    },
    AppState,
};

use super::{user_account::create_user_account, user_session::user_session_expires};

pub async fn user_register(
    state: Arc<AppState>,
    data: UserModelNew,
) -> Result<UserSessionModelGet, String> {
    let create_result = state.db.user.create_user(data).await;
    let create = match create_result {
        Ok(c) => c,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let user_id = change_insertoneresult_into_object_id(create);
    let get_user = state
        .db
        .user
        .get_user_by_id(user_id)
        .await
        .map_err(|e| e.to_string())?;

    let user_session = create_user_session(get_user.clone(), state.clone()).await?;
    create_user_account(state, user_id, user_session.id.unwrap()).await?;

    Ok(UserSessionModel::format(user_session))
}

pub async fn user_login(
    state: Arc<AppState>,
    data: UserLoginModel,
) -> Result<UserSessionModelGet, String> {
    if data.provider != AccountProviders::Credentials {
        return Err("Invalid login provider".to_string());
    }

    let email = data.email.ok_or("Email is required".to_string())?;
    let user = state
        .db
        .user
        .get_user_by_email(email)
        .await
        .map_err(|e| e.to_string())?;

    let password = data.password.ok_or("Password is required".to_string())?;
    let stored_hash = user
        .password
        .clone()
        .ok_or("Password not set for user".to_string())?;
    let salt = user.salt.ok_or("Salt missing for user".to_string())?;

    if !verify_password(&password, salt, &stored_hash) {
        return Err("Invalid email or password".to_string());
    }

    match get_session_by_user_id(state.clone(), &user.id.unwrap()).await {
        Ok(session) => {
            let updated_session = update_session_by_id(&state, &session.id.unwrap()).await?;

            let user_account_model = UserAccountPut {
                user_id: Some(user.id.unwrap().to_string()),
                provider: Some(AccountProviders::Credentials),
                expires_at: user_session_expires(),
                session_id: Some(session.id.unwrap().to_string()),
            };

            if let Err(e) = update_user_account(state.clone(), user_account_model).await {
                error!("Failed to update user account: {:#?}", e);
            }

            Ok(UserSessionModel::format(updated_session))
        }
        Err(_) => {
            let user_session = create_user_session(user.clone(), state.clone()).await?;
            create_user_account(state, user.id.unwrap(), user_session.id.unwrap()).await?;
            Ok(UserSessionModel::format(user_session))
        }
    }
}
