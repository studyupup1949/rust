#[cfg(feature = "mp")]
use crate::auth::AuthMethod;
#[cfg(feature = "mp")]
use crate::error::{AbpilotError, Result};
#[cfg(feature = "mp")]
use crate::models::*;
#[cfg(feature = "mp")]
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

#[cfg(feature = "mp")]
#[derive(Clone)]
pub struct MpClient {
    base_url: String,
    auth: Option<AuthMethod>,
    http_client: reqwest::Client,
}

#[cfg(feature = "mp")]
impl MpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            auth: None,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_auth(mut self, auth: AuthMethod) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn set_auth(&mut self, auth: AuthMethod) {
        self.auth = Some(auth);
    }

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        if let Some(auth) = &self.auth {
            match auth {
                AuthMethod::JwtToken(token) => {
                    let value = HeaderValue::from_str(&format!("Bearer {}", token))
                        .map_err(|_| AbpilotError::AuthError("Invalid token format".to_string()))?;
                    headers.insert(AUTHORIZATION, value);
                }
                AuthMethod::ApiKey(key) => {
                    let value = HeaderValue::from_str(key)
                        .map_err(|_| AbpilotError::AuthError("Invalid API key format".to_string()))?;
                    headers.insert("X-Api-Key", value);
                }
                _ => {}
            }
        }

        Ok(headers)
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        
        if status.is_success() {
            response.json::<T>().await.map_err(|e| e.into())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(AbpilotError::ApiError {
                status: status.as_u16(),
                message: error_text,
            })
        }
    }

    // ============ Authentication ============

    pub async fn send_verification_code(&self, email: &str) -> Result<()> {
        let url = format!("{}/auth/send-code", self.base_url);
        let body = SendCodeRequest {
            email: email.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<MessageResponse>(response).await?;
        Ok(())
    }

    pub async fn verify_code(&self, email: &str, code: &str) -> Result<AuthToken> {
        let url = format!("{}/auth/verify-code", self.base_url);
        let body = VerifyCodeRequest {
            email: email.to_string(),
            code: code.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<AuthToken>(response).await
    }

    // ============ API Key Management ============

    pub async fn create_api_key(&self, name: &str) -> Result<ApiKey> {
        let url = format!("{}/apikey", self.base_url);
        let body = CreateApiKeyRequest {
            name: name.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<ApiKey>(response).await
    }

    pub async fn delete_api_key(&self, apikey: &str) -> Result<()> {
        let url = format!("{}/apikey", self.base_url);
        let body = DeleteApiKeyRequest {
            apikey: apikey.to_string(),
        };

        let response = self.http_client
            .request(reqwest::Method::DELETE, &url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<MessageResponse>(response).await?;
        Ok(())
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        let url = format!("{}/apikey", self.base_url);

        let response = self.http_client
            .get(&url)
            .headers(self.build_headers()?)
            .send()
            .await?;

        let result = self.handle_response::<ListApiKeysResponse>(response).await?;
        Ok(result.apikeys)
    }

    // ============ App Management ============

    pub async fn create_app(&self, name: &str) -> Result<App> {
        let url = format!("{}/app", self.base_url);
        let body = CreateAppRequest {
            name: name.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<App>(response).await
    }

    pub async fn delete_app(&self, app_id: &str) -> Result<()> {
        let url = format!("{}/app", self.base_url);
        let body = DeleteAppRequest {
            app_id: app_id.to_string(),
        };

        let response = self.http_client
            .request(reqwest::Method::DELETE, &url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<MessageResponse>(response).await?;
        Ok(())
    }

    pub async fn list_apps(&self) -> Result<Vec<App>> {
        let url = format!("{}/app", self.base_url);

        let response = self.http_client
            .get(&url)
            .headers(self.build_headers()?)
            .send()
            .await?;

        let result = self.handle_response::<ListAppsResponse>(response).await?;
        Ok(result.apps)
    }

    pub async fn reset_app_secret(&self, app_id: &str) -> Result<App> {
        let url = format!("{}/app/reset-secret", self.base_url);
        let body = ResetAppSecretRequest {
            app_id: app_id.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<App>(response).await
    }

    pub async fn get_app_upload_urls(&self, app_id: &str, files: &[&str]) -> Result<Vec<String>> {
        let url = format!("{}/app/upload", self.base_url);
        let body = GetAppUploadUrlsRequest {
            app_id: app_id.to_string(),
            files: files.iter().map(|s| s.to_string()).collect(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        let result = self.handle_response::<GetUrlsResponse>(response).await?;
        Ok(result.urls)
    }

    pub async fn get_app_download_urls(&self, app_id: &str, files: &[&str]) -> Result<Vec<String>> {
        let url = format!("{}/app/files", self.base_url);
        let body = GetAppDownloadUrlsRequest {
            app_id: app_id.to_string(),
            files: files.iter().map(|s| s.to_string()).collect(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        let result = self.handle_response::<GetUrlsResponse>(response).await?;
        Ok(result.urls)
    }

    // ============ World Management ============

    pub async fn create_world(&self, name: &str) -> Result<World> {
        let url = format!("{}/world", self.base_url);
        let body = CreateWorldRequest {
            name: name.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<World>(response).await
    }

    pub async fn delete_world(&self, world_id: &str) -> Result<()> {
        let url = format!("{}/world", self.base_url);
        let body = DeleteWorldRequest {
            world_id: world_id.to_string(),
        };

        let response = self.http_client
            .request(reqwest::Method::DELETE, &url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<MessageResponse>(response).await?;
        Ok(())
    }

    pub async fn list_worlds(&self) -> Result<Vec<World>> {
        let url = format!("{}/world", self.base_url);

        let response = self.http_client
            .get(&url)
            .headers(self.build_headers()?)
            .send()
            .await?;

        let result = self.handle_response::<ListWorldsResponse>(response).await?;
        Ok(result.worlds)
    }

    pub async fn get_world(&self, world_id: &str) -> Result<World> {
        let url = format!("{}/world/get", self.base_url);
        let body = GetWorldRequest {
            world_id: world_id.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<World>(response).await
    }

    pub async fn reset_world_secret(&self, world_id: &str) -> Result<World> {
        let url = format!("{}/world/reset-secret", self.base_url);
        let body = ResetWorldSecretRequest {
            world_id: world_id.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<World>(response).await
    }

    pub async fn get_world_upload_urls(&self, world_id: &str, files: &[&str]) -> Result<Vec<String>> {
        let url = format!("{}/world/upload", self.base_url);
        let body = GetWorldUploadUrlsRequest {
            world_id: world_id.to_string(),
            files: files.iter().map(|s| s.to_string()).collect(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        let result = self.handle_response::<GetUrlsResponse>(response).await?;
        Ok(result.urls)
    }

    pub async fn get_world_download_urls(&self, world_id: &str, files: &[&str]) -> Result<Vec<String>> {
        let url = format!("{}/world/files", self.base_url);
        let body = GetWorldDownloadUrlsRequest {
            world_id: world_id.to_string(),
            files: files.iter().map(|s| s.to_string()).collect(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_headers()?)
            .json(&body)
            .send()
            .await?;

        let result = self.handle_response::<GetUrlsResponse>(response).await?;
        Ok(result.urls)
    }
}
