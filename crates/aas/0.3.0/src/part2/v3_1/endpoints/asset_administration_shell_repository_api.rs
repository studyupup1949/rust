//! Asset Administration Shell Repository API

use crate::part2::v3_1::services::AASRepositoryService;
use axum::extract::State;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/shells",
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "List of all Asset Administration Shells")
    )
)]
pub async fn get_all_asset_administration_shells<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells",
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 201, description = "Asset Administration Shell created successfully")
    )
)]
pub async fn post_asset_administration_shell<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/$reference",
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "References to all Asset Administration Shells")
    )
)]
pub async fn get_all_asset_administration_shells_reference<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Asset Administration Shell retrieved successfully")
    )
)]
pub async fn get_asset_administration_shell_by_id<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shells/{aasIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Asset Administration Shell updated successfully"),
        (status = 201, description = "Asset Administration Shell created successfully")
    )
)]
pub async fn put_asset_administration_shell_by_id<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shells/{aasIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Asset Administration Shell deleted successfully")
    )
)]
pub async fn delete_asset_administration_shell_by_id<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/$reference",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Asset Administration Shell reference retrieved successfully")
    )
)]
pub async fn get_asset_administration_shell_by_id_reference_aas_repository<
    S: AASRepositoryService,
>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/asset-information",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Asset Information retrieved successfully")
    )
)]
pub async fn get_asset_information_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shells/{aasIdentifier}/asset-information",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Asset Information updated successfully")
    )
)]
pub async fn put_asset_information_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/asset-information/thumbnail",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Thumbnail retrieved successfully")
    )
)]
pub async fn get_thumbnail_aas_repository<S: AASRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shells/{aasIdentifier}/asset-information/thumbnail",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Thumbnail updated successfully")
    )
)]
pub async fn put_thumbnail_aas_repository<S: AASRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shells/{aasIdentifier}/asset-information/thumbnail",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Thumbnail deleted successfully")
    )
)]
pub async fn delete_thumbnail_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodel-refs",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "List of all submodel references")
    )
)]
pub async fn get_all_submodel_references_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells/{aasIdentifier}/submodel-refs",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 201, description = "Submodel reference created successfully")
    )
)]
pub async fn post_submodel_reference_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shells/{aasIdentifier}/submodel-refs/{submodelIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel reference deleted successfully")
    )
)]
pub async fn delete_submodel_reference_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel retrieved successfully")
    )
)]
pub async fn get_submodel_by_id_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel updated successfully")
    )
)]
pub async fn put_submodel_by_id_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel updated successfully")
    )
)]
pub async fn patch_submodel_aas_repository<S: AASRepositoryService>(State(_service): State<Arc<S>>) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel deleted successfully")
    )
)]
pub async fn delete_submodel_by_id_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/$metadata",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel metadata retrieved successfully")
    )
)]
pub async fn get_submodel_by_id_metadata_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/$metadata",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel metadata updated successfully")
    )
)]
pub async fn patch_submodel_by_id_metadata_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel value retrieved successfully")
    )
)]
pub async fn get_submodel_by_id_value_only_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel value updated successfully")
    )
)]
pub async fn patch_submodel_by_id_value_only_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/$reference",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel reference retrieved successfully")
    )
)]
pub async fn get_submodel_by_id_reference_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/$path",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel path retrieved successfully")
    )
)]
pub async fn get_submodel_by_id_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "List of all submodel elements")
    )
)]
pub async fn get_all_submodel_elements_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 201, description = "Submodel element created successfully")
    )
)]
pub async fn post_submodel_element_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$metadata",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel elements metadata retrieved successfully")
    )
)]
pub async fn get_all_submodel_elements_metadata_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel elements value retrieved successfully")
    )
)]
pub async fn get_all_submodel_elements_value_only_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$reference",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel elements references retrieved successfully")
    )
)]
pub async fn get_all_submodel_elements_reference_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$path",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel elements path retrieved successfully")
    )
)]
pub async fn get_all_submodel_elements_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel element retrieved successfully")
    )
)]
pub async fn get_submodel_element_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 201, description = "Submodel element created successfully")
    )
)]
pub async fn post_submodel_element_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel element updated successfully")
    )
)]
pub async fn put_submodel_element_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel element value updated successfully")
    )
)]
pub async fn patch_submodel_element_value_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel element deleted successfully")
    )
)]
pub async fn delete_submodel_element_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$metadata",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel element metadata retrieved successfully")
    )
)]
pub async fn get_submodel_element_by_path_metadata_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$metadata",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel element metadata updated successfully")
    )
)]
pub async fn patch_submodel_element_value_by_path_metadata<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel element value retrieved successfully")
    )
)]
pub async fn get_submodel_element_by_path_value_only_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    patch,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "Submodel element value updated successfully")
    )
)]
pub async fn patch_submodel_element_value_by_path_value_only<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$reference",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel element reference retrieved successfully")
    )
)]
pub async fn get_submodel_element_by_path_reference_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$path",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Submodel element path retrieved successfully")
    )
)]
pub async fn get_submodel_element_by_path_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/attachment",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "File content downloaded successfully")
    )
)]
pub async fn get_file_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    put,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/attachment",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "File content uploaded successfully")
    )
)]
pub async fn put_file_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    delete,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/attachment",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the submodel element")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 204, description = "File content deleted successfully")
    )
)]
pub async fn delete_file_by_path_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/invoke",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the operation")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Operation invoked successfully")
    )
)]
pub async fn invoke_operation_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/invoke/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the operation")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Operation invoked successfully (ValueOnly)")
    )
)]
pub async fn invoke_operation_value_only_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/invoke-async",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the operation")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Asynchronous operation invoked successfully")
    )
)]
pub async fn invoke_operation_async_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/invoke-async/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the operation")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Asynchronous operation invoked successfully (ValueOnly)")
    )
)]
pub async fn invoke_operation_async_value_only_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/operation-status/{handleId}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the operation"),
        ("handleId" = String, Path, description = "Handle ID of the async operation")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Operation status retrieved successfully")
    )
)]
pub async fn get_operation_async_status_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/operation-results/{handleId}",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the operation"),
        ("handleId" = String, Path, description = "Handle ID of the async operation")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Operation result retrieved successfully")
    )
)]
pub async fn get_operation_async_result_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    get,
    path = "/shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/operation-results/{handleId}/$value",
    params(
        ("aasIdentifier" = String, Path, description = "Asset Administration Shell ID"),
        ("submodelIdentifier" = String, Path, description = "Submodel ID"),
        ("idShortPath" = String, Path, description = "IdShort path to the operation"),
        ("handleId" = String, Path, description = "Handle ID of the async operation")
    ),
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Operation result retrieved successfully (ValueOnly)")
    )
)]
pub async fn get_operation_async_result_value_only_aas_repository<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

#[utoipa::path(
    post,
    path = "/query/shells",
    tag = "Asset Administration Shell Repository API",
    responses(
        (status = 200, description = "Query results for Asset Administration Shells")
    )
)]
pub async fn query_asset_administration_shells<S: AASRepositoryService>(
    State(_service): State<Arc<S>>,
) {
    unimplemented!()
}

// Define OpenApi documentation object including all paths
#[derive(OpenApi)]
#[openapi(paths(
    get_all_asset_administration_shells,
    post_asset_administration_shell,
    get_all_asset_administration_shells_reference,
    get_asset_administration_shell_by_id,
    put_asset_administration_shell_by_id,
    delete_asset_administration_shell_by_id,
    get_asset_administration_shell_by_id_reference_aas_repository,
    get_asset_information_aas_repository,
    put_asset_information_aas_repository,
    get_thumbnail_aas_repository,
    put_thumbnail_aas_repository,
    delete_thumbnail_aas_repository,
    get_all_submodel_references_aas_repository,
    post_submodel_reference_aas_repository,
    delete_submodel_reference_aas_repository,
    get_submodel_by_id_aas_repository,
    put_submodel_by_id_aas_repository,
    patch_submodel_aas_repository,
    delete_submodel_by_id_aas_repository,
    get_submodel_by_id_metadata_aas_repository,
    patch_submodel_by_id_metadata_aas_repository,
    get_submodel_by_id_value_only_aas_repository,
    patch_submodel_by_id_value_only_aas_repository,
    get_submodel_by_id_reference_aas_repository,
    get_submodel_by_id_path_aas_repository,
    get_all_submodel_elements_aas_repository,
    post_submodel_element_aas_repository,
    get_all_submodel_elements_metadata_aas_repository,
    get_all_submodel_elements_value_only_aas_repository,
    get_all_submodel_elements_reference_aas_repository,
    get_all_submodel_elements_path_aas_repository,
    get_submodel_element_by_path_aas_repository,
    post_submodel_element_by_path_aas_repository,
    put_submodel_element_by_path_aas_repository,
    patch_submodel_element_value_by_path_aas_repository,
    delete_submodel_element_by_path_aas_repository,
    get_submodel_element_by_path_metadata_aas_repository,
    patch_submodel_element_value_by_path_metadata,
    get_submodel_element_by_path_value_only_aas_repository,
    patch_submodel_element_value_by_path_value_only,
    get_submodel_element_by_path_reference_aas_repository,
    get_submodel_element_by_path_path_aas_repository,
    get_file_by_path_aas_repository,
    put_file_by_path_aas_repository,
    delete_file_by_path_aas_repository,
    invoke_operation_aas_repository,
    invoke_operation_value_only_aas_repository,
    invoke_operation_async_aas_repository,
    invoke_operation_async_value_only_aas_repository,
    get_operation_async_status_aas_repository,
    get_operation_async_result_aas_repository,
    get_operation_async_result_value_only_aas_repository,
    query_asset_administration_shells,
))]
pub struct AASRepositoryAPI;

// Create router using utoipa_axum OpenApiRouter
pub fn router(service: impl AASRepositoryService) -> OpenApiRouter {
    OpenApiRouter::new()
        // Gruppe: /shells (Root)
        .routes(routes!(
            get_all_asset_administration_shells,
            post_asset_administration_shell,
        ))
        // Gruppe: /shells/$reference
        .routes(routes!(get_all_asset_administration_shells_reference))
        // Gruppe: /shells/{aasIdentifier}
        .routes(routes!(
            get_asset_administration_shell_by_id,
            put_asset_administration_shell_by_id,
            delete_asset_administration_shell_by_id,
        ))
        // Gruppe: /shells/{aasIdentifier}/$reference
        .routes(routes!(
            get_asset_administration_shell_by_id_reference_aas_repository
        ))
        // Gruppe: /shells/{aasIdentifier}/asset-information
        .routes(routes!(
            get_asset_information_aas_repository,
            put_asset_information_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/asset-information/thumbnail
        .routes(routes!(
            get_thumbnail_aas_repository,
            put_thumbnail_aas_repository,
            delete_thumbnail_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodel-refs
        .routes(routes!(
            get_all_submodel_references_aas_repository,
            post_submodel_reference_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodel-refs/{submodelIdentifier}
        .routes(routes!(delete_submodel_reference_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}
        .routes(routes!(
            get_submodel_by_id_aas_repository,
            put_submodel_by_id_aas_repository,
            patch_submodel_aas_repository,
            delete_submodel_by_id_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/$metadata
        .routes(routes!(
            get_submodel_by_id_metadata_aas_repository,
            patch_submodel_by_id_metadata_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/$value
        .routes(routes!(
            get_submodel_by_id_value_only_aas_repository,
            patch_submodel_by_id_value_only_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/$reference
        .routes(routes!(get_submodel_by_id_reference_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/$path
        .routes(routes!(get_submodel_by_id_path_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements
        .routes(routes!(
            get_all_submodel_elements_aas_repository,
            post_submodel_element_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$metadata
        .routes(routes!(get_all_submodel_elements_metadata_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$value
        .routes(routes!(get_all_submodel_elements_value_only_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$reference
        .routes(routes!(get_all_submodel_elements_reference_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/$path
        .routes(routes!(get_all_submodel_elements_path_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}
        .routes(routes!(
            get_submodel_element_by_path_aas_repository,
            post_submodel_element_by_path_aas_repository,
            put_submodel_element_by_path_aas_repository,
            patch_submodel_element_value_by_path_aas_repository,
            delete_submodel_element_by_path_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$metadata
        .routes(routes!(
            get_submodel_element_by_path_metadata_aas_repository,
            patch_submodel_element_value_by_path_metadata,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$value
        .routes(routes!(
            get_submodel_element_by_path_value_only_aas_repository,
            patch_submodel_element_value_by_path_value_only,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$reference
        .routes(routes!(
            get_submodel_element_by_path_reference_aas_repository
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/$path
        .routes(routes!(get_submodel_element_by_path_path_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/attachment
        .routes(routes!(
            get_file_by_path_aas_repository,
            put_file_by_path_aas_repository,
            delete_file_by_path_aas_repository,
        ))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/invoke
        .routes(routes!(invoke_operation_aas_repository))
        .routes(routes!(invoke_operation_value_only_aas_repository))
        .routes(routes!(invoke_operation_async_aas_repository))
        .routes(routes!(invoke_operation_async_value_only_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/operation-status/{handleId}
        .routes(routes!(get_operation_async_status_aas_repository))
        // Gruppe: /shells/{aasIdentifier}/submodels/{submodelIdentifier}/submodel-elements/{idShortPath}/operation-results/{handleId}
        .routes(routes!(get_operation_async_result_aas_repository,))
        .routes(routes!(
            get_operation_async_result_value_only_aas_repository
        ))
        // Gruppe: /query/shells
        .routes(routes!(query_asset_administration_shells))
        .with_state(Arc::new(service))
}
