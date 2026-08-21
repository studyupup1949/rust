use std::time::Duration;

use bytes::Bytes;
use futures::Stream;
use reqwest::Response;
use tokio_stream::StreamExt;

use crate::error::RouterError;

/// Create a shared HTTP client with sensible defaults.
pub fn create_client(timeout: Duration) -> Result<reqwest::Client, RouterError> {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(RouterError::Http)
}

/// Check an HTTP response for errors and return the body as JSON.
pub async fn handle_json_response(
    response: Response,
    provider: &str,
) -> Result<serde_json::Value, RouterError> {
    let status = response.status().as_u16();

    if status == 401 || status == 403 {
        let body = response.text().await.unwrap_or_default();
        return Err(RouterError::Auth {
            provider: provider.to_string(),
            message: body,
        });
    }

    if status == 429 {
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);
        return Err(RouterError::RateLimited {
            provider: provider.to_string(),
            retry_after,
        });
    }

    if !response.status().is_success() {
        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or(serde_json::Value::Null);

        let message = body["error"]["message"]
            .as_str()
            .or(body["error"].as_str())
            .or(body["message"].as_str())
            .unwrap_or("Unknown error")
            .to_string();

        return Err(RouterError::api_error(provider, status, message, body));
    }

    response.json().await.map_err(RouterError::Http)
}

/// Parse a Server-Sent Events stream into individual data payloads.
///
/// Filters out comments, empty lines, and non-data events.
/// Yields the raw string content of each `data:` line.
pub fn sse_stream(
    response: Response,
) -> impl Stream<Item = Result<String, RouterError>> + Send {
    let byte_stream = response.bytes_stream();

    // Buffer for incomplete lines across chunks
    let stream = async_stream::stream! {
        let mut buffer = String::new();
        let mut byte_stream = std::pin::pin!(byte_stream);

        while let Some(chunk) = byte_stream.next().await {
            let chunk: Bytes = chunk.map_err(RouterError::Http)?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // Process complete lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim_end_matches('\r').to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        return;
                    }
                    yield Ok(data.to_string());
                } else if let Some(data) = line.strip_prefix("data:") {
                    if data == "[DONE]" {
                        return;
                    }
                    yield Ok(data.to_string());
                }
            }
        }

        // Process any remaining data in the buffer
        if !buffer.is_empty() {
            let line = buffer.trim();
            if let Some(data) = line.strip_prefix("data: ") {
                if data != "[DONE]" {
                    yield Ok(data.to_string());
                }
            } else if let Some(data) = line.strip_prefix("data:") {
                if data != "[DONE]" {
                    yield Ok(data.to_string());
                }
            }
        }
    };

    stream
}
