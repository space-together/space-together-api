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
    let create_result = state.db.user.create_user(data).await;
    let user_id = match create_result {
        Ok(i) => change_insertoneresult_into_object_id(i),
        Err(e) => {
            return Err(e.to_string());
        }
    };

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
        user_session.id.expect("Expect user id but is not fount"),
        provider
            .clone()
            .map_or(AccountProviders::Credentials, |f| f),
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
    let salt = user.salt.ok_or("Salt missing for user".to_string())?;

    if !verify_password(&password, salt, &stored_hash) {
        return Err("Invalid email or password".to_string());
    }

    get_user_session_if_exit(&state, &user, &None).await
}

pub async fn oauth2_login(
    state: &Arc<AppState>,
    data: &UserModelNew,
) -> Result<UserSessionModelGet, String> {
    let user = state.db.user.get_user_by_email(data.email.clone()).await;
    match user {
        Ok(res) => {
            // update user account
            get_user_session_if_exit(state, &res, &data.provider).await
        }
        Err(_) => {
            let mut create_user =
                user_register(state.clone(), data.clone(), &data.provider).await?;
            create_user.redirect = Some(true);
            Ok(create_user)
        } //user not exit
    }
}

async fn get_user_session_if_exit(
    state: &Arc<AppState>,
    user: &UserModel,
    provider: &Option<AccountProviders>,
) -> Result<UserSessionModelGet, String> {
    match get_session_by_user_id(
        state.clone(),
        &user.id.expect("Expect user id but is not fount"),
    )
    .await
    {
        Ok(session) => {
            let updated_session =
                update_session_by_id(state, &session.id.expect("Expect user id but is not fount"))
                    .await?;

            let user_account_model = UserAccountPut {
                user_id: Some(
                    user.id
                        .expect("Expect user id but is not fount")
                        .to_string(),
                ),
                provider: provider.clone(),
                expires_at: user_session_expires(),
                session_id: Some(
                    session
                        .id
                        .expect("Expect user id but is not fount")
                        .to_string(),
                ),
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
                user.id.expect("Expect user id but is not fount"),
                user_session.id.expect("Expect user id but is not fount"),
                provider
                    .clone()
                    .map_or(AccountProviders::Credentials, |f| f),
            )
            .await?;
            Ok(UserSessionModel::format(user_session))
        }
    }
}
