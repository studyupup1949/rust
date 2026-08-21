use serde::Deserialize;

/// Query parameters for the list view.
#[derive(Debug, Default, Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

/// Results of a list operation.
pub struct ListResult {
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}
