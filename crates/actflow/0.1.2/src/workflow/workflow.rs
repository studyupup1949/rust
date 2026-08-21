//! Runtime workflow representation using a directed graph.
//!
//! This module provides the internal workflow representation used during execution.
//! It wraps the workflow model in a directed graph structure (using petgraph) for
//! efficient traversal and state management.

use std::collections::HashMap;

use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{
    ActflowError, Result, ShareLock, WorkflowModel,
    common::Vars,
    workflow::{
        edge::{Edge, EdgeId, EdgeSelectOptions, SourceHandle},
        node::{Node, NodeId, NodeState},
    },
};

/// Runtime workflow representation as a directed graph.
///
/// The workflow graph maintains:
/// - Nodes representing actions to execute
/// - Edges representing transitions between nodes
/// - Execution state for each node and edge
///
/// The graph structure enables efficient:
/// - Dependency resolution (which nodes are ready to execute)
/// - Parallel execution (multiple branches can execute concurrently)
/// - Conditional branching (if_else nodes with true/false paths)
#[derive(Clone)]
pub struct Workflow {
    /// Thread-safe directed graph storing nodes and edges.
    graph: ShareLock<DiGraph<Node, Edge>>,
}

#[allow(unused)]
impl Workflow {
    /// create a new workflow
    pub fn new() -> Self {
        Self {
            graph: ShareLock::new(DiGraph::new().into()),
        }
    }

    /// Output a human-readable representation of the workflow graph
    pub fn schema(&self) -> String {
        let graph = self.graph.read().unwrap();
        let mut lines = Vec::new();

        lines.push("=== Workflow Graph ===".to_string());
        lines.push(format!("Nodes: {}, Edges: {}", graph.node_count(), graph.edge_count()));
        lines.push(String::new());

        // Print nodes
        lines.push("--- Nodes ---".to_string());
        for idx in graph.node_indices() {
            let node = &graph[idx];
            lines.push(format!(
                "[{}] {} (type: {}, status: {})",
                node.id,
                node.title,
                node.uses.as_ref(),
                node.status.as_ref()
            ));
        }
        lines.push(String::new());

        // Print edges
        lines.push("--- Edges ---".to_string());
        for idx in graph.edge_indices() {
            let edge = &graph[idx];
            let handle = match &edge.source_handle {
                SourceHandle::Fixed(h) => h.as_ref().to_string(),
                SourceHandle::Node(n) => n.clone(),
            };
            lines.push(format!(
                "{} --[{}]--> {} (id: {}, status: {})",
                edge.source,
                handle,
                edge.target,
                edge.id,
                edge.status.as_ref()
            ));
        }
        lines.push(String::new());

        // Print graph structure (adjacency list style)
        lines.push("--- Graph Structure ---".to_string());
        for idx in graph.node_indices() {
            let node = &graph[idx];
            let outgoing: Vec<String> = graph
                .edges_directed(idx, Direction::Outgoing)
                .map(|e| {
                    let target_idx = e.target();
                    let target_id = &graph[target_idx].id;
                    let handle = match &e.weight().source_handle {
                        SourceHandle::Fixed(h) => h.as_ref().to_string(),
                        SourceHandle::Node(n) => n.clone(),
                    };
                    format!("{}({})", target_id, handle)
                })
                .collect();

            if outgoing.is_empty() {
                lines.push(format!("{} -> (end)", node.id));
            } else {
                lines.push(format!("{} -> {}", node.id, outgoing.join(", ")));
            }
        }

        lines.join("\n")
    }

    /// add node to graph
    pub fn add_node(
        &self,
        node: Node,
    ) -> NodeIndex {
        let mut graph = self.graph.write().unwrap();
        graph.add_node(node)
    }

    /// add edge between two nodes
    pub fn add_edge(
        &self,
        from: NodeIndex,
        to: NodeIndex,
        edge: Edge,
    ) {
        let mut graph = self.graph.write().unwrap();
        graph.add_edge(from, to, edge);
    }

    /// get node by id
    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<Node> {
        let graph = self.graph.read().unwrap();
        graph.node_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].clone())
    }

    /// get edge by id
    pub fn get_edge(
        &self,
        id: &EdgeId,
    ) -> Option<Edge> {
        let graph = self.graph.read().unwrap();
        graph.edge_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].clone())
    }

    /// get node state by id
    pub fn get_node_state(
        &self,
        id: &NodeId,
    ) -> Option<NodeState> {
        self.get_node(id).map(|n| n.status)
    }

    /// get edge state by id
    pub fn get_edge_state(
        &self,
        id: &EdgeId,
    ) -> Option<NodeState> {
        self.get_edge(id).map(|e| e.status)
    }

    /// get max parallelism
    pub fn get_max_parallelism(&self) -> usize {
        let graph = self.graph.read().unwrap();
        graph.node_indices().map(|idx| graph.neighbors_directed(idx, petgraph::Direction::Outgoing).count()).max().unwrap_or(0)
    }

    /// get root node
    pub fn get_root_node(&self) -> Option<Node> {
        let graph = self.graph.read().unwrap();
        graph.node_indices().find(|idx| graph.neighbors_directed(*idx, petgraph::Direction::Incoming).count() == 0).map(|idx| graph[idx].clone())
    }

    /// get all node ids
    pub fn get_all_node_ids(&self) -> Vec<NodeId> {
        let graph = self.graph.read().unwrap();
        graph.node_indices().map(|idx| graph[idx].id.clone()).collect()
    }

    /// get next ready node
    pub fn get_next_ready_node(
        &self,
        nid: &NodeId,
        edge_select: EdgeSelectOptions,
    ) -> Vec<String> {
        let graph = self.graph.read().unwrap();
        graph
            .node_indices()
            .find(|idx| graph[*idx].id.eq(nid))
            .map(|src_idx| {
                graph
                    .edges_directed(src_idx, Direction::Outgoing)
                    .filter(|edge_ref| edge_ref.weight().source_handle == edge_select.source_handle)
                    .map(|edge_ref| edge_ref.target())
                    .filter(|dst_idx| {
                        let incoming_count = graph.neighbors_directed(*dst_idx, Direction::Incoming).count();
                        let completed_count = graph
                            .neighbors_directed(*dst_idx, Direction::Incoming)
                            .filter(|pred_idx| graph[*pred_idx].status == NodeState::Executed || graph[*pred_idx].status == NodeState::Skipped)
                            .count();

                        graph[*dst_idx].status == NodeState::Unknown && incoming_count == completed_count
                    })
                    .map(|dst_idx| graph[dst_idx].id.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// mark node as taken
    pub fn mark_node_taken(
        &self,
        id: &NodeId,
    ) {
        let mut graph = self.graph.write().unwrap();
        graph.node_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].status = NodeState::Taken);
    }

    /// mark node as skipped
    pub fn mark_node_skipped(
        &self,
        id: &NodeId,
    ) {
        let mut graph = self.graph.write().unwrap();
        graph.node_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].status = NodeState::Skipped);
    }

    /// mark node as executed
    pub fn mark_node_executed(
        &self,
        id: &NodeId,
    ) {
        let mut graph = self.graph.write().unwrap();
        graph.node_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].status = NodeState::Executed);
    }

    /// mark edge as taken
    pub fn mark_edge_taken(
        &self,
        id: &EdgeId,
    ) {
        let mut graph = self.graph.write().unwrap();
        graph.edge_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].status = NodeState::Taken);
    }

    /// mark edge as skipped
    pub fn mark_edge_skipped(
        &self,
        id: &EdgeId,
    ) {
        let mut graph = self.graph.write().unwrap();
        graph.edge_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].status = NodeState::Skipped);
    }

    /// mark edge as executed
    pub fn mark_edge_executed(
        &self,
        id: &EdgeId,
    ) {
        let mut graph = self.graph.write().unwrap();
        graph.edge_indices().find(|idx| graph[*idx].id.eq(id)).map(|idx| graph[idx].status = NodeState::Executed);
    }

    /// check if node is ready
    pub fn is_node_ready(
        &self,
        nid: &NodeId,
    ) -> Result<bool> {
        let graph = self.graph.read().unwrap();
        let node_idx = graph.node_indices().find(|idx| graph[*idx].id.eq(nid)).ok_or(ActflowError::Runtime(format!("node {} not found", nid)))?;

        if graph.neighbors_directed(node_idx, Direction::Incoming).filter(|e| graph[*e].status == NodeState::Unknown).count() > 0 {
            return Ok(false);
        }

        Ok(true)
    }

    /// check if node is end node
    pub fn is_end_node(
        &self,
        nid: &NodeId,
    ) -> Result<bool> {
        let graph = self.graph.read().unwrap();
        let node_idx = graph.node_indices().find(|idx| graph[*idx].id.eq(nid)).ok_or(ActflowError::Runtime(format!("node {} not found", nid)))?;

        if graph.neighbors_directed(node_idx, Direction::Outgoing).count() == 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// check if all nodes are executed or skipped
    pub fn is_all_node_executed(&self) -> bool {
        let graph = self.graph.read().unwrap();
        graph.node_indices().all(|idx| graph[idx].status == NodeState::Executed || graph[idx].status == NodeState::Skipped)
    }

    /// Get all outgoing edges from a node
    pub fn get_outgoing_edges(
        &self,
        nid: &NodeId,
    ) -> Vec<Edge> {
        let graph = self.graph.read().unwrap();
        graph
            .node_indices()
            .find(|idx| graph[*idx].id.eq(nid))
            .map(|src_idx| graph.edges_directed(src_idx, Direction::Outgoing).map(|edge_ref| edge_ref.weight().clone()).collect())
            .unwrap_or_default()
    }

    /// Skip a branch: mark the edge and all downstream nodes as skipped
    /// Returns a list of (node_id, edge_id) pairs that were skipped
    pub fn skip_branch(
        &self,
        edge_id: &EdgeId,
    ) -> Vec<(NodeId, EdgeId)> {
        let mut skipped = Vec::new();
        let mut to_process = vec![edge_id.clone()];

        while let Some(current_edge_id) = to_process.pop() {
            let mut graph = self.graph.write().unwrap();

            // Find and mark the edge as skipped
            let edge_idx = graph.edge_indices().find(|idx| graph[*idx].id.eq(&current_edge_id));
            let Some(edge_idx) = edge_idx else {
                continue;
            };

            // Skip if already processed
            if graph[edge_idx].status == NodeState::Skipped {
                continue;
            }

            graph[edge_idx].status = NodeState::Skipped;

            // Get the target node
            let (_, target_idx) = graph.edge_endpoints(edge_idx).unwrap();
            let target_node_id = graph[target_idx].id.clone();

            // Check if all incoming edges to this node are skipped
            let all_incoming_skipped = graph.edges_directed(target_idx, Direction::Incoming).all(|e| e.weight().status == NodeState::Skipped);

            if all_incoming_skipped && graph[target_idx].status == NodeState::Unknown {
                // Mark node as skipped
                graph[target_idx].status = NodeState::Skipped;
                skipped.push((target_node_id.clone(), current_edge_id.clone()));

                // Add all outgoing edges to process queue
                let outgoing_edges: Vec<EdgeId> = graph.edges_directed(target_idx, Direction::Outgoing).map(|e| e.weight().id.clone()).collect();

                drop(graph); // Release lock before adding to queue
                to_process.extend(outgoing_edges);
            }
        }

        skipped
    }

    /// Skip all unselected branches from a node
    /// Returns a list of (node_id, edge_id) pairs that were skipped
    pub fn skip_unselected_branches(
        &self,
        nid: &NodeId,
        selected_handle: &SourceHandle,
    ) -> Vec<(NodeId, EdgeId)> {
        let edges = self.get_outgoing_edges(nid);
        let mut all_skipped = Vec::new();

        for edge in edges {
            // Skip edges that match the selected handle
            if edge.source_handle == *selected_handle {
                continue;
            }

            // Skip this branch
            let skipped = self.skip_branch(&edge.id);
            all_skipped.extend(skipped);
        }

        all_skipped
    }
}

impl TryFrom<&WorkflowModel> for Workflow {
    type Error = ActflowError;

    fn try_from(model: &WorkflowModel) -> Result<Self> {
        let mut graph: DiGraph<Node, Edge> = DiGraph::new();

        let mut nodes = HashMap::new();

        for node in model.nodes.iter() {
            let node_value = serde_json::to_value(node).map_err(|e| ActflowError::from(e))?;
            let input = Vars::from(node_value);

            let node = Node::new(input)?;
            let nid = node.id.clone();
            let node_idx = graph.add_node(node);
            nodes.insert(nid, node_idx);
        }
        for edge in model.edges.iter() {
            let edge_value = serde_json::to_value(edge).map_err(|e| ActflowError::from(e))?;
            let input = Vars::from(edge_value);

            let edge = Edge::new(input)?;
            let source = nodes.get(&edge.source).ok_or(ActflowError::Edge(format!("source node {} not found", edge.source)))?;
            let target = nodes.get(&edge.target).ok_or(ActflowError::Edge(format!("target node {} not found", edge.target)))?;
            graph.add_edge(*source, *target, edge);
        }
        Ok(Self {
            graph: ShareLock::new(graph.into()),
        })
    }
}
