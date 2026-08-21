//! AASX File Server API

use axum::extract::State;
use axum::http::StatusCode;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::part2::v3_1::services::AASXFileServerService;

#[utoipa::path(
    get,
    path = "/packages",
    tag = "AASX File Server API",
    summary = "Returns a list of available AASX packages at the server",
    responses(
        (status = 200, description = "Returns a list of available AASX packages at the server")
    )
)]
pub async fn get_all_aasx_package_ids<S: AASXFileServerService>(
    State(_service): State<Arc<S>>,
) -> StatusCode {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/packages",
    tag = "AASX File Server API",
    summary = "Stores the AASX package at the server",
    responses(
        (status = 201, description = "AASX package stored successfully"),
        (status = 501, description = "Not implemented")
    )
)]
pub async fn post_aasx_package<S: AASXFileServerService>(
    State(_service): State<Arc<S>>,
) -> StatusCode {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/packages/{packageId}",
    tag = "AASX File Server API",
    summary = "Returns a specific AASX package from the server",
    params(
        ("packageId" = String, Path, description = "Package identifier")
    ),
    responses(
        (status = 200, description = "AASX package returned successfully"),
        (status = 501, description = "Not implemented")
    )
)]
pub async fn get_aasx_by_package_id<S: AASXFileServerService>(
    State(_service): State<Arc<S>>,
) -> StatusCode {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/packages/{packageId}",
    tag = "AASX File Server API",
    summary = "Creates or updates the AASX package at the server",
    params(
        ("packageId" = String, Path, description = "Package identifier")
    ),
    responses(
        (status = 200, description = "AASX package updated successfully"),
        (status = 201, description = "AASX package created successfully"),
        (status = 501, description = "Not implemented")
    )
)]
pub async fn put_aasx_by_package_id<S: AASXFileServerService>(
    State(_service): State<Arc<S>>,
) -> StatusCode {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/packages/{packageId}",
    tag = "AASX File Server API",
    summary = "Deletes a specific AASX package from the server",
    params(
        ("packageId" = String, Path, description = "Package identifier")
    ),
    responses(
        (status = 204, description = "AASX package deleted successfully"),
        (status = 501, description = "Not implemented")
    )
)]
pub async fn delete_aasx_by_package_id<S: AASXFileServerService>(
    State(_service): State<Arc<S>>,
) -> StatusCode {
    unimplemented!()
}

/// Router for AASX File Server API
pub fn router(service: impl AASXFileServerService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_all_aasx_package_ids, post_aasx_package))
        .routes(routes!(
            get_aasx_by_package_id,
            put_aasx_by_package_id,
            delete_aasx_by_package_id
        ))
        .with_state(Arc::new(service))
}
