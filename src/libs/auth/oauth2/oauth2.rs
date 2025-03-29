use reqwest::Client;

use crate::libs::auth::user_session::generate_token;

use super::oauth2_providers::{OAuthClient, ParsedUser, TokenResponse};

use serde_json::Value;
use std::collections::HashMap;

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
