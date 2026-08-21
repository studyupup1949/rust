//! Device management functions.
//!
//! Functions for managing authorized devices via ACE API.
//!
//! Endpoints:
//! - GET /api/v1/auth/devices
//! - PATCH /api/v1/auth/devices/{id}
//! - DELETE /api/v1/auth/devices/{id}
//! - GET /api/v1/auth/device-limit

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};

use crate::errors::AceError;
use crate::types::{Device, DeviceLimit, RemoveDeviceResult, CORE_VERSION};

/// Build authenticated headers for device API calls.
fn build_auth_headers(token: &str) -> Result<HeaderMap, AceError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| AceError::Other(e.to_string()))?,
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&format!("ace-sdk-rust/{}", CORE_VERSION))
            .map_err(|e| AceError::Other(e.to_string()))?,
    );
    Ok(headers)
}

/// List all authorized devices for the current user.
///
/// GET /api/v1/auth/devices
pub async fn list_devices(server_url: &str, token: &str) -> Result<Vec<Device>, AceError> {
    let url = format!("{}/api/v1/auth/devices", server_url);
    let headers = build_auth_headers(token)?;
    let client = reqwest::Client::new();

    let response = client.get(&url).headers(headers).send().await?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().await.unwrap_or_default();
        return Err(AceError::from_http_response(status, &body));
    }

    let devices: Vec<Device> = response.json().await?;
    Ok(devices)
}

/// Rename a device for easier identification.
///
/// PATCH /api/v1/auth/devices/{device_id}
pub async fn rename_device(
    server_url: &str,
    token: &str,
    device_id: &str,
    new_name: &str,
) -> Result<Device, AceError> {
    let url = format!(
        "{}/api/v1/auth/devices/{}",
        server_url,
        urlencoding::encode(device_id)
    );
    let headers = build_auth_headers(token)?;
    let client = reqwest::Client::new();

    let body = serde_json::json!({ "device_name": new_name });

    let response = client
        .patch(&url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().await.unwrap_or_default();
        return Err(AceError::from_http_response(status, &body));
    }

    let device: Device = response.json().await?;
    Ok(device)
}

/// Remove a device and revoke all its sessions.
///
/// DELETE /api/v1/auth/devices/{device_id}
pub async fn remove_device(
    server_url: &str,
    token: &str,
    device_id: &str,
) -> Result<RemoveDeviceResult, AceError> {
    let url = format!(
        "{}/api/v1/auth/devices/{}",
        server_url,
        urlencoding::encode(device_id)
    );
    let headers = build_auth_headers(token)?;
    let client = reqwest::Client::new();

    let response = client.delete(&url).headers(headers).send().await?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().await.unwrap_or_default();
        return Err(AceError::from_http_response(status, &body));
    }

    let result: RemoveDeviceResult = response.json().await?;
    Ok(result)
}

/// Get device limit information for the current user.
///
/// GET /api/v1/auth/device-limit
pub async fn get_device_limit(server_url: &str, token: &str) -> Result<DeviceLimit, AceError> {
    let url = format!("{}/api/v1/auth/device-limit", server_url);
    let headers = build_auth_headers(token)?;
    let client = reqwest::Client::new();

    let response = client.get(&url).headers(headers).send().await?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().await.unwrap_or_default();
        return Err(AceError::from_http_response(status, &body));
    }

    let limit: DeviceLimit = response.json().await?;
    Ok(limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_devices_builds_correct_url() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/auth/devices")
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_body(
                r#"[
                    {
                        "device_id": "dev_abc123",
                        "device_name": "My MacBook",
                        "first_seen_at": "2024-01-01T00:00:00Z",
                        "last_seen_at": "2024-06-01T00:00:00Z",
                        "clients": ["cli", "claude-code"],
                        "is_current": true
                    }
                ]"#,
            )
            .create_async()
            .await;

        let devices = list_devices(&server.url(), "test_token").await.unwrap();
        mock.assert_async().await;
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, "dev_abc123");
        assert_eq!(devices[0].device_name, Some("My MacBook".to_string()));
        assert!(devices[0].is_current);
        assert_eq!(devices[0].clients, vec!["cli", "claude-code"]);
    }

    #[tokio::test]
    async fn test_list_devices_empty() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/auth/devices")
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;

        let devices = list_devices(&server.url(), "test_token").await.unwrap();
        mock.assert_async().await;
        assert!(devices.is_empty());
    }

    #[tokio::test]
    async fn test_rename_device() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/api/v1/auth/devices/dev_abc123")
            .match_header("authorization", "Bearer test_token")
            .match_body(mockito::Matcher::Json(
                serde_json::json!({"device_name": "New Name"}),
            ))
            .with_status(200)
            .with_body(
                r#"{
                    "device_id": "dev_abc123",
                    "device_name": "New Name",
                    "first_seen_at": "2024-01-01T00:00:00Z",
                    "last_seen_at": "2024-06-01T00:00:00Z",
                    "clients": ["cli"],
                    "is_current": false
                }"#,
            )
            .create_async()
            .await;

        let device = rename_device(&server.url(), "test_token", "dev_abc123", "New Name")
            .await
            .unwrap();
        mock.assert_async().await;
        assert_eq!(device.device_name, Some("New Name".to_string()));
    }

    #[tokio::test]
    async fn test_remove_device() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/api/v1/auth/devices/dev_abc123")
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_body(r#"{"revoked_count": 3}"#)
            .create_async()
            .await;

        let result = remove_device(&server.url(), "test_token", "dev_abc123")
            .await
            .unwrap();
        mock.assert_async().await;
        assert_eq!(result.revoked_count, 3);
    }

    #[tokio::test]
    async fn test_get_device_limit() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/auth/device-limit")
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_body(r#"{"current_devices": 2, "max_devices": 5, "is_custom": false}"#)
            .create_async()
            .await;

        let limit = get_device_limit(&server.url(), "test_token").await.unwrap();
        mock.assert_async().await;
        assert_eq!(limit.current_devices, 2);
        assert_eq!(limit.max_devices, 5);
        assert!(!limit.is_custom);
    }

    #[tokio::test]
    async fn test_list_devices_auth_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/auth/devices")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create_async()
            .await;

        let result = list_devices(&server.url(), "bad_token").await;
        mock.assert_async().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_device_not_found() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/api/v1/auth/devices/dev_nonexistent")
            .with_status(404)
            .with_body(r#"{"error": "Device not found"}"#)
            .create_async()
            .await;

        let result = remove_device(&server.url(), "test_token", "dev_nonexistent").await;
        mock.assert_async().await;
        assert!(result.is_err());
    }
}
