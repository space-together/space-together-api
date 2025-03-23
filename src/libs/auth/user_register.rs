use std::sync::Arc;

use crate::{
    libs::{
        auth::user_session::create_user_session,
        functions::object_id::change_insertoneresult_into_object_id,
    },
    models::user_model::user_model_model::UserModelNew,
    AppState,
};

use super::user_session::Cookies;

pub async fn user_register(
    state: Arc<AppState>,
    mut cookies: impl Cookies,
    data: UserModelNew,
) -> Result<String, String> {
    let create = state
        .db
        .user
        .create_user(data)
        .await
        .map_err(|e| e.to_string())?;
    let user_id = change_insertoneresult_into_object_id(create);
    create_user_session(user_id, &mut cookies, state).await?;
    Ok("Account created".to_string())
}
