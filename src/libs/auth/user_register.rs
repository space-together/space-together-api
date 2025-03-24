use std::sync::Arc;

use crate::{
    libs::{
        auth::user_session::create_user_session,
        functions::object_id::change_insertoneresult_into_object_id,
    },
    models::user_model::user_model_model::{
        AccountProviders, UserAccount, UserAccountNew, UserModelNew,
    },
    AppState,
};

use super::user_session::SESSION_EXPIRATION_SECONDS;

pub async fn user_register(
    state: Arc<AppState>,
    data: UserModelNew,
) -> Result<UserAccount, String> {
    let create = state
        .db
        .user
        .create_user(data)
        .await
        .map_err(|e| e.to_string())?;

    let user_id = change_insertoneresult_into_object_id(create);

    let get_user = state
        .db
        .user
        .get_user_by_id(user_id)
        .await
        .map_err(|e| e.to_string())?;

    let user_session = create_user_session(get_user, state.clone()).await?;

    let user_account = UserAccountNew {
        user_id: user_id.to_string(),
        provider: AccountProviders::Credentials,
        session_id: user_session.id.map(|i| i.to_string()),
        expires_at: Some(SESSION_EXPIRATION_SECONDS),
    };

    let account = state
        .db
        .user_account
        .create(
            UserAccount::new(user_account),
            Some("user_accounts".to_string()),
        )
        .await
        .map_err(|e| e.to_string())?;

    let get_account = state
        .db
        .user_account
        .get_one_by_id(account, Some("user_account".to_string()))
        .await
        .map_err(|e| e.to_string())?;

    Ok(get_account)
}
