//! Workflow model for defining complete workflow structures.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ActflowError, Result,
    model::{EdgeModel, NodeModel},
};

/// Represents a complete workflow definition.
///
/// A workflow consists of nodes (actions) connected by edges (transitions).
/// It can include environment variables that are accessible to all nodes.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use actflow::{WorkflowModel, NodeModel, EdgeModel};
/// use serde_json::json;
///
/// let workflow = WorkflowModel {
///     id: "my_workflow".to_string(),
///     name: "My Workflow".to_string(),
///     desc: "A sample workflow".to_string(),
///     env: HashMap::from([("API_KEY".to_string(), "secret".to_string())]),
///     nodes: vec![
///         NodeModel {
///             id: "start".to_string(),
///             uses: "start".to_string(),
///             action: json!({}),
///             ..Default::default()
///         },
///     ],
///     edges: vec![],
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowModel {
    /// Unique identifier for this workflow.
    pub id: String,
    /// Human-readable name for the workflow.
    pub name: String,
    /// Description of the workflow's purpose.
    pub desc: String,
    /// Environment variables accessible to all nodes via `{{#env.KEY#}}`.
    pub env: HashMap<String, String>,
    /// List of nodes that make up the workflow.
    pub nodes: Vec<NodeModel>,
    /// List of edges defining the execution flow between nodes.
    pub edges: Vec<EdgeModel>,
}

impl WorkflowModel {
    /// Parses a workflow from a JSON string.
    ///
    /// # Arguments
    ///
    /// * `s` - JSON string representing the workflow
    ///
    /// # Returns
    ///
    /// Returns `Ok(WorkflowModel)` on success, or an error if parsing fails.
    pub fn from_json(s: &str) -> Result<Self> {
        let workflow = serde_json::from_str::<WorkflowModel>(s);
        match workflow {
            Ok(v) => Ok(v),
            Err(e) => Err(ActflowError::Workflow(format!("{}", e))),
        }
    }
}
