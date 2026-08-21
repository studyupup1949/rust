//! Async Bulk Submodel Registry API

use crate::part2::v3_1::services::AsyncBulkSubmodelRegistryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    post,
    path = "/bulk/submodel-descriptors",
    tag = "Async Bulk Submodel Registry API",
    summary = "Creates multiple new Submodel Descriptors",
    responses(
        (status = 202, description = "Bulk operation accepted"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn bulk_post_submodel() {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/bulk/submodel-descriptors",
    tag = "Async Bulk Submodel Registry API",
    summary = "Updates multiple existing Submodel Descriptors",
    responses(
        (status = 202, description = "Bulk operation accepted"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn bulk_put_submodel_descriptors_by_id() {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/bulk/submodel-descriptors",
    tag = "Async Bulk Submodel Registry API",
    summary = "Deletes multiple Submodel Descriptors",
    responses(
        (status = 202, description = "Bulk operation accepted"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn bulk_delete_submodel_descriptors_by_id<S: AsyncBulkSubmodelRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

pub fn router(service: impl AsyncBulkSubmodelRegistryService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(
            bulk_post_submodel,
            bulk_put_submodel_descriptors_by_id,
            bulk_delete_submodel_descriptors_by_id
        ))
        .with_state(Arc::new(service))
}
