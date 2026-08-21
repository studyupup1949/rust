//! Submodel Registry API

use crate::part2::v3_1::services::SubmodelRegistryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/submodel-descriptors",
    tag = "Submodel Registry API",
    summary = "Returns all Submodel Descriptors",
    responses(
        (status = 200, description = "List of Submodel Descriptors returned successfully"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn get_all_submodel_descriptors<S: SubmodelRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodel-descriptors",
    tag = "Submodel Registry API",
    summary = "Creates a new Submodel Descriptor, i.e. registers a submodel",
    responses(
        (status = 201, description = "Submodel Descriptor created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 409, description = "Conflict - Submodel Descriptor with same identifier already exists")
    )
)]
pub async fn post_submodel_descriptor<S: SubmodelRegistryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel-descriptors/{submodelIdentifier}",
    tag = "Submodel Registry API",
    summary = "Returns a specific Submodel Descriptor",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Requested Submodel Descriptor"),
        (status = 404, description = "Submodel Descriptor not found")
    )
)]
pub async fn get_submodel_descriptor_by_id<S: SubmodelRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/submodel-descriptors/{submodelIdentifier}",
    tag = "Submodel Registry API",
    summary = "Creates or updates an existing Submodel Descriptor",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Submodel Descriptor updated successfully"),
        (status = 201, description = "Submodel Descriptor created successfully"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn put_submodel_descriptor_by_id<S: SubmodelRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/submodel-descriptors/{submodelIdentifier}",
    tag = "Submodel Registry API",
    summary = "Deletes a Submodel Descriptor, i.e. de-registers a submodel",
    params(
        ("submodelIdentifier" = String, Path, description = "The Submodel's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 204, description = "Submodel Descriptor deleted successfully"),
        (status = 404, description = "Submodel Descriptor not found")
    )
)]
pub async fn delete_submodel_descriptor_by_id<S: SubmodelRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/query/submodel-descriptors",
    tag = "Submodel Registry API",
    summary = "Returns all Submodel Descriptors that confirm to the input query",
    responses(
        (status = 200, description = "Query results returned successfully"),
        (status = 400, description = "Bad Request - Invalid query syntax")
    )
)]
pub async fn query_submodel_descriptors<S: SubmodelRegistryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

pub fn router(service: impl SubmodelRegistryService) -> OpenApiRouter {
    OpenApiRouter::new()
        // /submodel-descriptors Pfadgruppe
        .routes(routes!(
            get_all_submodel_descriptors,
            post_submodel_descriptor,
        ))
        // /submodel-descriptors/{submodelIdentifier} Pfadgruppe
        .routes(routes!(
            get_submodel_descriptor_by_id,
            put_submodel_descriptor_by_id,
            delete_submodel_descriptor_by_id,
        ))
        // /query/submodel-descriptors Pfadgruppe
        .routes(routes!(query_submodel_descriptors))
        .with_state(Arc::new(service))
}
