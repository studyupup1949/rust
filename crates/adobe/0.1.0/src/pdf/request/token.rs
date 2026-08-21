//! https://developer.adobe.com/document-services/docs/apis/#tag/Generate-Token/operation/authentication.generatetoken

use std::collections::HashMap;
use std::fmt::Debug;

use reqwest::{Client, Url};

use serde::{Deserialize, Serialize};

use crate::{ApiHttpRequest, Error, Result};

#[derive(Clone, Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("access_token", &"***REDACTED***")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenParams {
    pub client_id: String,
    pub client_secret: String,
}

impl ApiHttpRequest for Token {
    type Response = Token;
    type Params = TokenParams;

    async fn send(base_url: &Url, params: Self::Params) -> Result<Self::Response> {
        let url = base_url.join("/token")?;
        let mut form = HashMap::new();

        form.insert("client_id", params.client_id);
        form.insert("client_secret", params.client_secret);

        let client = Client::new();
        let resp = client.post(url).form(&form).send().await?;

        if resp.status().is_success() {
            let token: Self = resp.json().await?;
            return Ok(token);
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        Err(Error::ApiError(format!(
            "Failed to get token: {} - {}",
            status, text
        )))
    }
}
