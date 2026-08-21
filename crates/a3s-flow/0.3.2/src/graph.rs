//! DAG graph representation and validation.
//!
//! [`DagGraph`] parses a JSON flow definition, builds a directed acyclic graph,
//! validates it (no cycles, all referenced node IDs exist), and exposes
//! topological-order iteration for the runner.

use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::condition::Condition;
use crate::error::{FlowError, Result};
use crate::node::RetryPolicy;

/// A single node declaration inside a flow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDef {
    /// Unique identifier within the flow (e.g. `"llm_1"`, `"http_call"`).
    pub id: String,
    /// The node type, used to resolve the executor (e.g. `"noop"`, `"llm"`, `"http"`).
    #[serde(rename = "type")]
    pub node_type: String,
    /// Static configuration for this node (prompt template, URL, etc.).
    /// Mirrors Dify's `data` field.
    #[serde(default)]
    pub data: Value,
    /// Optional guard condition extracted from `data["run_if"]` during `from_json`.
    /// Not present in the wire format — lives inside `data`.
    #[serde(skip)]
    pub(crate) run_if: Option<Condition>,
    /// Optional retry policy extracted from `data["retry"]` during `from_json`.
    #[serde(skip)]
    pub(crate) retry: Option<RetryPolicy>,
    /// Optional execution timeout in milliseconds extracted from `data["timeout_ms"]`.
    #[serde(skip)]
    pub(crate) timeout_ms: Option<u64>,
    /// If true, a node failure produces `{"__error__": "reason"}` as output
    /// instead of halting the flow. Parsed from `data["continue_on_error"]`.
    #[serde(skip)]
    pub(crate) continue_on_error: bool,
    /// If true, the runner merges the node's output object into the live
    /// variable map after the wave completes, making the values available to
    /// all downstream nodes in `ctx.variables`.
    ///
    /// Set automatically for `node_type == "assign"`.
    #[serde(skip)]
    pub(crate) write_to_variables: bool,
}

/// A directed edge from one node to another in a flow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDef {
    /// ID of the upstream (source) node.
    pub source: String,
    /// ID of the downstream (target) node.
    pub target: String,
}

/// Internal: the top-level shape of a flow definition JSON object.
#[derive(Deserialize)]
struct FlowDef {
    nodes: Vec<NodeDef>,
    #[serde(default)]
    edges: Vec<EdgeDef>,
}

/// A parsed and validated directed acyclic graph representing a workflow.
///
/// `Clone` is implemented so the `"iteration"` node can share one parsed
/// sub-flow DAG across concurrent loop iterations without re-parsing.
///
/// Constructed via [`DagGraph::from_json`]. After construction the graph is
/// guaranteed to be acyclic and self-consistent (all edge source/target IDs exist).
#[derive(Clone)]
pub struct DagGraph {
    /// Node definitions, keyed by node ID.
    pub(crate) nodes: HashMap<String, NodeDef>,
    /// Topologically sorted node IDs (sources first, sinks last).
    pub(crate) topo_order: Vec<String>,
    /// Adjacency: node ID → list of direct dependency node IDs.
    pub(crate) deps: HashMap<String, Vec<String>>,
}

impl DagGraph {
    /// Parse and validate a flow definition from a JSON value.
    ///
    /// The expected shape is a `{ "nodes": [...], "edges": [...] }` object
    /// (Dify-compatible format):
    ///
    /// ```json
    /// {
    ///   "nodes": [
    ///     { "id": "fetch", "type": "http-request", "data": { "url": "https://api.example.com" } },
    ///     { "id": "notify", "type": "noop", "data": { "run_if": { "from": "fetch", "path": "ok", "op": "eq", "value": true } } }
    ///   ],
    ///   "edges": [
    ///     { "source": "fetch", "target": "notify" }
    ///   ]
    /// }
    /// ```
    pub fn from_json(value: &Value) -> Result<Self> {
        let raw: FlowDef = serde_json::from_value(value.clone())
            .map_err(|e| FlowError::InvalidDefinition(e.to_string()))?;

        if raw.nodes.is_empty() {
            return Err(FlowError::InvalidDefinition(
                "flow must contain at least one node".into(),
            ));
        }

        // Build id → NodeDef map, extracting run_if from data["run_if"].
        let mut nodes: HashMap<String, NodeDef> = HashMap::new();
        for mut def in raw.nodes {
            if nodes.contains_key(&def.id) {
                return Err(FlowError::InvalidDefinition(format!(
                    "duplicate node id: {}",
                    def.id
                )));
            }
            if let Some(run_if_val) = def.data.get("run_if") {
                def.run_if = serde_json::from_value(run_if_val.clone()).map_err(|e| {
                    FlowError::InvalidDefinition(format!("node '{}': invalid run_if: {e}", def.id))
                })?;
            }
            if let Some(retry_val) = def.data.get("retry") {
                def.retry = serde_json::from_value(retry_val.clone()).map_err(|e| {
                    FlowError::InvalidDefinition(format!("node '{}': invalid retry: {e}", def.id))
                })?;
            }
            if let Some(ms) = def.data.get("timeout_ms").and_then(|v| v.as_u64()) {
                def.timeout_ms = Some(ms);
            }
            if let Some(true) = def.data.get("continue_on_error").and_then(|v| v.as_bool()) {
                def.continue_on_error = true;
            }
            if def.node_type == "assign" {
                def.write_to_variables = true;
            }
            nodes.insert(def.id.clone(), def);
        }

        // Build deps from edges (target → [sources]). Validate all IDs exist.
        let mut deps: HashMap<String, Vec<String>> =
            nodes.keys().map(|id| (id.clone(), vec![])).collect();
        for edge in &raw.edges {
            if !nodes.contains_key(&edge.source) {
                return Err(FlowError::UnknownNode(edge.source.clone()));
            }
            if !nodes.contains_key(&edge.target) {
                return Err(FlowError::UnknownNode(edge.target.clone()));
            }
            deps.entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
        }

        // Build petgraph DiGraph for cycle detection and topological sort.
        let mut graph: DiGraph<String, ()> = DiGraph::new();
        let mut id_to_idx: HashMap<String, NodeIndex> = HashMap::new();

        for id in nodes.keys() {
            let idx = graph.add_node(id.clone());
            id_to_idx.insert(id.clone(), idx);
        }

        for (target, sources) in &deps {
            let to = id_to_idx[target];
            for source in sources {
                let from = id_to_idx[source];
                graph.add_edge(from, to, ());
            }
        }

        let sorted = toposort(&graph, None).map_err(|_| FlowError::CyclicGraph)?;

        let topo_order: Vec<String> = sorted.into_iter().map(|idx| graph[idx].clone()).collect();

        Ok(Self {
            nodes,
            topo_order,
            deps,
        })
    }

    /// Returns node definitions in topological order (dependencies first).
    pub fn nodes_in_order(&self) -> impl Iterator<Item = &NodeDef> {
        self.topo_order.iter().map(move |id| &self.nodes[id])
    }

    /// Returns the direct dependencies of a node by ID.
    pub fn dependencies_of(&self, id: &str) -> &[String] {
        self.deps.get(id).map(Vec::as_slice).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_simple_linear_dag() {
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" },
                { "id": "c", "type": "noop" }
            ],
            "edges": [
                { "source": "a", "target": "b" },
                { "source": "b", "target": "c" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let order: Vec<&str> = dag.nodes_in_order().map(|n| n.id.as_str()).collect();
        // "a" must come before "b", "b" before "c"
        assert!(order.iter().position(|&x| x == "a") < order.iter().position(|&x| x == "b"));
        assert!(order.iter().position(|&x| x == "b") < order.iter().position(|&x| x == "c"));
    }

    #[test]
    fn rejects_cyclic_graph() {
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" }
            ],
            "edges": [
                { "source": "b", "target": "a" },
                { "source": "a", "target": "b" }
            ]
        });
        assert!(matches!(
            DagGraph::from_json(&def),
            Err(FlowError::CyclicGraph)
        ));
    }

    #[test]
    fn rejects_unknown_dependency() {
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" }
            ],
            "edges": [
                { "source": "nonexistent", "target": "a" }
            ]
        });
        assert!(matches!(
            DagGraph::from_json(&def),
            Err(FlowError::UnknownNode(_))
        ));
    }

    #[test]
    fn rejects_duplicate_node_ids() {
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "a", "type": "noop" }
            ],
            "edges": []
        });
        assert!(matches!(
            DagGraph::from_json(&def),
            Err(FlowError::InvalidDefinition(_))
        ));
    }

    #[test]
    fn rejects_empty_flow() {
        let def = json!({ "nodes": [], "edges": [] });
        assert!(matches!(
            DagGraph::from_json(&def),
            Err(FlowError::InvalidDefinition(_))
        ));
    }
}
