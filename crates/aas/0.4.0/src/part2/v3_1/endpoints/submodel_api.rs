//! Submodel API

use crate::part2::v3_1::services::SubmodelService;
use axum::extract::State;
use std::sync::Arc;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/submodel",
    tag = "Submodel API",
    summary = "Returns the Submodel",
    responses(
        (status = 200, description = "Requested Submodel"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/submodel",
    tag = "Submodel API",
    summary = "Updates the Submodel",
    responses(
        (status = 204, description = "Submodel updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn put_submodel<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodel",
    tag = "Submodel API",
    summary = "Updates the Submodel",
    responses(
        (status = 204, description = "Submodel updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn patch_submodel<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/$metadata",
    tag = "Submodel API",
    summary = "Returns the metadata attributes of a specific Submodel",
    responses(
        (status = 200, description = "Requested Submodel metadata"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_metadata<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodel/$metadata",
    tag = "Submodel API",
    summary = "Updates the metadata attributes of the Submodel",
    responses(
        (status = 204, description = "Submodel metadata updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn patch_submodel_metadata<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/$value",
    tag = "Submodel API",
    summary = "Returns the Submodel in the ValueOnly representation",
    responses(
        (status = 200, description = "Requested Submodel in ValueOnly representation"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_value_only<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodel/$value",
    tag = "Submodel API",
    summary = "Updates the values of the Submodel",
    responses(
        (status = 204, description = "Submodel values updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn patch_submodel_value_only<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/$reference",
    tag = "Submodel API",
    summary = "Returns the Reference of the Submodel",
    responses(
        (status = 200, description = "Requested Submodel reference"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_reference<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/$path",
    tag = "Submodel API",
    summary = "Returns the Submodel in the Path notation",
    responses(
        (status = 200, description = "Requested Submodel path"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_submodel_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements",
    tag = "Submodel API",
    summary = "Returns all submodel elements including their hierarchy",
    responses(
        (status = 200, description = "List of all submodel elements"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodel/submodel-elements",
    tag = "Submodel API",
    summary = "Creates a new submodel element",
    responses(
        (status = 201, description = "Submodel element created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel not found"),
        (status = 409, description = "Submodel element already exists")
    )
)]
pub async fn post_submodel_element<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/$metadata",
    tag = "Submodel API",
    summary = "Returns the metadata attributes of all submodel elements including their hierarchy",
    responses(
        (status = 200, description = "Metadata of all submodel elements"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_metadata<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/$value",
    tag = "Submodel API",
    summary = "Returns all submodel elements including their hierarchy in the ValueOnly representation",
    responses(
        (status = 200, description = "All submodel elements in ValueOnly representation"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_value_only<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/$reference",
    tag = "Submodel API",
    summary = "Returns the References of all submodel elements",
    responses(
        (status = 200, description = "References of all submodel elements"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_reference<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/$path",
    tag = "Submodel API",
    summary = "Returns all submodel elements including their hierarchy in the Path notation",
    responses(
        (status = 200, description = "All submodel elements in Path notation"),
        (status = 404, description = "Submodel not found")
    )
)]
pub async fn get_all_submodel_elements_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}",
    tag = "Submodel API",
    summary = "Returns a specific submodel element from the Submodel at a specified path",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "Requested submodel element"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn get_submodel_element_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodel/submodel-elements/{idShortPath}",
    tag = "Submodel API",
    summary = "Creates a new submodel element at a specified path within submodel elements hierarchy",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 201, description = "Submodel element created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Parent element not found"),
        (status = 409, description = "Submodel element already exists")
    )
)]
pub async fn post_submodel_element_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/submodel/submodel-elements/{idShortPath}",
    tag = "Submodel API",
    summary = "Creates or updates an existing submodel element at a specified path within submodel elements hierarchy",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "Submodel element updated successfully"),
        (status = 201, description = "Submodel element created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Parent element not found")
    )
)]
pub async fn put_submodel_element_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodel/submodel-elements/{idShortPath}",
    tag = "Submodel API",
    summary = "Updates an existing SubmodelElement",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 204, description = "Submodel element updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn patch_submodel_element_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/submodel/submodel-elements/{idShortPath}",
    tag = "Submodel API",
    summary = "Deletes a submodel element at a specified path within the submodel elements hierarchy",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 204, description = "Submodel element deleted successfully"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn delete_submodel_element_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/$metadata",
    tag = "Submodel API",
    summary = "Returns the metadata attributes of a specific submodel element from the Submodel at a specified path",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "Requested submodel element metadata"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn get_submodel_element_by_path_metadata<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodel/submodel-elements/{idShortPath}/$metadata",
    tag = "Submodel API",
    summary = "Updates the metadata attributes an existing SubmodelElement",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 204, description = "Submodel element metadata updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn patch_submodel_element_by_path_metadata<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/$value",
    tag = "Submodel API",
    summary = "Returns a specific submodel element from the Submodel at a specified path in the ValueOnly representation",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "Requested submodel element in ValueOnly representation"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn get_submodel_element_by_path_value_only<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/submodel/submodel-elements/{idShortPath}/$value",
    tag = "Submodel API",
    summary = "Updates the value of an existing SubmodelElement",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 204, description = "Submodel element value updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn patch_submodel_element_by_path_value_only<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/$reference",
    tag = "Submodel API",
    summary = "Returns the Reference of a specific submodel element from the Submodel at a specified path",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "Requested submodel element reference"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn get_submodel_element_by_path_reference<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/$path",
    tag = "Submodel API",
    summary = "Returns a specific submodel element from the Submodel at a specified path in the Path notation",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "Requested submodel element path"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn get_submodel_element_by_path_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/attachment",
    tag = "Submodel API",
    summary = "Downloads file content from a specific submodel element from the Submodel at a specified path",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "File content downloaded successfully"),
        (status = 404, description = "Submodel element or file not found")
    )
)]
pub async fn get_file_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/submodel/submodel-elements/{idShortPath}/attachment",
    tag = "Submodel API",
    summary = "Uploads file content to an existing submodel element at a specified path within submodel elements hierarchy",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 200, description = "File content uploaded successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Submodel element not found")
    )
)]
pub async fn put_file_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/submodel/submodel-elements/{idShortPath}/attachment",
    tag = "Submodel API",
    summary = "Deletes file content of an existing submodel element at a specified path within submodel elements hierarchy",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element (dot-separated)")
    ),
    responses(
        (status = 204, description = "File content deleted successfully"),
        (status = 404, description = "Submodel element or file not found")
    )
)]
pub async fn delete_file_by_path<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodel/submodel-elements/{idShortPath}/invoke",
    tag = "Submodel API",
    summary = "Synchronously invokes an Operation at a specified path",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the operation (dot-separated)")
    ),
    responses(
        (status = 200, description = "Operation invoked successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Operation not found"),
        (status = 500, description = "Operation execution failed")
    )
)]
pub async fn invoke_operation<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodel/submodel-elements/{idShortPath}/invoke/$value",
    tag = "Submodel API",
    summary = "Synchronously invokes an Operation at a specified path with ValueOnly representation",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the operation (dot-separated)")
    ),
    responses(
        (status = 200, description = "Operation invoked successfully with ValueOnly result"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Operation not found"),
        (status = 500, description = "Operation execution failed")
    )
)]
pub async fn invoke_operation_sync_value_only<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodel/submodel-elements/{idShortPath}/invoke-async",
    tag = "Submodel API",
    summary = "Asynchronously invokes an Operation at a specified path",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the operation (dot-separated)")
    ),
    responses(
        (status = 202, description = "Operation invocation accepted"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Operation not found")
    )
)]
pub async fn invoke_operation_async<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/submodel/submodel-elements/{idShortPath}/invoke-async/$value",
    tag = "Submodel API",
    summary = "Asynchronously invokes an Operation at a specified path with ValueOnly representation",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the operation (dot-separated)")
    ),
    responses(
        (status = 202, description = "Operation invocation accepted"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Operation not found")
    )
)]
pub async fn invoke_operation_async_value_only<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/operation-status/{handleId}",
    tag = "Submodel API",
    summary = "Returns the status of an asynchronously invoked Operation",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the operation (dot-separated)"),
        ("handleId" = String, Path, description = "Handle ID of the asynchronous operation invocation")
    ),
    responses(
        (status = 200, description = "Operation status retrieved successfully"),
        (status = 404, description = "Operation or handle ID not found")
    )
)]
pub async fn get_operation_async_status<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/operation-results/{handleId}",
    tag = "Submodel API",
    summary = "Returns the Operation result of an asynchronously invoked Operation",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the operation (dot-separated)"),
        ("handleId" = String, Path, description = "Handle ID of the asynchronous operation invocation")
    ),
    responses(
        (status = 200, description = "Operation result retrieved successfully"),
        (status = 404, description = "Operation or handle ID not found")
    )
)]
pub async fn get_operation_async_result<S: SubmodelService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/submodel/submodel-elements/{idShortPath}/operation-results/{handleId}/$value",
    tag = "Submodel API",
    summary = "Returns the value of the Operation result of an asynchronously invoked Operation",
    params(
        ("idShortPath" = String, Path, description = "IdShort path to the operation (dot-separated)"),
        ("handleId" = String, Path, description = "Handle ID of the asynchronous operation invocation")
    ),
    responses(
        (status = 200, description = "Operation result value retrieved successfully"),
        (status = 404, description = "Operation or handle ID not found")
    )
)]
pub async fn get_operation_async_result_value_only<S: SubmodelService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

pub fn router(service: impl SubmodelService) -> OpenApiRouter {
    OpenApiRouter::new()
        // /submodel Pfadgruppe
        .routes(routes!(get_submodel, put_submodel, patch_submodel,))
        // /submodel/$metadata Pfadgruppe
        .routes(routes!(get_submodel_metadata, patch_submodel_metadata,))
        // /submodel/$value Pfadgruppe
        .routes(routes!(get_submodel_value_only, patch_submodel_value_only,))
        // /submodel/$reference Pfad
        .routes(routes!(get_submodel_reference))
        // /submodel/$path Pfad
        .routes(routes!(get_submodel_path))
        // /submodel/submodel-elements Pfadgruppe
        .routes(routes!(get_all_submodel_elements, post_submodel_element,))
        // /submodel/submodel-elements/$metadata Pfad
        .routes(routes!(get_all_submodel_elements_metadata))
        // /submodel/submodel-elements/$value Pfad
        .routes(routes!(get_all_submodel_elements_value_only))
        // /submodel/submodel-elements/$reference Pfad
        .routes(routes!(get_all_submodel_elements_reference))
        // /submodel/submodel-elements/$path Pfad
        .routes(routes!(get_all_submodel_elements_path))
        // /submodel/submodel-elements/{idShortPath} Pfadgruppe
        .routes(routes!(
            get_submodel_element_by_path,
            post_submodel_element_by_path,
            put_submodel_element_by_path,
            patch_submodel_element_by_path,
            delete_submodel_element_by_path,
        ))
        // /submodel/submodel-elements/{idShortPath}/$metadata Pfadgruppe
        .routes(routes!(
            get_submodel_element_by_path_metadata,
            patch_submodel_element_by_path_metadata,
        ))
        // /submodel/submodel-elements/{idShortPath}/$value Pfadgruppe
        .routes(routes!(
            get_submodel_element_by_path_value_only,
            patch_submodel_element_by_path_value_only,
        ))
        // /submodel/submodel-elements/{idShortPath}/$reference Pfad
        .routes(routes!(get_submodel_element_by_path_reference))
        // /submodel/submodel-elements/{idShortPath}/$path Pfad
        .routes(routes!(get_submodel_element_by_path_path))
        // /submodel/submodel-elements/{idShortPath}/attachment Pfadgruppe
        .routes(routes!(
            get_file_by_path,
            put_file_by_path,
            delete_file_by_path,
        ))
        // /submodel/submodel-elements/{idShortPath}/invoke Pfadgruppe
        .routes(routes!(invoke_operation,))
        .routes(routes!(invoke_operation_sync_value_only))
        .routes(routes!(invoke_operation_async))
        .routes(routes!(invoke_operation_async_value_only))
        // /submodel/submodel-elements/{idShortPath}/operation-status/{handleId} Pfad
        .routes(routes!(get_operation_async_status))
        // /submodel/submodel-elements/{idShortPath}/operation-results/{handleId} Pfadgruppe
        .routes(routes!(get_operation_async_result,))
        .routes(routes!(get_operation_async_result_value_only))
        .with_state(Arc::new(service))
}
