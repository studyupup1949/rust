//! Asset Administration Shell Basic Discovery API

use crate::part2::v3_1::services::AASBasicDiscoveryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/lookup/shells",
    tag = "Asset Administration Shell Basic Discovery API",
    responses(
        (status = 200, description = "List of AAS IDs by asset link")
    )
)]
pub async fn get_all_asset_administration_shell_ids_by_asset_link<S: AASBasicDiscoveryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/lookup/shellsByAssetLink",
    tag = "Asset Administration Shell Basic Discovery API",
    responses(
        (status = 200, description = "Search results for AAS IDs by asset link")
    )
)]
pub async fn search_all_asset_administration_shell_ids_by_asset_link<
    S: AASBasicDiscoveryService,
>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/lookup/shells/{aasIdentifier}",
    tag = "Asset Administration Shell Basic Discovery API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 200, description = "List of asset links by AAS ID")
    )
)]
pub async fn get_all_asset_links_by_id<S: AASBasicDiscoveryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/lookup/shells/{aasIdentifier}",
    tag = "Asset Administration Shell Basic Discovery API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 200, description = "Created or replaced asset links")
    )
)]
pub async fn post_all_asset_links_by_id<S: AASBasicDiscoveryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/lookup/shells/{aasIdentifier}",
    tag = "Asset Administration Shell Basic Discovery API",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    responses(
        (status = 200, description = "Deleted asset links by AAS ID")
    )
)]
pub async fn delete_all_asset_links_by_id<S: AASBasicDiscoveryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

// Define OpenApi documentation object including above paths
#[derive(OpenApi)]
#[openapi(paths(
    get_all_asset_administration_shell_ids_by_asset_link,
    search_all_asset_administration_shell_ids_by_asset_link,
    get_all_asset_links_by_id,
    post_all_asset_links_by_id,
    delete_all_asset_links_by_id,
))]
pub struct AASBasicDiscoveryAPI;

pub fn router(service: impl AASBasicDiscoveryService) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(
            get_all_asset_administration_shell_ids_by_asset_link,
            search_all_asset_administration_shell_ids_by_asset_link,
        ))
        .routes(routes!(
            get_all_asset_links_by_id,
            post_all_asset_links_by_id,
            delete_all_asset_links_by_id,
        ))
        .with_state(Arc::new(service))
}
