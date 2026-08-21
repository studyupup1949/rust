use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use super::handler::A2AHandler;
use super::{jsonrpc, rest, streaming};

/// Build an axum router exposing the A2A REST, JSON-RPC, and discovery routes.
pub fn router<H>(handler: H) -> Router
where
    H: A2AHandler,
{
    let handler = Arc::new(handler);

    Router::new()
        .route(
            "/.well-known/agent-card.json",
            get(rest::get_agent_card::<H>),
        )
        .route("/message:send", post(rest::send_message::<H>))
        .route(
            "/{tenant}/message:send",
            post(rest::tenant_send_message::<H>),
        )
        .route("/message:stream", post(streaming::send_message::<H>))
        .route(
            "/{tenant}/message:stream",
            post(streaming::tenant_send_message::<H>),
        )
        .route("/tasks", get(rest::list_tasks::<H>))
        .route("/{tenant}/tasks", get(rest::tenant_list_tasks::<H>))
        // axum/matchit does not support literal suffixes like `:cancel` or `:subscribe`
        // on the same segment as a capture, so those canonical A2A paths are dispatched
        // inside the task handlers after extracting the full `{id}` segment.
        .route(
            "/tasks/{id}",
            get(rest::get_task_or_subscribe::<H>).post(rest::cancel_task::<H>),
        )
        .route(
            "/{tenant}/tasks/{id}",
            get(rest::tenant_get_task_or_subscribe::<H>).post(rest::tenant_cancel_task::<H>),
        )
        .route(
            "/tasks/{task_id}/pushNotificationConfigs",
            post(rest::create_task_push_notification_config::<H>)
                .get(rest::list_task_push_notification_configs::<H>),
        )
        .route(
            "/{tenant}/tasks/{task_id}/pushNotificationConfigs",
            post(rest::tenant_create_task_push_notification_config::<H>)
                .get(rest::tenant_list_task_push_notification_configs::<H>),
        )
        .route(
            "/tasks/{task_id}/pushNotificationConfigs/{id}",
            get(rest::get_task_push_notification_config::<H>)
                .delete(rest::delete_task_push_notification_config::<H>),
        )
        .route(
            "/{tenant}/tasks/{task_id}/pushNotificationConfigs/{id}",
            get(rest::tenant_get_task_push_notification_config::<H>)
                .delete(rest::tenant_delete_task_push_notification_config::<H>),
        )
        .route(
            "/extendedAgentCard",
            get(rest::get_extended_agent_card::<H>),
        )
        .route(
            "/{tenant}/extendedAgentCard",
            get(rest::tenant_get_extended_agent_card::<H>),
        )
        .route("/rpc", post(jsonrpc::handle::<H>))
        .route("/jsonrpc", post(jsonrpc::handle::<H>))
        .with_state(handler)
}
