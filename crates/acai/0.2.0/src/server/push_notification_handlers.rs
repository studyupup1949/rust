use std::sync::Arc;

use crate::error::validate_url;
use crate::server::task_manager::{TaskManager, TaskManagerError};
use crate::types::{JsonRpcError, TaskIdParams, TaskPushNotificationConfig};

/// Handler function for get push notification requests
pub async fn get_push_notification(
    task_manager: Arc<TaskManager>,
    params: TaskIdParams,
) -> Result<TaskPushNotificationConfig, JsonRpcError> {
    let result = task_manager.get_push_notification(&params).await;

    match result {
        Ok(config) => Ok(config),
        Err(TaskManagerError::TaskNotFound(id)) => Err(JsonRpcError::task_not_found(id)),
        Err(e) => Err(JsonRpcError::internal_error(e)),
    }
}

/// Handler function for set push notification requests
pub async fn set_push_notification(
    task_manager: Arc<TaskManager>,
    params: TaskPushNotificationConfig,
) -> Result<TaskPushNotificationConfig, JsonRpcError> {
    // Validate the push notification URL
    validate_url(&params.push_notification_config.url)?;

    // Set the push notification
    let result = task_manager.set_push_notification(&params).await;

    match result {
        Ok(config) => Ok(config),
        Err(TaskManagerError::TaskNotFound(id)) => Err(JsonRpcError::task_not_found(id)),
        Err(e) => {
            // Push notification might not be supported or there might be validation errors
            if e.to_string().contains("validation") {
                Err(JsonRpcError::invalid_parameters(e))
            } else {
                Err(JsonRpcError::push_notification(e))
            }
        }
    }
}
