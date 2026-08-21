//! Serialization API

use crate::part2::v3_1::services::SerializationService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/serialization",
    tag = "Serialization API",
    summary = "Returns an appropriate serialization based on the specified format (see SerializationFormat)",
    responses(
        (status = 200, description = "Requested serialization"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found")
    )
)]
pub async fn generate_serialization_by_ids<S: SerializationService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

/// Router for Serialization API
pub fn router(service: impl SerializationService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(generate_serialization_by_ids))
        .with_state(Arc::new(service))
}
