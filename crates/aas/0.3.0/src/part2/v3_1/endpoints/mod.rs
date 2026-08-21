mod aasx_file_server_api;
pub use aasx_file_server_api::router as aasx_file_server_api_router;
mod asset_administration_shell_api;
pub use asset_administration_shell_api::router as asset_administration_shell_api_router;
mod asset_administration_shell_basic_discovery_api;
pub use asset_administration_shell_basic_discovery_api::router as asset_administration_shell_basic_discovery_api_router;
mod asset_administration_shell_registry_api;
pub use asset_administration_shell_registry_api::router as asset_administration_shell_registry_api_router;
mod asset_administration_shell_repository_api;
pub use asset_administration_shell_repository_api::router as asset_administration_shell_repository_api_router;
mod async_bulk_asset_administration_shell_registry_api;
pub use async_bulk_asset_administration_shell_registry_api::router as async_bulk_asset_administration_shell_registry_api_router;
mod async_bulk_result_api;
pub use async_bulk_result_api::router as async_bulk_result_api_router;
mod async_bulk_status_api;
pub use async_bulk_status_api::router as async_bulk_status_api_router;
mod async_bulk_submodel_registry_api;
pub use async_bulk_status_api::router as async_bulk_submodel_registry_api_router;
mod concept_description_repository_api;
pub use concept_description_repository_api::router as concept_description_repository_api_router;
mod description_api;
pub use description_api::router as description_api_router;
mod serialization_api;
pub use serialization_api::router as serialization_api_router;
mod submodel_api;
pub use submodel_api::router as submodel_api_router;
mod submodel_registry_api;
pub use submodel_registry_api::router as submodel_registry_api_router;
mod submodel_repository_api;
pub use submodel_repository_api::router as submodel_repository_api_router;

use super::error::AASError;
use super::error::AASErrorMessageType;
use super::error::AASMessage;
use super::services::{AASBasicDiscoveryService, AASRegistryService, AASRepositoryService, AASShellService, AASXFileServerService, AsyncBulkAASRegistryService, AsyncBulkResultService, AsyncBulkStatusService, AsyncBulkSubmodelRegistryService, ConceptDescriptionRepositoryService, DescriptionService, SerializationService, SubmodelRegistryService, SubmodelRepositoryService, SubmodelService};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Asset Administration Shell API",
        version = "3.0",
        description = "The Asset Administration Shell API as defined in the AAS Specification Part 2"
    ),
    tags(
        (name = "AASX File Server API", description = "AASX File Server API endpoints"),
        (name = "Asset Administration Shell API", description = "Asset Administration Shell API endpoints"),
        (name = "Asset Administration Shell Basic Discovery API", description = "Discovery endpoints for Asset Administration Shells"),
        (name = "Asset Administration Shell Registry API", description = "Registry endpoints for Asset Administration Shells"),
        (name = "Asset Administration Shell Repository API", description = "Repository endpoints for Asset Administration Shells"),
        (name = "Async Bulk Asset Administration Shell Registry API", description = "Bulk registry operations"),
        (name = "Async Bulk Result API", description = "Async bulk result endpoints"),
        (name = "Async Bulk Status API", description = "Async bulk status endpoints"),
        (name = "Async Bulk Submodel Registry API", description = "Bulk submodel registry operations"),
        (name = "Concept Description Repository API", description = "Concept Description Repository endpoints"),
        (name = "Description API", description = "Service description endpoints"),
        (name = "Serialization API", description = "Serialization endpoints"),
        (name = "Submodel API", description = "Submodel API endpoints"),
        (name = "Submodel Registry API", description = "Submodel Registry endpoints"),
        (name = "Submodel Repository API", description = "Submodel Repository endpoints")
    ),
    components(
        schemas(AASError, AASMessage, AASErrorMessageType)
    )
)]
pub struct AASServicesAPIDoc;

/// Build the complete API router with all submodules
pub fn build_complete_api_router(
    aasx_file_server_services: impl AASXFileServerService,
    aas_shell_service: impl AASShellService,
    aas_basic_discovery_service: impl AASBasicDiscoveryService,
    aas_registry_service: impl AASRegistryService,
    aas_repository_service: impl AASRepositoryService,
    bulk_aas_registry_service: impl AsyncBulkAASRegistryService,
    bulk_result_service: impl AsyncBulkResultService,
    bulk_status_service: impl AsyncBulkStatusService,
    bulk_submodel_registry_service: impl AsyncBulkSubmodelRegistryService,
    concept_description_repository_service: impl ConceptDescriptionRepositoryService,
    description_service: impl DescriptionService,
    serialization_service: impl SerializationService,
    submodel_service: impl SubmodelService,
    submodel_registry_service: impl SubmodelRegistryService,
    submodel_repository_service: impl SubmodelRepositoryService,
) -> OpenApiRouter {
    OpenApiRouter::new()
        .merge(aasx_file_server_api::router(aasx_file_server_services))
        .merge(asset_administration_shell_api::router(aas_shell_service))
        .merge(asset_administration_shell_basic_discovery_api::router(
            aas_basic_discovery_service,
        ))
        .merge(asset_administration_shell_registry_api::router(
            aas_registry_service,
        ))
        .merge(asset_administration_shell_repository_api::router(
            aas_repository_service,
        ))
        .merge(async_bulk_asset_administration_shell_registry_api::router(
            bulk_aas_registry_service,
        ))
        .merge(async_bulk_result_api::router(bulk_result_service))
        .merge(async_bulk_status_api::router(bulk_status_service))
        .merge(async_bulk_submodel_registry_api::router(
            bulk_submodel_registry_service,
        ))
        .merge(concept_description_repository_api::router(
            concept_description_repository_service,
        ))
        .merge(description_api::router(description_service))
        .merge(serialization_api::router(serialization_service))
        .merge(submodel_api::router(submodel_service))
        .merge(submodel_registry_api::router(submodel_registry_service))
        .merge(submodel_repository_api::router(submodel_repository_service))
}
