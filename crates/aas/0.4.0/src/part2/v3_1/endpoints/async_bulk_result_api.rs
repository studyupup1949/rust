//! Async Bulk Result API

use crate::part2::v3_1::services::AsyncBulkResultService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/bulk/result/{handleId}",
    tag = "Async Bulk Result API",
    summary = "Returns the result object of an asynchronously invoked bulk operation",
    params(
        ("handleId" = String, Path, description = "Handle ID for the asynchronous bulk operation")
    ),
    responses(
        (status = 200, description = "Bulk operation result"),
        (status = 404, description = "Handle ID not found")
    )
)]
pub async fn bulk_get_async_result<S: AsyncBulkResultService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

/// Router for Async Bulk Result API
pub fn router(service: impl AsyncBulkResultService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(bulk_get_async_result))
        .with_state(Arc::new(service))
}
