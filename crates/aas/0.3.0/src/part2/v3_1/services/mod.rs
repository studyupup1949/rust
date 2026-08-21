use axum::http::StatusCode;

pub trait AASXFileServerService: Send + Sync + 'static {
    async fn get_all_aasx_package_ids(&self) -> StatusCode;
}

pub trait AASShellService: Send + Sync + 'static {}

pub trait AASBasicDiscoveryService: Send + Sync + 'static {
    // Methods for discovery
}

pub trait AASRegistryService: Send + Sync + 'static {
    // Registry methods
}

pub trait AASRepositoryService: Send + Sync + 'static {
    // Repository methods
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
    async fn smth();
}
