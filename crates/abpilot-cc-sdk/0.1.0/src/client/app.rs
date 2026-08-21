#[cfg(feature = "app")]
use crate::auth::SignatureGenerator;
#[cfg(feature = "app")]
use crate::error::{AbpilotError, Result};
#[cfg(feature = "app")]
use crate::models::*;
#[cfg(feature = "app")]
use reqwest::header::{HeaderMap, HeaderValue};

#[cfg(feature = "app")]
#[derive(Clone)]
pub struct AppClient {
    base_url: String,
    http_client: reqwest::Client,
}

#[cfg(feature = "app")]
impl AppClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http_client: reqwest::Client::new(),
        }
    }

    fn build_app_signature_headers(&self, app_id: &str, app_secret: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let generator = SignatureGenerator::new(app_secret);
        let (signature, timestamp) = generator.generate_app_signature(app_id);

        headers.insert(
            "X-App-Id",
            HeaderValue::from_str(app_id)
                .map_err(|_| AbpilotError::InvalidRequest("Invalid app_id".to_string()))?,
        );
        headers.insert(
            "X-Signature",
            HeaderValue::from_str(&signature)
                .map_err(|_| AbpilotError::SignatureError)?,
        );
        headers.insert(
            "X-Timestamp",
            HeaderValue::from_str(&timestamp.to_string())
                .map_err(|_| AbpilotError::InvalidRequest("Invalid timestamp".to_string()))?,
        );

        Ok(headers)
    }

    fn build_world_signature_headers(&self, world_id: &str, world_secret: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let generator = SignatureGenerator::new(world_secret);
        let (signature, timestamp) = generator.generate_world_signature(world_id);

        headers.insert(
            "X-World-Id",
            HeaderValue::from_str(world_id)
                .map_err(|_| AbpilotError::InvalidRequest("Invalid world_id".to_string()))?,
        );
        headers.insert(
            "X-Signature",
            HeaderValue::from_str(&signature)
                .map_err(|_| AbpilotError::SignatureError)?,
        );
        headers.insert(
            "X-Timestamp",
            HeaderValue::from_str(&timestamp.to_string())
                .map_err(|_| AbpilotError::InvalidRequest("Invalid timestamp".to_string()))?,
        );

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
            
            // Check for specific error messages
            if error_text.contains("Insufficient balance") {
                return Err(AbpilotError::InsufficientBalance);
            }
            if error_text.contains("Token expired") {
                return Err(AbpilotError::TokenExpired);
            }
            if error_text.contains("Token not found") {
                return Err(AbpilotError::NotFound("Token not found".to_string()));
            }
            if error_text.contains("Asset not found") {
                return Err(AbpilotError::NotFound("Asset not found".to_string()));
            }
            
            Err(AbpilotError::ApiError {
                status: status.as_u16(),
                message: error_text,
            })
        }
    }

    // ============ Asset Management ============

    pub async fn list_assets(
        &self,
        app_id: &str,
        app_secret: &str,
        device_id: &str,
        world_id: &str,
    ) -> Result<Vec<Asset>> {
        let url = format!("{}/assets/list", self.base_url);
        let body = ListAssetsRequest {
            device_id: device_id.to_string(),
            world_id: world_id.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_app_signature_headers(app_id, app_secret)?)
            .json(&body)
            .send()
            .await?;

        let result = self.handle_response::<ListAssetsResponse>(response).await?;
        Ok(result.assets)
    }

    pub async fn get_asset(
        &self,
        app_id: &str,
        app_secret: &str,
        device_id: &str,
        world_id: &str,
        asset_type: &str,
        asset_id: &str,
    ) -> Result<Asset> {
        let url = format!("{}/assets/get", self.base_url);
        let body = GetAssetRequest {
            device_id: device_id.to_string(),
            world_id: world_id.to_string(),
            r#type: asset_type.to_string(),
            id: asset_id.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_app_signature_headers(app_id, app_secret)?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<Asset>(response).await
    }

    pub async fn add_asset(
        &self,
        world_id: &str,
        world_secret: &str,
        device_id: &str,
        asset_type: &str,
        asset_id: &str,
        delta: i64,
    ) -> Result<Asset> {
        let url = format!("{}/assets/add", self.base_url);
        let body = AddAssetRequest {
            device_id: device_id.to_string(),
            world_id: world_id.to_string(),
            r#type: asset_type.to_string(),
            id: asset_id.to_string(),
            delta,
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_world_signature_headers(world_id, world_secret)?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<Asset>(response).await
    }

    // ============ World Node Management ============

    pub async fn update_world_node(
        &self,
        world_id: &str,
        world_secret: &str,
        base_url: &str,
        tags: &str,
    ) -> Result<WorldNode> {
        let url = format!("{}/world/node/update", self.base_url);
        let body = UpdateWorldNodeRequest {
            world_id: world_id.to_string(),
            base_url: base_url.to_string(),
            tags: tags.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_world_signature_headers(world_id, world_secret)?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<WorldNode>(response).await
    }

    pub async fn delete_world_node(
        &self,
        world_id: &str,
        world_secret: &str,
        base_url: &str,
    ) -> Result<()> {
        let url = format!("{}/world/node/delete", self.base_url);
        let body = DeleteWorldNodeRequest {
            world_id: world_id.to_string(),
            base_url: base_url.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_world_signature_headers(world_id, world_secret)?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<MessageResponse>(response).await?;
        Ok(())
    }

    // ============ Device Management ============

    pub async fn create_device_token(
        &self,
        app_id: &str,
        app_secret: &str,
        world_id: &str,
        device_id: &str,
        info: serde_json::Value,
        ttl: u64,
    ) -> Result<DeviceToken> {
        let url = format!("{}/world/device/create", self.base_url);
        let body = CreateDeviceTokenRequest {
            world_id: world_id.to_string(),
            device_id: device_id.to_string(),
            info,
            ttl,
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_app_signature_headers(app_id, app_secret)?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<DeviceToken>(response).await
    }

    pub async fn get_device_info(
        &self,
        world_id: &str,
        world_secret: &str,
        token: &str,
    ) -> Result<Device> {
        let url = format!("{}/world/device/get", self.base_url);
        let body = GetDeviceInfoRequest {
            world_id: world_id.to_string(),
            token: token.to_string(),
        };

        let response = self.http_client
            .post(&url)
            .headers(self.build_world_signature_headers(world_id, world_secret)?)
            .json(&body)
            .send()
            .await?;

        self.handle_response::<Device>(response).await
    }
}

// Add MessageResponse for app feature
#[cfg(feature = "app")]
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct MessageResponse {
    message: String,
}
