//! Asset Administration Shell Registry API

use crate::part2::v3_1::services::AASRegistryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/shell-descriptors",
    tag = "Asset Administration Shell Registry API",
    responses(
        (status = 200, description = "List of all Asset Administration Shell Descriptors")
    )
)]
pub async fn get_all_asset_administration_shell_descriptors<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shell-descriptors",
    tag = "Asset Administration Shell Registry API",
    responses(
        (status = 201, description = "Asset Administration Shell Descriptor created successfully")
    )
)]
pub async fn post_asset_administration_shell_descriptor<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shell-descriptors/{aasIdentifier}",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 200, description = "Asset Administration Shell Descriptor retrieved successfully")
    )
)]
pub async fn get_asset_administration_shell_descriptor_by_id<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shell-descriptors/{aasIdentifier}",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 200, description = "Asset Administration Shell Descriptor updated successfully"),
        (status = 201, description = "Asset Administration Shell Descriptor created successfully")
    )
)]
pub async fn put_asset_administration_shell_descriptor_by_id<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shell-descriptors/{aasIdentifier}",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 204, description = "Asset Administration Shell Descriptor deleted successfully")
    )
)]
pub async fn delete_asset_administration_shell_descriptor_by_id<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shell-descriptors/{aasIdentifier}/submodel-descriptors",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 200, description = "List of all Submodel Descriptors")
    )
)]
pub async fn get_all_submodel_descriptors_through_superpath<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shell-descriptors/{aasIdentifier}/submodel-descriptors",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 201, description = "Submodel Descriptor created successfully")
    )
)]
pub async fn post_submodel_descriptor_through_superpath<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shell-descriptors/{aasIdentifier}/submodel-descriptors/{submodelIdentifier}",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    responses(
        (status = 200, description = "Submodel Descriptor retrieved successfully")
    )
)]
pub async fn get_submodel_descriptor_by_id_through_superpath<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shell-descriptors/{aasIdentifier}/submodel-descriptors/{submodelIdentifier}",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    responses(
        (status = 200, description = "Submodel Descriptor updated successfully"),
        (status = 201, description = "Submodel Descriptor created successfully")
    )
)]
pub async fn put_submodel_descriptor_by_id_through_superpath<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shell-descriptors/{aasIdentifier}/submodel-descriptors/{submodelIdentifier}",
    tag = "Asset Administration Shell Registry API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    responses(
        (status = 204, description = "Submodel Descriptor deleted successfully")
    )
)]
pub async fn delete_submodel_descriptor_by_id_through_superpath<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/query/shell-descriptors",
    tag = "Asset Administration Shell Registry API",
    responses(
        (status = 200, description = "Query results for Asset Administration Shell Descriptors")
    )
)]
pub async fn query_asset_administration_shell_descriptors<S: AASRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

// Define OpenApi documentation object including all paths
#[derive(OpenApi)]
#[openapi(paths(
    get_all_asset_administration_shell_descriptors,
    post_asset_administration_shell_descriptor,
    get_asset_administration_shell_descriptor_by_id,
    put_asset_administration_shell_descriptor_by_id,
    delete_asset_administration_shell_descriptor_by_id,
    get_all_submodel_descriptors_through_superpath,
    post_submodel_descriptor_through_superpath,
    get_submodel_descriptor_by_id_through_superpath,
    put_submodel_descriptor_by_id_through_superpath,
    delete_submodel_descriptor_by_id_through_superpath,
    query_asset_administration_shell_descriptors,
))]
pub struct AASRegistryAPI;

pub fn router(service: impl AASRegistryService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_all_asset_administration_shell_descriptors))
        .routes(routes!(get_asset_administration_shell_descriptor_by_id))
        .routes(routes!(query_asset_administration_shell_descriptors))
        .routes(routes!(get_all_submodel_descriptors_through_superpath))
        .routes(routes!(get_submodel_descriptor_by_id_through_superpath))
        .routes(routes!(
            post_asset_administration_shell_descriptor,
            put_asset_administration_shell_descriptor_by_id,
            delete_asset_administration_shell_descriptor_by_id,
        ))
        .routes(routes!(
            post_submodel_descriptor_through_superpath,
            put_submodel_descriptor_by_id_through_superpath,
            delete_submodel_descriptor_by_id_through_superpath,
        ))
        .with_state(Arc::new(service))
}
