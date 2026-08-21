// Task handlers module - contains common handler implementations for standard task operations
//
// This module provides a set of standard handler functions for common task operations like
// create, read, cancel, etc. These functions can be used with the make_typed_handler function
// to create type-safe request handlers.

use std::sync::Arc;

use crate::server::task_manager::TaskManager;
use crate::{JsonRpcError, Task, TaskIdParams, TaskQueryParams, TaskSendParams, TaskStatus};

/// Standard handler for tasks/send requests - creates or updates a task
///
/// # Arguments
/// * `task_manager` - The task manager implementation
/// * `params` - Parameters for the task creation/update
///
/// # Returns
/// * The created or updated task, or an error if the operation fails
pub async fn handle_send_task(
    task_manager: Arc<TaskManager>,
    params: TaskSendParams,
) -> Result<Task, JsonRpcError> {
    // First get a copy of the task
    let task_query = TaskQueryParams::from_id(params.id.clone());

    // Create/update the task
    task_manager.upsert_task(&params).await?;

    // Get the updated task to return
    Ok(task_manager.get_task(&task_query).await?)
}

/// Standard handler for tasks/get requests - retrieves a task by ID
///
/// # Arguments
/// * `task_manager` - The task manager implementation
/// * `params` - Parameters for the task query
///
/// # Returns
/// * The requested task, or an error if the operation fails
pub async fn handle_get_task(
    task_manager: Arc<TaskManager>,
    params: TaskQueryParams,
) -> Result<Task, JsonRpcError> {
    Ok(task_manager.get_task(&params).await?)
}

/// Standard handler for tasks/cancel requests - cancels a task by ID
///
/// # Arguments
/// * `task_manager` - The task manager implementation
/// * `params` - Parameters containing the task ID
///
/// # Returns
/// * The canceled task, or an error if the operation fails
pub async fn handle_cancel_task(
    task_manager: Arc<TaskManager>,
    params: TaskIdParams,
) -> Result<Task, JsonRpcError> {
    // Get a copy of the task ID
    let task_id = params.id.clone();

    // Cancel the task
    task_manager.cancel_task(&params).await?;

    // Get the updated task to return
    let task_query = TaskQueryParams::from_id(task_id);

    Ok(task_manager.get_task(&task_query).await?)
}

/// Handler for updating task status
///
/// # Arguments
/// * `task_manager` - The task manager implementation
/// * `task_id` - ID of the task to update
/// * `status` - New status for the task
///
/// # Returns
/// * The updated task, or an error if the operation fails
pub async fn handle_update_task_status(
    task_manager: Arc<TaskManager>,
    task_id: String,
    status: TaskStatus,
) -> Result<Task, JsonRpcError> {
    // Update the task status
    task_manager.update_task_status(&task_id, status).await?;

    // Get the updated task to return
    let task_query = TaskQueryParams::from_id(task_id);

    Ok(task_manager.get_task(&task_query).await?)
}

/// Parameters for updating task status
#[derive(serde::Deserialize)]
pub struct UpdateTaskStatusParams {
    /// ID of the task to update
    pub id: String,

    /// New status for the task
    pub status: TaskStatus,
}

/// Convenience handler for tasks/updateStatus requests
///
/// # Arguments
/// * `task_manager` - The task manager implementation
/// * `params` - Parameters containing the task ID and new status
///
/// # Returns
/// * The updated task, or an error if the operation fails
pub async fn handle_update_task_status_params(
    task_manager: Arc<TaskManager>,
    params: UpdateTaskStatusParams,
) -> Result<Task, JsonRpcError> {
    handle_update_task_status(task_manager, params.id, params.status).await
}
