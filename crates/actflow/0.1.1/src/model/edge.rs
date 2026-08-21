use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeModel {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_handle: String,
}
