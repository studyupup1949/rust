use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ActflowError, Result,
    model::{EdgeModel, NodeModel},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowModel {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub env: HashMap<String, String>,
    pub nodes: Vec<NodeModel>,
    pub edges: Vec<EdgeModel>,
}

impl WorkflowModel {
    pub fn from_json(s: &str) -> Result<Self> {
        let workflow = serde_json::from_str::<WorkflowModel>(s);
        match workflow {
            Ok(v) => Ok(v),
            Err(e) => Err(ActflowError::Workflow(format!("{}", e))),
        }
    }
}
