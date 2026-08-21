//! Asset Administration Shell API

use crate::part1::v3_1::core::AssetAdministrationShell;
use crate::part2::v3_1::error::AASError;
use crate::part2::v3_1::services::AASShellService;
use crate::part2::v3_1::types::PutThumbnail;
use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/aas",
    tag = "Asset Administration Shell API",
    summary = "Returns a specific Asset Administration Shell",
    responses(
        (status = 200, body = AssetAdministrationShell, description = "Requested Asset Administration Shell"),
        (status = 400, body = AASError, description = "Bad Request, e.g. the request parameters of the format of the request body is wrong."),
        (status = 401, body = AASError, description = "Unauthorized"),
        (status = 403, body = AASError, description = "Forbidden"),
        (status = 404, body = AASError, description = "Asset Administration Shell not found")
    )
)]
pub async fn get_asset_administration_shell<S: AASShellService>(
    State(service): State<Arc<S>>,
) -> Result<Json<Vec<AssetAdministrationShell>>, Json<AASError>> {
    service
        .find_all_aas()
        .await
        .and_then(|aas| Ok(Json(aas)))
        .map_err(|err| Json(err))
}

#[utoipa::path(
    put,
    path = "/aas",
    tag = "Asset Administration Shell API",
    summary = "Creates or updates an existing Asset Administration Shell",
    responses(
        (status = 201, description = "Asset Administration Shell created successfully"),
        (status = 204, description = "Asset Administration Shell updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 401, body = AASError, description = "Unauthorized"),
        (status = 403, body = AASError, description = "Forbidden"),
        (status = 404, description = "Asset Administration Shell not found")
    )
)]
pub async fn put_asset_administration_shell<S: AASShellService>(
    State(service): State<Arc<S>>,
    Json(aas): Json<AssetAdministrationShell>,
) -> Result<StatusCode, Json<AASError>> {
    service
        .create_or_update_aas(&aas)
        .await
        .map_err(|err| Json(err))
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
    path = "/aas/{aasIdentifier}/asset-information/thumbnail",
    tag = "Asset Administration Shell API",
    summary = "Returns the thumbnail of the Asset Information",
    params(
        ("aasIdentifier" = String, Path, description = "Base64-URLSafe-NoPadding encoded aas id"),
    ),
    responses(
        (status = 200, body = String,   content_type = "application/octet-stream",  description = "The thumbnail of the Asset Information"),
        (status = 400, body = AASError, description = "Asset Administration Shell or thumbnail not found"),
        (status = 401, body = AASError, description = "Unauthorized, e.g. the server refused the authorization attempt."),
        (status = 403, body = AASError, description = "Forbidden"),
        (status = 404, body = AASError, description = "Asset Administration Shell or thumbnail not found"),
        (status = 500, body = AASError, description = "Internal Server Error"),
        (status = "default", body = AASError, description = "Default error handling for unmentioned error codes")
    )
)]
pub async fn get_thumbnail<S: AASShellService>(
    State(service): State<Arc<S>>,
    Path(aas_identifier): Path<String>,
) -> Result<Response, AASError> {
    service.get_thumbnail(aas_identifier).await
}

#[utoipa::path(
    put,
    path = "/aas/{aasIdentifier}/asset-information/thumbnail",
    params(
        ("aasIdentifier" = String, Path, description = "Base64-URLSafe-NoPadding encoded aas id"),
    ),
    tag = "Asset Administration Shell API",
    summary = "Updates the thumbnail of the Asset Information",
    request_body(content = PutThumbnail, content_type = "multipart/form-data"),
    responses(
        (status = 204, description = "Thumbnail updated successfully"),
        (status = 400, body = AASError, description = "Error"),
        (status = 401, body = AASError, description = "Unauthorized, e.g. the server refused the authorization attempt."),
        (status = 403, body = AASError, description = "Forbidden"),
        (status = 404, body = AASError, description = "Asset Information or thumbnail not found"),
        (status = 500, body = AASError, description = "Internal Server Error"),
        (status = "default", body = AASError, description = "Default error handling for unmentioned error codes")
    )
)]
pub async fn put_thumbnail<S: AASShellService>(
    State(service): State<Arc<S>>,
    Path(asset_id): Path<String>,
    thumbnail: Multipart,
) -> Result<(), AASError> {
    service.put_thumbnail(asset_id, thumbnail).await
}

#[utoipa::path(
    delete,
    path = "/aas/{aasIdentifier}/asset-information/thumbnail",
    params(
        ("aasIdentifier" = String, Path, description = "Base64-URLSafe-NoPadding encoded aas id"),
    ),
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
