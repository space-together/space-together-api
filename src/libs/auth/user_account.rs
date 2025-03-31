use std::sync::Arc;

use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    models::user_model::user_model_model::{
        AccountProviders, UserAccount, UserAccountNew, UserAccountPut,
    },
    AppState,
};

use super::user_session::user_session_expires;

pub async fn create_user_account(
    state: Arc<AppState>,
    user_id: ObjectId,
    session_id: ObjectId,
    provider: AccountProviders,
) -> Result<UserAccount, String> {
    let user_account = UserAccountNew {
        user_id: user_id.to_string(),
        provider,
        session_id: Some(session_id.to_string()),
        expires_at: user_session_expires(),
    };

    let id = state
        .db
        .user_account
        .create(
            UserAccount::new(user_account),
            Some("user_accounts".to_string()),
        )
        .await
        .map_err(|e| e.to_string())?;
    let account = state
        .db
        .user_account
        .get_one_by_id(id, Some("user_account".to_string()))
        .await
        .map_err(|e| e.to_string())?;

    Ok(account)
}

pub async fn update_user_account(
    state: Arc<AppState>,
    data: UserAccountPut,
) -> Result<UserAccount, String> {
    let update = state
        .db
        .user_account
        .collection
        .find_one_and_update(
            doc! {"user_id" : &data.user_id},
            doc! {"$set" : UserAccount::put(data)},
        )
        .await
        .map_err(|e| e.to_string())?;
    match update {
        Some(d) => Ok(d),
        None => Err("Account not found, please try other account".to_string()),
    }
}

pub async fn update_user_account_expires(
    state: &Arc<AppState>,
    user_id: &ObjectId,
) -> Result<UserAccount, String> {
    let update = state
        .db
        .user_account
        .collection
        .find_one_and_update(
            doc! {"user_id" : &user_id},
            doc! {"$set" : UserAccount::put_expires()},
        )
        .await
        .map_err(|e| e.to_string())?;
    match update {
        Some(d) => Ok(d),
        None => Err("Account not found, please try other account".to_string()),
    }
}
