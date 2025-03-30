use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    libs::{
        auth::{
            user_register::oauth2_login,
            user_session::{create_verification_token, get_verification_token_by_state},
        },
        functions::characters_fn::generate_hashed_state,
    },
    models::{
        auth::session_model::UserSessionModelGet,
        user_model::user_model_model::{AccountProviders, UserModelNew},
    },
    AppState,
};

use super::oauth2_providers::{
    create_discord_oauth2_provider, OAuthClient, ParsedUser, TokenResponse,
};

use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

impl OAuthClient {
    pub fn create_auth_url(
        auth_url: &str,
        client_id: &str,
        redirect_url: &str,
        scopes: &[&str],
        state: &str,
        code_verifier: &str,
    ) -> String {
        format!(
        "{auth_url}?client_id={client_id}&redirect_uri={redirect_url}&response_type=code&scope={}&state={state}&code_challenge_method=S256&code_challenge={code_verifier}",
        scopes.join("+"),
    )
    }

    pub async fn fetch_user(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_url: &str,
    ) -> Result<ParsedUser, String> {
        let token_response = self
            .fetch_token(code, code_verifier, redirect_url)
            .await
            .map_err(|e| e.to_string())?;

        let client = Client::new();
        let response = client
            .get(&self.urls.user)
            .header(
                "Authorization",
                format!(
                    "{} {}",
                    token_response.token_type, token_response.access_token
                ),
            )
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let user_data: Value = response.json().await.map_err(|e| e.to_string())?;
        (self.user_info.parser)(user_data)
    }

    async fn fetch_token(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_url: &str,
    ) -> Result<TokenResponse, reqwest::Error> {
        let client = Client::new();
        let mut params = HashMap::new();
        params.insert("code", code);
        params.insert("redirect_uri", redirect_url);
        params.insert("grant_type", "authorization_code");
        params.insert("client_id", &self.client_id);
        params.insert("client_secret", &self.client_secret);
        params.insert("code_verifier", code_verifier);

        let response = client.post(&self.urls.token).form(&params).send().await?;

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Oauth2ProviderUrlsModel {
    pub url: String,
    pub state: String,
    pub code_verifier: String,
}

pub async fn oauth2_provider_url(
    state: &Arc<AppState>,
    provider: AccountProviders,
) -> Result<Oauth2ProviderUrlsModel, String> {
    match provider {
        AccountProviders::Credentials => Ok(Oauth2ProviderUrlsModel {
            url: "".to_string(),
            state: "".to_string(),
            code_verifier: "".to_string(),
        }),
        AccountProviders::Discord => {
            let data_state = generate_hashed_state(32);
            let code_verifier = generate_hashed_state(41);
            let discord_client = create_discord_oauth2_provider(&data_state, &code_verifier)?;
            let url = OAuthClient::create_auth_url(
                &discord_client.urls.auth,
                &discord_client.client_id,
                &discord_client.redirect_uri,
                &discord_client
                    .scopes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
                &discord_client.state,
                &discord_client.code_verifier,
            );
            match create_verification_token(state, &Some(data_state), &Some(code_verifier)).await {
                Ok(verification_token) => Ok(Oauth2ProviderUrlsModel {
                    url,
                    state: verification_token.state.unwrap_or("".to_string()),
                    code_verifier: discord_client.code_verifier,
                }),
                Err(e) => Err(e),
            }
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct FetchOauthTokenModel {
    pub provider: AccountProviders,
    pub code: String,
    pub state: String,
    pub code_verifier: Option<String>,
}

pub async fn fetch_oauth_token(
    state: &Arc<AppState>,
    token: FetchOauthTokenModel,
) -> Result<UserSessionModelGet, String> {
    match token.provider {
        AccountProviders::Credentials => {
            Err("The credentials provider is not supported".to_string())
        }

        AccountProviders::Discord => {
            let get_verification = get_verification_token_by_state(state, &token.state).await?;
            let discord_client = create_discord_oauth2_provider(
                get_verification.state.as_deref().ok_or("Missing state")?,
                get_verification
                    .code_verifier
                    .as_deref()
                    .ok_or("Missing code verifier")?,
            )?;
            let user_session = fetch_oauth_token_by_provider(
                state,
                &FetchOauthTokenModel {
                    provider: AccountProviders::Discord,
                    code: token.code,
                    state: token.state,
                    code_verifier: get_verification.code_verifier,
                },
                &discord_client,
            )
            .await?;
            Ok(user_session)
        }
    }
}

async fn fetch_oauth_token_by_provider(
    state: &Arc<AppState>,
    token: &FetchOauthTokenModel,
    provider: &OAuthClient,
) -> Result<UserSessionModelGet, String> {
    let oauth_user = OAuthClient::fetch_user(
        provider,
        &token.code,
        token
            .code_verifier
            .as_deref()
            .ok_or("Missing code verifier")?,
        &provider.redirect_uri,
    )
    .await
    .map_err(|e| e.to_string())?;

    let user_model = UserModelNew {
        name: oauth_user.name,
        email: oauth_user.email,
        password: None,
        provider: Some(AccountProviders::Discord),
        image: oauth_user.image,
        username: None,
    };

    let user = oauth2_login(state, &user_model).await?;
    Ok(user)
}
