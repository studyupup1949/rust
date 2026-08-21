use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue};
use serde::{Deserialize, Serialize};

use crate::{
    ActflowError, Result,
    common::Vars,
    runtime::Context,
    workflow::{
        actions::{Action, ActionOutput, ActionType},
        node::NodeId,
        template,
    },
};

use super::models::*;

const STATUS_CODE_KEY: &str = "status_code";
const BODY_KEY: &str = "body";
const HEADERS_KEY: &str = "headers";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpRequestAction {
    url: String,
    auth: AuthorizationConfig,
    method: HttpRequestMethod,
    headers: HashMap<String, String>,
    params: HashMap<String, String>,
    body: HttpBody,
    // http request timeout in milliseconds
    timeout: u64,
}

impl HttpRequestAction {
    /// Apply authorization headers based on auth config
    fn apply_auth_headers(
        &self,
        headers: &mut HeaderMap,
    ) -> Result<()> {
        match &self.auth.auth_type {
            AuthorizationType::NoAuth => {}
            AuthorizationType::ApiKey(api_key_type) => {
                let api_key = self.auth.api_key.as_ref().ok_or_else(|| ActflowError::Action("api_key is required for api-key authorization".to_string()))?;

                // Default header name is "Authorization"
                let header_name = self.auth.header.as_deref().unwrap_or("Authorization");
                let header_key: HeaderName = header_name.parse().map_err(|err: reqwest::header::InvalidHeaderName| ActflowError::Runtime(err.to_string()))?;

                let header_value = match api_key_type {
                    ApiKeyType::Bearer => format!("Bearer {}", api_key),
                    ApiKeyType::Basic => {
                        let encoded = if api_key.contains(':') {
                            STANDARD.encode(api_key.as_bytes())
                        } else {
                            api_key.clone()
                        };
                        format!("Basic {}", encoded)
                    }
                    ApiKeyType::Custom => api_key.clone(),
                };

                headers.insert(
                    header_key,
                    header_value.parse().map_err(|err: InvalidHeaderValue| ActflowError::Runtime(err.to_string()))?,
                );
            }
        }
        Ok(())
    }

    fn build_request(
        &self,
        ctx: Arc<Context>,
    ) -> Result<reqwest::RequestBuilder> {
        // Resolve URL template
        let resolved_url = template::resolve_template(&ctx, &self.url)?;

        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("accept"), HeaderValue::from_static("*/*"));

        // Apply authorization headers
        self.apply_auth_headers(&mut headers)?;

        for (key, value) in &self.headers {
            // Resolve header value template
            let resolved_value = template::resolve_template(&ctx, value)?;
            headers.insert(
                key.parse::<HeaderName>().map_err(|err| ActflowError::Runtime(err.to_string()))?,
                resolved_value.parse().map_err(|err: InvalidHeaderValue| ActflowError::Runtime(err.to_string()))?,
            );
        }

        let mut query = Vec::new();
        for (key, value) in &self.params {
            // Resolve query param value template
            let resolved_value = template::resolve_template(&ctx, value)?;
            query.push((key.clone(), resolved_value));
        }

        let client = reqwest::Client::new();

        let mut request = client
            .request(
                self.method.as_ref().parse().map_err(|_| ActflowError::Runtime(format!("invalid method '{:?}'", self.method)))?,
                &resolved_url,
            )
            .headers(headers)
            .query(&query);

        match self.body.content_type {
            ContentType::Text | ContentType::Html => {
                if let Some(text) = &self.body.data {
                    let data = text.as_str().ok_or(ActflowError::Action("content-type did not match the body content".to_string()))?;
                    // Resolve template in text body
                    let resolved_data = template::resolve_template(&ctx, data)?;
                    request = request.body::<String>(resolved_data);
                }
            }
            ContentType::Json => {
                if let Some(json) = &self.body.data {
                    // Resolve templates in JSON body recursively
                    let resolved_json = template::resolve_json_value(&ctx, json)?;
                    let body = serde_json::to_vec(&resolved_json)?;
                    request = request.body(body);
                }
            }
            ContentType::FormData | ContentType::UrlEncoded => {
                if let Some(form) = &self.body.data {
                    // Resolve templates in form data
                    let resolved_form = template::resolve_json_value(&ctx, form)?;
                    let data = resolved_form.as_object().ok_or(ActflowError::Action("content-type did not match the body content".to_string()))?;
                    request = request.form(data);
                }
            }
            ContentType::Binary | ContentType::Image | ContentType::Video | ContentType::Audio => {
                if let Some(value) = &self.body.data {
                    let data = value.as_str().ok_or(ActflowError::Action("content-type did not match the body content".to_string()))?;
                    let data = STANDARD.decode(data).map_err(|err| ActflowError::Action(err.to_string()))?;
                    request = request.body(data);
                }
            }
            _ => {}
        }

        // Set timeout
        request = request.timeout(Duration::from_millis(self.timeout));

        Ok(request)
    }
}

#[async_trait]
#[typetag::serde]
impl Action for HttpRequestAction {
    fn create(params: serde_json::Value) -> Result<Self> {
        jsonschema::validate(&params, &Self::schema())?;
        let action = serde_json::from_value::<Self>(params)?;
        Ok(action)
    }

    fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["url", "method", "auth", "headers", "params", "body", "timeout"],
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Request URL, supports template variables like {{#nodeId.key#}}"
                },
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"],
                    "description": "HTTP request method"
                },
                "auth": {
                    "type": "object",
                    "required": ["auth_type"],
                    "properties": {
                        "auth_type": {
                            "oneOf": [
                                { "const": "no_auth" },
                                {
                                    "type": "object",
                                    "properties": {
                                        "api_key": {
                                            "type": "string",
                                            "enum": ["basic", "bearer", "custom"]
                                        }
                                    }
                                }
                            ]
                        },
                        "api_key": { "type": ["string", "null"] },
                        "header": { "type": ["string", "null"] }
                    }
                },
                "headers": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "HTTP headers, values support template variables"
                },
                "params": {
                    "type": "object",
                    "additionalProperties": { "type": "string" },
                    "description": "Query parameters, values support template variables"
                },
                "body": {
                    "type": "object",
                    "required": ["content_type"],
                    "properties": {
                        "content_type": {
                            "type": "string",
                            "enum": ["none", "text", "html", "json", "urlencoded", "form-data", "binary", "image", "video", "audio"]
                        },
                        "data": {
                            "description": "Request body data, supports template variables in string values"
                        }
                    }
                },
                "timeout": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Request timeout in milliseconds"
                }
            }
        })
    }

    fn action_type(&self) -> ActionType {
        ActionType::HttpRequest
    }

    async fn run(
        &self,
        ctx: Arc<Context>,
        _nid: NodeId,
    ) -> Result<ActionOutput> {
        let mut outputs = Vars::new();

        let request = self.build_request(ctx.clone())?;
        let res = request.send().await.map_err(|err| ActflowError::Runtime(format!("Http error: {}", err)))?;

        outputs.insert(STATUS_CODE_KEY.to_string(), res.status().as_u16().into());

        // Convert HeaderMap to a serializable HashMap
        let headers_map: HashMap<String, String> = res.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
        outputs.insert(
            HEADERS_KEY.to_string(),
            serde_json::to_value(headers_map).map_err(|err| ActflowError::Runtime(err.to_string()))?,
        );

        outputs.insert(
            BODY_KEY.to_string(),
            res.text().await.map_err(|err| ActflowError::Runtime(err.to_string()))?.into(),
        );

        Ok(ActionOutput::success(outputs))
    }
}
