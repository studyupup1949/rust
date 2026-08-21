//! Async Bulk Asset Administration Shell Registry API

use crate::part2::v3_1::services::AsyncBulkAASRegistryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    post,
    path = "/bulk/shell-descriptors",
    tag = "Async Bulk Asset Administration Shell Registry API",
    summary = "Creates multiple new Asset Administration Shell Descriptors, i.e. registers multiple Asset Administration Shells",
    responses(
        (status = 202, description = "Bulk operation accepted"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn bulk_post_asset_administration_shell_descriptor<S: AsyncBulkAASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/bulk/shell-descriptors",
    tag = "Async Bulk Asset Administration Shell Registry API",
    summary = "Creates or updates multiple existing Asset Administration Shell Descriptors",
    responses(
        (status = 202, description = "Bulk operation accepted"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn bulk_put_asset_administration_shell_descriptor_by_id<
    S: AsyncBulkAASRegistryService,
>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/bulk/shell-descriptors",
    tag = "Async Bulk Asset Administration Shell Registry API",
    summary = "Deletes multiple Asset Administration Shell Descriptors, i.e. de-registers multiple Asset Administration Shells",
    responses(
        (status = 202, description = "Bulk operation accepted"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn bulk_delete_asset_administration_shell_descriptor_by_id<
    S: AsyncBulkAASRegistryService,
>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

pub fn router(service: impl AsyncBulkAASRegistryService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(
            bulk_post_asset_administration_shell_descriptor,
            bulk_put_asset_administration_shell_descriptor_by_id,
            bulk_delete_asset_administration_shell_descriptor_by_id
        ))
        .with_state(Arc::new(service))
}
