use std::collections::HashMap;
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

use super::user_session::{Cookie, CookieOptions, Cookies, SESSION_EXPIRATION_SECONDS};

#[derive(Debug, Default)]
pub struct FakeCookies {
    store: HashMap<String, (String, CookieOptions)>,
}

impl Cookies for FakeCookies {
    fn set(&mut self, key: String, value: String, options: CookieOptions) {
        self.store.insert(key, (value, options));
    }

    fn get(&self, key: &str) -> Option<Cookie> {
        self.store.get(key).map(|(value, _)| Cookie {
            name: key.to_string(),
            value: value.clone(),
        })
    }

    fn delete(&mut self, key: &str) {
        self.store.remove(key);
    }
}

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

    // Create a fake cookie store inside the function
    let mut cookies = FakeCookies::default();

    // Set a cookie
    cookies.set(
        "session".to_string(),
        "abc123".to_string(),
        CookieOptions {
            secure: Some(true),
            http_only: Some(true),
            same_site: Some("strict".to_string()),
            expires: Some(1712345678),
        },
    );

    let user_session = create_user_session(user_id, &mut cookies, state.clone()).await?;

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
