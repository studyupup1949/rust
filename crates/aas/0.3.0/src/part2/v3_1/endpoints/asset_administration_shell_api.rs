//! Asset Administration Shell API

use crate::part2::v3_1::services::AASShellService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/aas",
    tag = "Asset Administration Shell API",
    summary = "Returns a specific Asset Administration Shell",
    responses(
        (status = 200, description = "Requested Asset Administration Shell"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn get_asset_administration_shell<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/aas",
    tag = "Asset Administration Shell API",
    summary = "Creates or updates an existing Asset Administration Shell",
    responses(
        (status = 200, description = "Asset Administration Shell updated successfully"),
        (status = 201, description = "Asset Administration Shell created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn put_asset_administration_shell<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/aas/$reference",
    tag = "Asset Administration Shell API",
    summary = "Returns a specific Asset Administration Shell as a Reference",
    responses(
        (status = 200, description = "Requested Asset Administration Shell as Reference"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn get_asset_administration_shell_reference<S: AASShellService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/aas/asset-information",
    tag = "Asset Administration Shell API",
    summary = "Returns the Asset Information",
    responses(
        (status = 200, description = "Requested Asset Information"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn get_asset_information<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/aas/asset-information",
    tag = "Asset Administration Shell API",
    summary = "Updates the Asset Information",
    responses(
        (status = 204, description = "Asset Information updated successfully"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn put_asset_information<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/aas/asset-information/thumbnail",
    tag = "Asset Administration Shell API",
    summary = "Returns the thumbnail of the Asset Information",
    responses(
        (status = 200, description = "Requested thumbnail"),
        (status = 404, description = "Asset Administration Shell or thumbnail not found")
    )
)]
pub async fn get_thumbnail<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/aas/asset-information/thumbnail",
    tag = "Asset Administration Shell API",
    summary = "Updates the thumbnail of the Asset Information",
    responses(
        (status = 204, description = "Thumbnail updated successfully"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn put_thumbnail<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/aas/asset-information/thumbnail",
    tag = "Asset Administration Shell API",
    summary = "Deletes the thumbnail from the Asset Information",
    responses(
        (status = 204, description = "Thumbnail deleted successfully"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn delete_thumbnail<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/aas/submodel-refs",
    tag = "Asset Administration Shell API",
    summary = "Returns all submodel references",
    responses(
        (status = 200, description = "List of submodel references"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn get_all_submodel_references<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/aas/submodel-refs",
    tag = "Asset Administration Shell API",
    summary = "Creates a submodel reference at the Asset Administration Shell",
    responses(
        (status = 201, description = "Submodel reference created successfully"),
        (status = 404, description = "Asset Administration Shell not found"),
        (status = 409, description = "Submodel reference already exists")
    )
)]
pub async fn post_submodel_reference<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/aas/submodel-refs/{submodelIdentifier}",
    tag = "Asset Administration Shell API",
    summary = "Deletes the submodel reference from the Asset Administration Shell",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 204, description = "Submodel reference deleted successfully"),
        (status = 404, description = "Asset Administration Shell or Submodel reference not found")
    )
)]
pub async fn delete_submodel_reference<S: AASShellService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

pub fn router(service: impl AASShellService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(
            get_asset_administration_shell,
            put_asset_administration_shell,
        ))
        .routes(routes!(get_asset_information, put_asset_information,))
        .routes(routes!(get_thumbnail, put_thumbnail, delete_thumbnail,))
        .routes(routes!(
            get_all_submodel_references,
            post_submodel_reference,
            delete_submodel_reference,
        ))
        .routes(routes!(get_asset_administration_shell_reference,))
        .with_state(Arc::new(service))
}
