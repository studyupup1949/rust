//! Description API

use crate::part2::v3_1::services::DescriptionService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/description",
    tag = "Description API",
    summary = "Returns the self-describing information of a network resource (ServiceDescription)",
    responses(
        (status = 200, description = "Requested service description"),
        (status = 404, description = "Service description not found")
    )
)]
pub async fn get_self_description<S: DescriptionService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

/// Router for Description API
pub fn router(service: impl DescriptionService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_self_description))
        .with_state(Arc::new(service))
}
