#[allow(async_fn_in_trait)]
use crate::part1::v3_1::core::AssetAdministrationShell;
use crate::part1::v3_1::core::Submodel;
use crate::part1::v3_1::reference::Reference;
use crate::part2::v3_1::error::AASError;
use axum::Json;
use axum::extract::Multipart;
use axum::http::StatusCode;

pub trait AASXFileServerService: Send + Sync + 'static {
    fn get_all_aasx_package_ids(
        &self,
    ) -> impl Future<Output = Result<Json<String>, AASError>> + Send;
}

pub trait AASShellService: Send + Sync + 'static {
    /// Get all aas
    fn find_all_aas(
        &self,
    ) -> impl Future<Output = Result<Vec<AssetAdministrationShell>, AASError>> + Send;

    fn create_or_update_aas(
        &self,
        aas: &AssetAdministrationShell,
    ) -> impl Future<Output = Result<StatusCode, AASError>> + Send;

    // maximum flexibility with return type "response", so body and headers can are editable by the
    // implementing service
    fn get_thumbnail(
        &self,
        aas_id: String,
    ) -> impl Future<Output = Result<axum::response::Response, AASError>> + Send;

    fn put_thumbnail(
        &self,
        aas_id: String,
        thumbnail: Multipart,
    ) -> impl Future<Output = Result<(), AASError>> + Send;
}

pub trait AASBasicDiscoveryService: Send + Sync + 'static {
    // Methods for discovery
}

pub trait AASRegistryService: Send + Sync + 'static {
    // Registry methods
}

pub trait AASRepositoryService: Send + Sync + 'static {
    // Repository methods
    fn find_all_aas(
        &self,
    ) -> impl Future<Output = Result<Vec<AssetAdministrationShell>, AASError>> + Send;

    fn create_aas(
        &self,
        aas: &AssetAdministrationShell,
    ) -> impl Future<Output = Result<(), AASError>> + Send;

    fn get_aas_as_references(
        &self,
        asset_ids: Option<Vec<String>>,
        id_short: Option<String>,
        limit: Option<usize>,
        cursor: Option<usize>,
    ) -> impl Future<Output = Result<Vec<Reference>, AASError>> + Send;

    fn get_thumbnail(
        &self,
        aas_id: String,
    ) -> impl Future<Output = Result<Vec<u8>, AASError>> + Send;
}

pub trait AsyncBulkAASRegistryService: Send + Sync + 'static {
    // Bulk registry methods
}

pub trait AsyncBulkResultService: Send + Sync + 'static {
    // Bulk result handling
}

pub trait AsyncBulkStatusService: Send + Sync + 'static {
    // Status checking
}

pub trait AsyncBulkSubmodelRegistryService: Send + Sync + 'static {
    // Submodel bulk registry
}

pub trait ConceptDescriptionRepositoryService: Send + Sync + 'static {
    // Concepts methods
}

pub trait DescriptionService: Send + Sync + 'static {
    // Description methods
}

pub trait SerializationService: Send + Sync + 'static {
    // Serialization methods
}

pub trait SubmodelService: Send + Sync + 'static {
    // Submodel methods
}

pub trait SubmodelRegistryService: Send + Sync + 'static {
    // Registry methods
}

pub trait SubmodelRepositoryService: Send + Sync + 'static {
    fn find_all_submodels(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Submodel>, AASError>> + Send;
}
