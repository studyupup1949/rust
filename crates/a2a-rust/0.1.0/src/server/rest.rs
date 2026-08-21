use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::A2AError;
use crate::error::ProblemDetails;
use crate::types::{
    AgentCard, CancelTaskRequest, DeleteTaskPushNotificationConfigRequest,
    GetExtendedAgentCardRequest, GetTaskPushNotificationConfigRequest, GetTaskRequest,
    ListTaskPushNotificationConfigsRequest, ListTaskPushNotificationConfigsResponse,
    ListTasksRequest, ListTasksResponse, SendMessageRequest, SendMessageResponse,
    SubscribeToTaskRequest, Task, TaskPushNotificationConfig,
};

use super::handler::A2AHandler;
use super::streaming;

pub(super) async fn get_agent_card<H>(
    State(handler): State<Arc<H>>,
) -> Result<Json<AgentCard>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.get_agent_card().await.map(Json).map_err(rest_error)
}

pub(super) async fn send_message<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Json(request): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    request.validate()?;

    handler
        .send_message(request)
        .await
        .and_then(|response| {
            response.validate()?;
            Ok(response)
        })
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn tenant_send_message<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(tenant): Path<String>,
    Json(mut request): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, RestErrorResponse>
where
    H: A2AHandler,
{
    request.tenant = Some(tenant);
    send_message(State(handler), headers, Json(request)).await
}

pub(super) async fn get_task_or_subscribe<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GetTaskQuery>,
) -> Response
where
    H: A2AHandler,
{
    if let Err(error) = handler.validate_protocol_headers(&headers).await {
        return rest_error(error).into_response();
    }

    if let Err(error) = reject_query_tenant(&query.tenant) {
        return error.into_response();
    }

    if let Some(id) = id.strip_suffix(":subscribe") {
        return match streaming::subscribe_to_task_response(
            handler,
            SubscribeToTaskRequest {
                id: id.to_owned(),
                tenant: query.tenant,
            },
        )
        .await
        {
            Ok(response) => response.into_response(),
            Err(error) => error.into_response(),
        };
    }

    get_task(State(handler), headers, Path(id), Query(query))
        .await
        .into_response()
}

pub(super) async fn tenant_get_task_or_subscribe<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((tenant, id)): Path<(String, String)>,
    Query(mut query): Query<GetTaskQuery>,
) -> Response
where
    H: A2AHandler,
{
    if let Err(error) = handler.validate_protocol_headers(&headers).await {
        return rest_error(error).into_response();
    }

    query.tenant = Some(tenant);

    if let Some(id) = id.strip_suffix(":subscribe") {
        return match streaming::subscribe_to_task_response(
            handler,
            SubscribeToTaskRequest {
                id: id.to_owned(),
                tenant: query.tenant,
            },
        )
        .await
        {
            Ok(response) => response.into_response(),
            Err(error) => error.into_response(),
        };
    }

    match handler
        .get_task(GetTaskRequest {
            id,
            history_length: query.history_length,
            tenant: query.tenant,
        })
        .await
    {
        Ok(task) => Json(task).into_response(),
        Err(error) => rest_error(error).into_response(),
    }
}

pub(super) async fn get_task<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<GetTaskQuery>,
) -> Result<Json<Task>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&query.tenant)?;

    if id.ends_with(":cancel") || id.ends_with(":subscribe") {
        return Err(rest_error(A2AError::MethodNotFound("not found".to_owned())));
    }

    handler
        .get_task(GetTaskRequest {
            id,
            history_length: query.history_length,
            tenant: query.tenant,
        })
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn list_tasks<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Query(request): Query<ListTasksRequest>,
) -> Result<Json<ListTasksResponse>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&request.tenant)?;
    request.validate()?;

    handler
        .list_tasks(request)
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn tenant_list_tasks<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(tenant): Path<String>,
    Query(mut request): Query<ListTasksRequest>,
) -> Result<Json<ListTasksResponse>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    request.tenant = Some(tenant);

    request.validate()?;

    handler
        .list_tasks(request)
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn cancel_task<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<TenantQuery>,
    body: Option<Json<CancelTaskBody>>,
) -> Result<Json<Task>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&query.tenant)?;

    let Some(id) = id.strip_suffix(":cancel") else {
        return Err(rest_error(A2AError::MethodNotFound("not found".to_owned())));
    };

    handler
        .cancel_task(CancelTaskRequest {
            id: id.to_owned(),
            tenant: query.tenant,
            metadata: body.and_then(|body| body.metadata.clone()),
        })
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn tenant_cancel_task<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((tenant, id)): Path<(String, String)>,
    Query(mut query): Query<TenantQuery>,
    body: Option<Json<CancelTaskBody>>,
) -> Result<Json<Task>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    query.tenant = Some(tenant);

    let Some(id) = id.strip_suffix(":cancel") else {
        return Err(rest_error(A2AError::MethodNotFound("not found".to_owned())));
    };

    handler
        .cancel_task(CancelTaskRequest {
            id: id.to_owned(),
            tenant: query.tenant,
            metadata: body.and_then(|body| body.metadata.clone()),
        })
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn get_extended_agent_card<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Query(query): Query<TenantQuery>,
) -> Result<Json<AgentCard>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&query.tenant)?;

    handler
        .get_extended_agent_card(GetExtendedAgentCardRequest {
            tenant: query.tenant,
        })
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn tenant_get_extended_agent_card<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(tenant): Path<String>,
    Query(mut query): Query<TenantQuery>,
) -> Result<Json<AgentCard>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    query.tenant = Some(tenant);

    handler
        .get_extended_agent_card(GetExtendedAgentCardRequest {
            tenant: query.tenant,
        })
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn create_task_push_notification_config<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<TenantQuery>,
    Json(mut config): Json<TaskPushNotificationConfig>,
) -> Result<Json<TaskPushNotificationConfig>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&query.tenant)?;

    config.task_id = task_id;
    config.tenant = query.tenant;

    handler
        .create_task_push_notification_config(config)
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn tenant_create_task_push_notification_config<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((tenant, task_id)): Path<(String, String)>,
    Query(_query): Query<TenantQuery>,
    Json(mut config): Json<TaskPushNotificationConfig>,
) -> Result<Json<TaskPushNotificationConfig>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    config.task_id = task_id;
    config.tenant = Some(tenant);

    handler
        .create_task_push_notification_config(config)
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn get_task_push_notification_config<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((task_id, id)): Path<(String, String)>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<TaskPushNotificationConfig>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&query.tenant)?;

    handler
        .get_task_push_notification_config(GetTaskPushNotificationConfigRequest {
            id,
            task_id,
            tenant: query.tenant,
        })
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn tenant_get_task_push_notification_config<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((tenant, task_id, id)): Path<(String, String, String)>,
    Query(mut query): Query<TenantQuery>,
) -> Result<Json<TaskPushNotificationConfig>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    query.tenant = Some(tenant);

    handler
        .get_task_push_notification_config(GetTaskPushNotificationConfigRequest {
            id,
            task_id,
            tenant: query.tenant,
        })
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn list_task_push_notification_configs<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<ListTaskPushNotificationConfigsQuery>,
) -> Result<Json<ListTaskPushNotificationConfigsResponse>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&query.tenant)?;

    let request = ListTaskPushNotificationConfigsRequest {
        task_id,
        page_size: query.page_size,
        page_token: query.page_token,
        tenant: query.tenant,
    };
    request.validate()?;

    handler
        .list_task_push_notification_configs(request)
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn tenant_list_task_push_notification_configs<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((tenant, task_id)): Path<(String, String)>,
    Query(mut query): Query<ListTaskPushNotificationConfigsQuery>,
) -> Result<Json<ListTaskPushNotificationConfigsResponse>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    query.tenant = Some(tenant);

    let request = ListTaskPushNotificationConfigsRequest {
        task_id,
        page_size: query.page_size,
        page_token: query.page_token,
        tenant: query.tenant,
    };
    request.validate()?;

    handler
        .list_task_push_notification_configs(request)
        .await
        .map(Json)
        .map_err(rest_error)
}

pub(super) async fn delete_task_push_notification_config<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((task_id, id)): Path<(String, String)>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    reject_query_tenant(&query.tenant)?;

    handler
        .delete_task_push_notification_config(DeleteTaskPushNotificationConfigRequest {
            id,
            task_id,
            tenant: query.tenant,
        })
        .await
        .map(|()| Json(serde_json::json!({})))
        .map_err(rest_error)
}

pub(super) async fn tenant_delete_task_push_notification_config<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path((tenant, task_id, id)): Path<(String, String, String)>,
    Query(mut query): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    query.tenant = Some(tenant);

    handler
        .delete_task_push_notification_config(DeleteTaskPushNotificationConfigRequest {
            id,
            task_id,
            tenant: query.tenant,
        })
        .await
        .map(|()| Json(serde_json::json!({})))
        .map_err(rest_error)
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetTaskQuery {
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub history_length: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ListTaskPushNotificationConfigsQuery {
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub page_size: Option<i32>,
    #[serde(default)]
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TenantQuery {
    #[serde(default)]
    pub tenant: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CancelTaskBody {
    #[serde(default)]
    pub metadata: Option<crate::types::JsonObject>,
}

pub(super) fn rest_error(error: A2AError) -> RestErrorResponse {
    RestErrorResponse {
        status: error.status_code(),
        body: Box::new(error.to_problem_details()),
    }
}

impl From<A2AError> for RestErrorResponse {
    fn from(value: A2AError) -> Self {
        rest_error(value)
    }
}

fn reject_query_tenant(tenant: &Option<String>) -> Result<(), RestErrorResponse> {
    if tenant.is_some() {
        return Err(rest_error(A2AError::InvalidRequest(
            "tenant must be supplied via tenant-prefixed routes".to_owned(),
        )));
    }

    Ok(())
}

pub(super) struct RestErrorResponse {
    status: StatusCode,
    body: Box<ProblemDetails>,
}

impl IntoResponse for RestErrorResponse {
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(*self.body)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}
