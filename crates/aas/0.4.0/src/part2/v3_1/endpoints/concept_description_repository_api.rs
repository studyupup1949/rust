//! Concept Description Repository API

use crate::part2::v3_1::services::ConceptDescriptionRepositoryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/concept-descriptions",
    tag = "Concept Description Repository API",
    summary = "Returns all Concept Descriptions",
    responses(
        (status = 200, description = "List of Concept Descriptions")
    )
)]
pub async fn get_all_concept_descriptions<S: ConceptDescriptionRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/concept-descriptions",
    tag = "Concept Description Repository API",
    summary = "Creates a new Concept Description",
    responses(
        (status = 201, description = "Concept Description created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 409, description = "Concept Description already exists")
    )
)]
pub async fn post_concept_description<S: ConceptDescriptionRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/concept-descriptions/{cdIdentifier}",
    tag = "Concept Description Repository API",
    summary = "Returns a specific Concept Description",
    params(
        ("cdIdentifier" = String, Path, description = "The Concept Description's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Requested Concept Description"),
        (status = 404, description = "Concept Description not found")
    )
)]
pub async fn get_concept_description_by_id<S: ConceptDescriptionRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/concept-descriptions/{cdIdentifier}",
    tag = "Concept Description Repository API",
    summary = "Creates or updates an existing Concept Description",
    params(
        ("cdIdentifier" = String, Path, description = "The Concept Description's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 200, description = "Concept Description updated successfully"),
        (status = 201, description = "Concept Description created successfully"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn put_concept_description_by_id<S: ConceptDescriptionRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/concept-descriptions/{cdIdentifier}",
    tag = "Concept Description Repository API",
    summary = "Deletes a Concept Description",
    params(
        ("cdIdentifier" = String, Path, description = "The Concept Description's unique id (UTF8-BASE64-URL-encoded)")
    ),
    responses(
        (status = 204, description = "Concept Description deleted successfully"),
        (status = 404, description = "Concept Description not found")
    )
)]
pub async fn delete_concept_description_by_id<S: ConceptDescriptionRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/query/concept-descriptions",
    tag = "Concept Description Repository API",
    summary = "Returns all Concept Descriptions that confirm to the input query",
    responses(
        (status = 200, description = "Query results returned successfully"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn query_concept_descriptions<S: ConceptDescriptionRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

pub fn router(service: impl ConceptDescriptionRepositoryService) -> OpenApiRouter {
    OpenApiRouter::new()
        // /concept-descriptions Pfadgruppe
        .routes(routes!(
            get_all_concept_descriptions,
            post_concept_description,
        ))
        // /concept-descriptions/{cdIdentifier} Pfadgruppe
        .routes(routes!(
            get_concept_description_by_id,
            put_concept_description_by_id,
            delete_concept_description_by_id,
        ))
        // /query/concept-descriptions Pfadgruppe
        .routes(routes!(query_concept_descriptions))
        .with_state(Arc::new(service))
}
