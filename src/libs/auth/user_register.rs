use std::sync::Arc;
use tracing::error;

use crate::{
    libs::{
        auth::{
            user_account::update_user_account,
            user_session::{create_user_session, get_session_by_user_id, update_session_by_id},
        },
        functions::{
            characters_fn::verify_password, object_id::change_insertoneresult_into_object_id,
        },
    },
    models::{
        auth::session_model::{UserSessionModel, UserSessionModelGet},
        user_model::user_model_model::{
            AccountProviders, UserAccountPut, UserLoginModel, UserModel, UserModelNew,
        },
    },
    AppState,
};

use super::{user_account::create_user_account, user_session::user_session_expires};

pub async fn user_register(
    state: Arc<AppState>,
    data: UserModelNew,
    provider: &Option<AccountProviders>,
) -> Result<UserSessionModelGet, String> {
    let user_id = state
        .db
        .user
        .create_user(data)
        .await
        .map_err(|e| e.to_string())
        .map(change_insertoneresult_into_object_id)?;

    let get_user = state
        .db
        .user
        .get_user_by_id(user_id)
        .await
        .map_err(|e| e.to_string())?;

    let user_session = create_user_session(get_user.clone(), state.clone()).await?;

    create_user_account(
        state,
        user_id,
        user_session.id.ok_or("User ID not found")?,
        provider.clone().unwrap_or(AccountProviders::Credentials),
    )
    .await?;

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
    let salt = user
        .salt
        .ok_or("Your not allowed to login using password, try other account".to_string())?;

    if !verify_password(&password, salt, &stored_hash) {
        return Err("Invalid email or password".to_string());
    }

    get_user_session_if_exists(&state, &user, &None).await
}

pub async fn oauth2_login(
    state: &Arc<AppState>,
    data: &UserModelNew,
) -> Result<UserSessionModelGet, String> {
    match state.db.user.get_user_by_email(data.email.clone()).await {
        Ok(res) => get_user_session_if_exists(state, &res, &data.provider).await,
        Err(_) => {
            let mut create_user =
                user_register(state.clone(), data.clone(), &data.provider).await?;
            create_user.redirect = Some(true);
            Ok(create_user)
        }
    }
}

async fn get_user_session_if_exists(
    state: &Arc<AppState>,
    user: &UserModel,
    provider: &Option<AccountProviders>,
) -> Result<UserSessionModelGet, String> {
    match get_session_by_user_id(state.clone(), &user.id.ok_or("User ID not found")?).await {
        Ok(session) => {
            let updated_session =
                update_session_by_id(state, &session.id.ok_or("User ID not found")?).await?;

            let user_account_model = UserAccountPut {
                user_id: Some(user.id.ok_or("User ID not found")?.to_string()),
                provider: provider.clone(),
                expires_at: user_session_expires(),
                session_id: Some(session.id.ok_or("User ID not found")?.to_string()),
            };

            if let Err(e) = update_user_account(state.clone(), user_account_model).await {
                error!("Failed to update user account: {:#?}", e);
            }

            Ok(updated_session)
        }
        Err(_) => {
            let user_session = create_user_session(user.clone(), state.clone()).await?;
            create_user_account(
                state.clone(),
                user.id.ok_or("User ID not found")?,
                user_session.id.ok_or("User ID not found")?,
                provider.clone().unwrap_or(AccountProviders::Credentials),
            )
            .await?;
            Ok(UserSessionModel::format(user_session))
        }
    }
}
