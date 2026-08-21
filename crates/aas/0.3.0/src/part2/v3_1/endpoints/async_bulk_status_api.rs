//! Async Bulk Status API

use crate::part2::v3_1::services::AsyncBulkStatusService;
use axum::extract::State;
use axum::handler::Handler;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/bulk/status/{handleId}",
    tag = "Async Bulk Status API",
    summary = "Returns the status of an asynchronously invoked bulk operation",
    params(
        ("handleId" = String, Path, description = "Handle ID for the asynchronous bulk operation")
    ),
    responses(
        (status = 200, description = "Bulk operation status"),
        (status = 404, description = "Handle ID not found")
    )
)]
pub async fn bulk_get_async_status<S: AsyncBulkStatusService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

/// Router for Async Bulk Status API
pub fn router(service: impl AsyncBulkStatusService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(bulk_get_async_status))
        .with_state(Arc::new(service))
}
