use serde::{Deserialize, Serialize};

// ============ MP Models ============

#[cfg(feature = "mp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub created_at: i64,
}

#[cfg(feature = "mp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub apikey: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub app_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub world_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

// ============ APP Models ============

#[cfg(feature = "app")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
    pub value: i64,
}

#[cfg(feature = "app")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldNode {
    pub world_id: String,
    pub base_url: String,
    pub tags: String,
}

#[cfg(feature = "app")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub world_id: String,
    pub info: serde_json::Value,
}

#[cfg(feature = "app")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceToken {
    pub token: String,
    pub items: Vec<WorldNodeInfo>,
}

#[cfg(feature = "app")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldNodeInfo {
    pub base_url: String,
    pub tags: String,
}

// ============ Internal Request/Response Models ============

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct SendCodeRequest {
    pub email: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct CreateApiKeyRequest {
    pub name: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct DeleteApiKeyRequest {
    pub apikey: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Deserialize)]
pub(crate) struct ListApiKeysResponse {
    pub apikeys: Vec<ApiKey>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct CreateAppRequest {
    pub name: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct DeleteAppRequest {
    pub app_id: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Deserialize)]
pub(crate) struct ListAppsResponse {
    pub apps: Vec<App>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct ResetAppSecretRequest {
    pub app_id: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct GetAppUploadUrlsRequest {
    pub app_id: String,
    pub files: Vec<String>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct GetAppDownloadUrlsRequest {
    pub app_id: String,
    pub files: Vec<String>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Deserialize)]
pub(crate) struct GetUrlsResponse {
    pub urls: Vec<String>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct CreateWorldRequest {
    pub name: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct DeleteWorldRequest {
    pub world_id: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Deserialize)]
pub(crate) struct ListWorldsResponse {
    pub worlds: Vec<World>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct GetWorldRequest {
    pub world_id: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct ResetWorldSecretRequest {
    pub world_id: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct GetWorldUploadUrlsRequest {
    pub world_id: String,
    pub files: Vec<String>,
}

#[cfg(feature = "mp")]
#[derive(Debug, Serialize)]
pub(crate) struct GetWorldDownloadUrlsRequest {
    pub world_id: String,
    pub files: Vec<String>,
}

#[cfg(feature = "app")]
#[derive(Debug, Serialize)]
pub(crate) struct ListAssetsRequest {
    pub device_id: String,
    pub world_id: String,
}

#[cfg(feature = "app")]
#[derive(Debug, Deserialize)]
pub(crate) struct ListAssetsResponse {
    pub assets: Vec<Asset>,
}

#[cfg(feature = "app")]
#[derive(Debug, Serialize)]
pub(crate) struct GetAssetRequest {
    pub device_id: String,
    pub world_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
}

#[cfg(feature = "app")]
#[derive(Debug, Serialize)]
pub(crate) struct AddAssetRequest {
    pub device_id: String,
    pub world_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
    pub delta: i64,
}

#[cfg(feature = "app")]
#[derive(Debug, Serialize)]
pub(crate) struct UpdateWorldNodeRequest {
    pub world_id: String,
    pub base_url: String,
    pub tags: String,
}

#[cfg(feature = "app")]
#[derive(Debug, Serialize)]
pub(crate) struct DeleteWorldNodeRequest {
    pub world_id: String,
    pub base_url: String,
}

#[cfg(feature = "app")]
#[derive(Debug, Serialize)]
pub(crate) struct CreateDeviceTokenRequest {
    pub world_id: String,
    pub device_id: String,
    pub info: serde_json::Value,
    pub ttl: u64,
}

#[cfg(feature = "app")]
#[derive(Debug, Serialize)]
pub(crate) struct GetDeviceInfoRequest {
    pub world_id: String,
    pub token: String,
}

#[cfg(feature = "mp")]
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct MessageResponse {
    pub message: String,
}

#[cfg(feature = "app")]
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct ErrorResponse {
    pub error: String,
}
