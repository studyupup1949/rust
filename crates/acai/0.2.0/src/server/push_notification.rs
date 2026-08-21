use std::error::Error as StdError;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{UrlValidationError, validate_url};
use crate::types::{PushNotificationConfig, Task, TaskState, TaskStatusUpdateEvent};

/// Authentication token for push notifications
#[derive(Debug, Serialize, Deserialize)]
struct PushNotificationToken {
    /// Issued at timestamp (in seconds since epoch)
    iat: u64,
    /// SHA-256 hash of the request body
    request_body_sha256: String,
    /// Expiration timestamp (in seconds since epoch)
    /// Set to 5 minutes after issuance
    exp: u64,
}

/// Error type for push notification operations
#[derive(Debug)]
pub enum PushNotificationError {
    HttpError(reqwest::Error),
    SerializationError(serde_json::Error),
    TokenError(String),
    ValidationError(UrlValidationError),
    Other(String),
}

impl fmt::Display for PushNotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpError(e) => write!(f, "HTTP error: {}", e),
            Self::SerializationError(e) => write!(f, "Failed to serialize request body: {}", e),
            Self::TokenError(e) => write!(f, "Failed to generate token: {}", e),
            Self::ValidationError(e) => write!(f, "URL validation error: {}", e),
            Self::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl StdError for PushNotificationError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::HttpError(e) => Some(e),
            Self::SerializationError(e) => Some(e),
            Self::ValidationError(e) => Some(e),
            Self::TokenError(_) | Self::Other(_) => None,
        }
    }
}

impl From<reqwest::Error> for PushNotificationError {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpError(error)
    }
}

impl From<serde_json::Error> for PushNotificationError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError(error)
    }
}

/// Sender for push notifications
pub struct PushNotificationSender {
    client: Client,
    token_key: Hmac<Sha256>,
}

impl PushNotificationSender {
    /// Create a new push notification sender with the provided key
    ///
    /// # Arguments
    ///
    /// * `key` - A 32-byte array to use as the HMAC key for signing tokens
    pub fn new(key: &[u8]) -> Result<Self, PushNotificationError> {
        let token_key = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|e| PushNotificationError::TokenError(e.to_string()))?;

        Ok(Self {
            client: Client::new(),
            token_key,
        })
    }

    /// Verify that a notification URL is valid by sending a validation token
    ///
    /// This follows the same pattern as the Azure Event Grid webhook validation:
    /// 1. Generate a random validation token
    /// 2. Send a GET request to the URL with the token as a query parameter
    /// 3. The webhook should respond with the same token as the response body
    pub async fn verify_push_notification_url(
        &self,
        url: &str,
    ) -> Result<bool, PushNotificationError> {
        // First validate URL format
        validate_url(url)?;

        let validation_token = Uuid::new_v4().to_string();
        let url_with_query = format!("{}?validationToken={}", url, validation_token);

        let response = self
            .client
            .get(&url_with_query)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        // Use error_for_status to handle HTTP errors appropriately
        match response.error_for_status() {
            Ok(response) => {
                let response_text = response.text().await?;
                Ok(response_text == validation_token)
            }
            Err(_) => Ok(false), // Return false for non-success status codes
        }
    }

    /// Calculate SHA-256 hash of the request body
    ///
    /// Takes a pre-serialized JSON string to ensure consistency across different JSON parsers
    fn calculate_request_body_sha256(&self, json_string: &str) -> String {
        let digest = Sha256::digest(json_string.as_bytes());
        hex::encode(digest)
    }

    /// Send a push notification to the specified URL
    pub async fn send_push_notification(
        &self,
        config: &PushNotificationConfig,
        task: &Task,
    ) -> Result<(), PushNotificationError> {
        // Validate URL format first
        validate_url(&config.url)?;

        // Generate the event data (task status update)
        let event = TaskStatusUpdateEvent {
            id: task.id.clone(),
            status: task.status.clone(),
            final_status: matches!(
                task.status.state,
                TaskState::Completed | TaskState::Failed | TaskState::Canceled
            ),
            metadata: task.metadata.clone(),
        };

        // Serialize the event to ensure we use the same JSON for both the hash and the request
        let event_json = serde_json::to_string(&event)?;

        // Generate JWT token for authentication using the pre-serialized JSON
        let request_body_sha256 = self.calculate_request_body_sha256(&event_json);

        // Create token claims
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| PushNotificationError::TokenError(e.to_string()))?
            .as_secs();

        let claims = PushNotificationToken {
            iat: now,
            request_body_sha256,
            exp: now + 300, // 5 minutes expiration
        };

        // Sign the token
        let token = claims
            .sign_with_key(&self.token_key)
            .map_err(|e| PushNotificationError::TokenError(e.to_string()))?;

        // Send the notification with exactly the same JSON used for the hash
        let mut request_builder = self
            .client
            .post(&config.url)
            .timeout(Duration::from_secs(10))
            .header("Content-Type", "application/json")
            .body(event_json) // Use the pre-serialized JSON directly
            .header("Authorization", format!("Bearer {}", token));

        // Add custom token header if provided in config
        if let Some(ref token) = config.token {
            request_builder = request_builder.header("X-Notification-Token", token);
        }

        // Add additional authentication if provided in config
        if let Some(ref auth) = config.authentication {
            if let Some(ref creds) = auth.credentials {
                for scheme in &auth.schemes {
                    match scheme.to_lowercase().as_str() {
                        "bearer" => {
                            request_builder = request_builder
                                .header("Authorization", format!("Bearer {}", creds));
                        }
                        "basic" => {
                            request_builder =
                                request_builder.header("Authorization", format!("Basic {}", creds));
                        }
                        "apikey" => {
                            request_builder = request_builder.header("X-API-Key", creds);
                        }
                        _ => {
                            // Ignore unknown schemes
                        }
                    }
                }
            }
        }

        // Send the request and use error_for_status to handle HTTP errors
        let _response = request_builder.send().await?.error_for_status()?;

        // If we get here, the response was successful
        Ok(())
    }
}

/// Trait for task managers that support push notifications
#[async_trait::async_trait]
pub trait PushNotificationSupport: Send + Sync {
    /// Set push notification configuration for a task
    async fn set_push_notification_config(
        &self,
        task_id: &str,
        config: PushNotificationConfig,
    ) -> Result<(), PushNotificationError>;

    /// Get push notification configuration for a task
    async fn get_push_notification_config(
        &self,
        task_id: &str,
    ) -> Result<Option<PushNotificationConfig>, PushNotificationError>;

    /// Check if a task has push notification configuration
    async fn has_push_notification_config(
        &self,
        task_id: &str,
    ) -> Result<bool, PushNotificationError> {
        Ok(self.get_push_notification_config(task_id).await?.is_some())
    }

    /// Send a push notification for task status update
    async fn send_task_status_notification(&self, task: &Task)
    -> Result<(), PushNotificationError>;
}

/// Default implementation for push notification verification
#[derive(Default)]
pub struct PushNotificationVerifier {
    client: Client,
}

impl PushNotificationVerifier {
    /// Create a new push notification verifier
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Verify a webhook URL by sending a validation token
    pub async fn verify_push_notification_url(
        &self,
        url: &str,
    ) -> Result<bool, PushNotificationError> {
        // First validate URL format
        validate_url(url)?;

        let validation_token = Uuid::new_v4().to_string();
        let url_with_query = format!("{}?validationToken={}", url, validation_token);

        let response = self
            .client
            .get(&url_with_query)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        // Use error_for_status to handle HTTP errors appropriately
        match response.error_for_status() {
            Ok(response) => {
                let response_text = response.text().await?;
                Ok(response_text == validation_token)
            }
            Err(_) => Ok(false), // Return false for non-success status codes
        }
    }
}
