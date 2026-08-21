// Allow unwrap in this module - HTTP client building with valid settings cannot fail
#![allow(clippy::unwrap_used)]

use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use std::time::Duration;
use crate::errors::Error;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct HttpTransport {
    pub base_url: String,
    pub client: reqwest::Client,
    pub timeout: Duration,
}

impl HttpTransport {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(base_url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap(),
            timeout,
        }
    }
}

#[async_trait]
impl crate::generated::api_methods::AccumulateRpc for HttpTransport {
    async fn rpc_call<TP: Serialize + Send + Sync, TR: DeserializeOwned>(
        &self, method: &str, params: &TP
    ) -> Result<TR, Error> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let res = self.client.post(&self.base_url)
            .json(&payload)
            .send().await
            .map_err(|e| Error::General(format!("Transport error: {}", e)))?;

        if !res.status().is_success() {
            return Err(Error::General(format!("HTTP status error: {}", res.status().as_u16())));
        }

        let v: serde_json::Value = res.json().await
            .map_err(|e| Error::General(format!("Failed to parse JSON response: {}", e)))?;

        if let Some(e) = v.get("error") {
            let code = e.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
            let message = e.get("message").and_then(|m| m.as_str()).unwrap_or("rpc error").to_string();
            return Err(Error::General(format!("RPC error {}: {}", code, message)));
        }

        let result = v.get("result").ok_or_else(|| Error::General("missing result".into()))?;
        let typed: TR = serde_json::from_value(result.clone())
            .map_err(|e| Error::General(format!("Failed to deserialize result: {}", e)))?;
        Ok(typed)
    }
}