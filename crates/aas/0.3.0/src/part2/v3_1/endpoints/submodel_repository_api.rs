//! Submodel Repository API

use crate::part2::v3_1::services::SubmodelRepositoryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/submodels",
    tag = "Submodel Repository API",
    summary = "Returns all Submodels",
    responses(
        (status = 200, description = "List of all Submodels"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn get_all_submodels<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodels",
    tag = "Submodel Repository API",
    summary = "Creates a new Submodel",
    responses(
        (status = 201, description = "Submodel created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 409, description = "Conflict - Submodel with same identifier already exists")
    )
)]
pub async fn post_submodel<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/$metadata",
    tag = "Submodel Repository API",
    summary = "Returns the metadata attributes of all Submodels",
    responses(
        (status = 200, description = "Metadata of all Submodels"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn get_all_submodels_metadata<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/$value",
    tag = "Submodel Repository API",
    summary = "Returns all Submodels in their ValueOnly representation",
    responses(
        (status = 200, description = "All Submodels in ValueOnly representation"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn get_all_submodels_value_only<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/$reference",
    tag = "Submodel Repository API",
    summary = "Returns the References for all Submodels",
    responses(
        (status = 200, description = "References of all Submodels"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn get_all_submodels_reference<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/$path",
    tag = "Submodel Repository API",
    summary = "Returns all Submodels in the Path notation",
    responses(
        (status = 200, description = "All Submodels in Path notation"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn get_all_submodels_path<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}",
    tag = "Submodel Repository API",
    summary = "Returns a specific Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Requested Submodel"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_by_id<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/submodels/{submodelIdentifier}",
    tag = "Submodel Repository API",
    summary = "Creates or updates an existing Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Submodel updated successfully"),
        (status = 201, description = "Submodel created successfully"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn put_submodel_by_id<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodels/{submodelIdentifier}",
    tag = "Submodel Repository API",
    summary = "Updates an existing Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 204, description = "Submodel updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn patch_submodel_by_id<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/submodels/{submodelIdentifier}",
    tag = "Submodel Repository API",
    summary = "Deletes a Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 204, description = "Submodel deleted successfully"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn delete_submodel_by_id<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/$metadata",
    tag = "Submodel Repository API",
    summary = "Returns the metadata attributes of a specific Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Requested Submodel metadata"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_by_id_metadata<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodels/{submodelIdentifier}/$metadata",
    tag = "Submodel Repository API",
    summary = "Updates the metadata attributes of an existing Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 204, description = "Submodel metadata updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn patch_submodel_by_id_metadata<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/$value",
    tag = "Submodel Repository API",
    summary = "Returns a specific Submodel in the ValueOnly representation",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Requested Submodel in ValueOnly representation"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_by_id_value_only<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodels/{submodelIdentifier}/$value",
    tag = "Submodel Repository API",
    summary = "Updates the values of an existing Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 204, description = "Submodel values updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn patch_submodel_by_id_value_only<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/$reference",
    tag = "Submodel Repository API",
    summary = "Returns the Reference of a specific Submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Requested Submodel reference"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_by_id_reference<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/$path",
    tag = "Submodel Repository API",
    summary = "Returns a specific Submodel in the Path notation",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Requested Submodel path"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_by_id_path<S: SubmodelRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/submodel-elements",
    tag = "Submodel Repository API",
    summary = "Returns all submodel elements including their hierarchy",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "List of all submodel elements"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_submodel_repository<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodels/{submodelIdentifier}/submodel-elements",
    tag = "Submodel Repository API",
    summary = "Creates a new submodel element",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 201, description = "Submodel element created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found"),
        (status = 409, description = "Submodel element already exists")
    )
)]
pub async fn post_submodel_element_submodel_repo<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/submodel-elements/$metadata",
    tag = "Submodel Repository API",
    summary = "Returns the metadata attributes of all submodel elements including their hierarchy",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Metadata of all submodel elements"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_metadata_submodel_repo<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/submodel-elements/$value",
    tag = "Submodel Repository API",
    summary = "Returns all submodel elements including their hierarchy in the ValueOnly representation",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "All submodel elements in ValueOnly representation"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_value_only_submodel_repo<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/submodel-elements/$reference",
    tag = "Submodel Repository API",
    summary = "Returns the References of all submodel elements",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "References of all submodel elements"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_reference_submodel_repo<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodels/{submodelIdentifier}/submodel-elements/$path",
    tag = "Submodel Repository API",
    summary = "Returns all submodel elements including their hierarchy in the Path notation",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "All submodel elements in Path notation"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_path_submodel_repo<S: SubmodelRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

pub fn router(service: impl SubmodelRepositoryService) -> OpenApiRouter {
    OpenApiRouter::new()
        // /submodels Pfadgruppe
        .routes(routes!(get_all_submodels, post_submodel,))
        // /submodels/$metadata Pfad
        .routes(routes!(get_all_submodels_metadata))
        // /submodels/$value Pfad
        .routes(routes!(get_all_submodels_value_only))
        // /submodels/$reference Pfad
        .routes(routes!(get_all_submodels_reference))
        // /submodels/$path Pfad
        .routes(routes!(get_all_submodels_path))
        // /submodels/{submodelIdentifier} Pfadgruppe
        .routes(routes!(
            get_submodel_by_id,
            put_submodel_by_id,
            patch_submodel_by_id,
            delete_submodel_by_id,
        ))
        // /submodels/{submodelIdentifier}/$metadata Pfadgruppe
        .routes(routes!(
            get_submodel_by_id_metadata,
            patch_submodel_by_id_metadata,
        ))
        // /submodels/{submodelIdentifier}/$value Pfadgruppe
        .routes(routes!(
            get_submodel_by_id_value_only,
            patch_submodel_by_id_value_only,
        ))
        // /submodels/{submodelIdentifier}/$reference Pfad
        .routes(routes!(get_submodel_by_id_reference))
        // /submodels/{submodelIdentifier}/$path Pfad
        .routes(routes!(get_submodel_by_id_path))
        // /submodels/{submodelIdentifier}/submodel-elements Pfadgruppe
        .routes(routes!(
            get_all_submodel_elements_submodel_repository,
            post_submodel_element_submodel_repo,
        ))
        // /submodels/{submodelIdentifier}/submodel-elements/$metadata Pfad
        .routes(routes!(get_all_submodel_elements_metadata_submodel_repo))
        // /submodels/{submodelIdentifier}/submodel-elements/$value Pfad
        .routes(routes!(get_all_submodel_elements_value_only_submodel_repo))
        // /submodels/{submodelIdentifier}/submodel-elements/$reference Pfad
        .routes(routes!(get_all_submodel_elements_reference_submodel_repo))
        // /submodels/{submodelIdentifier}/submodel-elements/$path Pfad
        .routes(routes!(get_all_submodel_elements_path_submodel_repo))
        .with_state(Arc::new(service))
}
