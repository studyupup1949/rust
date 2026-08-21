use graph_types::{Edge, EdgeDirection, EdgeInsertID, EdgeQuery, GraphEngine};

use std::collections::BTreeMap;

mod di_graph;
mod un_graph;

type EdgeID = u32;
type StartNodeID = u32;
type EndNodeID = u32;

/// Sparse, node first undirected adjacency list based graph
///
/// # Space Complexity
///
/// - O(|V| + |E|)
///
/// # Time Complexity
///
/// - Insert node: O(1)
/// - Insert edge: O(1)
/// - Query node: O(1)
/// - Query edge: O(V)
pub type UnGraph = AdjacencyList<true>;

/// Sparse, node first directed adjacency list based graph
///
/// # Space Complexity
///
/// - O(|V| + 2|E|)
///
/// # Time Complexity
///
/// - Insert node: O(1)
/// - Insert edge: O(1)
/// - Query node: O(1)
/// - Query edge: O(V)
pub type DiGraph = AdjacencyList<false>;

#[doc = include_str!("AdjacencyNodeList.html")]
#[derive(Debug)]
pub struct AdjacencyList<const TWO_WAY: bool> {
    head_nodes: BTreeMap<StartNodeID, NodeNeighbors>,
    last_edge: EdgeID,
}

#[derive(Debug)]
struct NodeNeighbors {
    end_nodes: BTreeMap<EdgeID, EndNodeID>,
}

impl<const TWO_WAY: bool> Default for AdjacencyList<TWO_WAY> {
    fn default() -> Self {
        Self { head_nodes: BTreeMap::new(), last_edge: 0 }
    }
}

impl<const TWO_WAY: bool> AdjacencyList<TWO_WAY> {
    /// Low level api, insert node
    pub(crate) fn insert_node(&mut self, node: u32) {
        self.head_nodes.entry(node).or_insert_with(|| NodeNeighbors { end_nodes: BTreeMap::new() });
    }
    /// Low level api, remove node
    pub(crate) fn remove_node(&mut self, node: u32) {
        self.head_nodes.remove(&node);
    }
}
