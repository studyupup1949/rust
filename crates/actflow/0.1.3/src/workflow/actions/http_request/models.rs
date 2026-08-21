use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationType {
    NoAuth,
    ApiKey(ApiKeyType),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyType {
    Basic,
    Bearer,
    Custom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr)]
pub enum HttpRequestMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum ContentType {
    #[serde(rename(deserialize = "none"))]
    None,
    #[serde(rename(deserialize = "text"))]
    Text,
    #[serde(rename(deserialize = "html"))]
    Html,
    #[default]
    #[serde(rename(deserialize = "json"))]
    Json,
    #[serde(rename(deserialize = "urlencoded"))]
    UrlEncoded,
    #[serde(rename(deserialize = "form-data"))]
    FormData,
    #[serde(rename(deserialize = "binary"))]
    Binary,
    #[serde(rename(deserialize = "image"))]
    Image,
    #[serde(rename(deserialize = "video"))]
    Video,
    #[serde(rename(deserialize = "audio"))]
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    pub auth_type: AuthorizationType,
    pub api_key: Option<String>,
    pub header: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpBody {
    pub content_type: ContentType,
    pub data: Option<JsonValue>,
}
