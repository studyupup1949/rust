use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};

use crate::A2AError;
use crate::jsonrpc::{
    JSONRPC_VERSION, JsonRpcId, JsonRpcRequest, JsonRpcResponse, METHOD_CANCEL_TASK,
    METHOD_CREATE_TASK_PUSH_NOTIFICATION_CONFIG, METHOD_DELETE_TASK_PUSH_NOTIFICATION_CONFIG,
    METHOD_GET_EXTENDED_AGENT_CARD, METHOD_GET_TASK, METHOD_GET_TASK_PUSH_NOTIFICATION_CONFIG,
    METHOD_LIST_TASK_PUSH_NOTIFICATION_CONFIGS, METHOD_LIST_TASKS, METHOD_SEND_MESSAGE,
    METHOD_SEND_STREAMING_MESSAGE, METHOD_SUBSCRIBE_TO_TASK,
};
use crate::types::{
    CancelTaskRequest, DeleteTaskPushNotificationConfigRequest, GetExtendedAgentCardRequest,
    GetTaskPushNotificationConfigRequest, GetTaskRequest, ListTaskPushNotificationConfigsRequest,
    ListTasksRequest, SendMessageRequest, SubscribeToTaskRequest, TaskPushNotificationConfig,
};

use super::handler::A2AHandler;

pub(super) async fn handle<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, Json<JsonRpcResponse>)
where
    H: A2AHandler,
{
    let request = match serde_json::from_slice::<JsonRpcRequest>(&body) {
        Ok(request) => request,
        Err(error) => {
            return (
                StatusCode::OK,
                Json(error_response(
                    JsonRpcId::Null,
                    A2AError::ParseError(error.to_string()),
                )),
            );
        }
    };

    if request.jsonrpc != JSONRPC_VERSION {
        // JSON-RPC envelope errors still return HTTP 200 with the protocol error
        // encoded in the body.
        return (
            StatusCode::OK,
            Json(error_response(
                request.id,
                A2AError::InvalidRequest("jsonrpc must be \"2.0\"".to_owned()),
            )),
        );
    }

    let id = request.id.clone();
    if let Err(error) = handler.validate_protocol_headers(&headers).await {
        return (StatusCode::OK, Json(error_response(id, error)));
    }

    let result = match request.method.as_str() {
        METHOD_SEND_MESSAGE => parse_params::<SendMessageRequest>(request.params)
            .and_then(|params| params.validate().map(|_| params))
            .and_then_async(|params| handler.send_message(params))
            .await
            .and_then(|response| response.validate().map(|_| response))
            .map(serde_json::to_value)
            .and_then(map_serialization_error),
        METHOD_SEND_STREAMING_MESSAGE => {
            parse_params::<SendMessageRequest>(request.params)
                .and_then(|params| params.validate().map(|_| params))
                .and_then_async(|_params| async {
                    Err(A2AError::UnsupportedOperation(
                        "SendStreamingMessage".to_owned(),
                    ))
                })
                .await
        }
        METHOD_GET_TASK => parse_params::<GetTaskRequest>(request.params)
            .and_then_async(|params| handler.get_task(params))
            .await
            .map(serde_json::to_value)
            .and_then(map_serialization_error),
        METHOD_LIST_TASKS => parse_params::<ListTasksRequest>(request.params)
            .and_then(|params| params.validate().map(|_| params))
            .and_then_async(|params| handler.list_tasks(params))
            .await
            .map(serde_json::to_value)
            .and_then(map_serialization_error),
        METHOD_CANCEL_TASK => parse_params::<CancelTaskRequest>(request.params)
            .and_then_async(|params| handler.cancel_task(params))
            .await
            .map(serde_json::to_value)
            .and_then(map_serialization_error),
        METHOD_SUBSCRIBE_TO_TASK => {
            parse_params::<SubscribeToTaskRequest>(request.params)
                .and_then_async(|_params| async {
                    Err(A2AError::UnsupportedOperation("SubscribeToTask".to_owned()))
                })
                .await
        }
        METHOD_CREATE_TASK_PUSH_NOTIFICATION_CONFIG => {
            parse_params::<TaskPushNotificationConfig>(request.params)
                .and_then_async(|params| handler.create_task_push_notification_config(params))
                .await
                .map(serde_json::to_value)
                .and_then(map_serialization_error)
        }
        METHOD_GET_TASK_PUSH_NOTIFICATION_CONFIG => {
            parse_params::<GetTaskPushNotificationConfigRequest>(request.params)
                .and_then_async(|params| handler.get_task_push_notification_config(params))
                .await
                .map(serde_json::to_value)
                .and_then(map_serialization_error)
        }
        METHOD_LIST_TASK_PUSH_NOTIFICATION_CONFIGS => {
            parse_params::<ListTaskPushNotificationConfigsRequest>(request.params)
                .and_then(|params| params.validate().map(|_| params))
                .and_then_async(|params| handler.list_task_push_notification_configs(params))
                .await
                .map(serde_json::to_value)
                .and_then(map_serialization_error)
        }
        METHOD_DELETE_TASK_PUSH_NOTIFICATION_CONFIG => {
            parse_params::<DeleteTaskPushNotificationConfigRequest>(request.params)
                .and_then_async(|params| handler.delete_task_push_notification_config(params))
                .await
                .map(|()| serde_json::json!({}))
        }
        METHOD_GET_EXTENDED_AGENT_CARD => {
            parse_params::<GetExtendedAgentCardRequest>(request.params)
                .and_then_async(|params| handler.get_extended_agent_card(params))
                .await
                .map(serde_json::to_value)
                .and_then(map_serialization_error)
        }
        method => Err(A2AError::MethodNotFound(method.to_owned())),
    };

    let response = match result {
        Ok(result) => JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            result: Some(result),
            error: None,
            id,
        },
        Err(error) => error_response(id, error),
    };

    (StatusCode::OK, Json(response))
}

fn parse_params<T>(params: Option<serde_json::Value>) -> Result<T, A2AError>
where
    T: serde::de::DeserializeOwned,
{
    let params = params.unwrap_or_else(|| serde_json::Value::Object(Default::default()));
    serde_json::from_value(params).map_err(|error| A2AError::InvalidParams(error.to_string()))
}

fn map_serialization_error(
    value: Result<serde_json::Value, serde_json::Error>,
) -> Result<serde_json::Value, A2AError> {
    value.map_err(A2AError::from)
}

fn error_response(id: JsonRpcId, error: A2AError) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: JSONRPC_VERSION.to_owned(),
        result: None,
        error: Some(error.to_jsonrpc_error()),
        id,
    }
}

trait AsyncResultExt<T> {
    async fn and_then_async<Fut, U>(self, func: impl FnOnce(T) -> Fut) -> Result<U, A2AError>
    where
        Fut: std::future::Future<Output = Result<U, A2AError>>;
}

impl<T> AsyncResultExt<T> for Result<T, A2AError> {
    async fn and_then_async<Fut, U>(self, func: impl FnOnce(T) -> Fut) -> Result<U, A2AError>
    where
        Fut: std::future::Future<Output = Result<U, A2AError>>,
    {
        match self {
            Ok(value) => func(value).await,
            Err(error) => Err(error),
        }
    }
}
