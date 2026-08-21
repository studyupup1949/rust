use super::*;
use graph_types::{GetEdgesVisitor, NodesVisitor};

impl GraphEngine for UnGraph {
    fn has_node(&self, node_id: usize) -> bool {
        self.head_nodes.contains_key(&(node_id as u32))
    }

    fn count_nodes(&self) -> usize {
        self.head_nodes.len()
    }

    fn insert_node(&mut self, node: usize) -> usize {
        self.insert_node(node as u32);
        node
    }

    fn remove_node_with_edges(&mut self, node_id: usize) {
        let node_id = node_id as u32;
        self.head_nodes.remove(&node_id);
    }

    fn traverse_nodes(&self) -> NodesVisitor<Self> {
        NodesVisitor::range(self, 0..self.count_nodes())
    }

    fn get_edges(&self) -> GetEdgesVisitor<Self> {
        todo!()
    }

    fn insert_edge_with_nodes<E: Edge>(&mut self, edge: E) -> EdgeInsertID {
        let min = edge.min_index() as u32;
        let max = edge.max_index() as u32;
        match edge.direction() {
            EdgeDirection::Disconnect => EdgeInsertID::Nothing,
            EdgeDirection::Forward | EdgeDirection::Reverse => {
                panic!("insert directed edge {edge} is not supported in undirected graph")
            }
            EdgeDirection::Dynamic | EdgeDirection::TwoWay => {
                let e = self.insert_undirected_edge(min, max);
                EdgeInsertID::OneEdge(e)
            }
        }
    }

    fn remove_edge<E>(&mut self, edge: E)
    where
        E: Into<EdgeQuery>,
    {
        match edge.into() {
            EdgeQuery::EdgeID(v) => {
                let edge_id = v as u32;
                for (_, node) in self.head_nodes.iter_mut() {
                    node.end_nodes.remove(&edge_id);
                    // edge id is unique in the graph
                    break;
                }
            }
            EdgeQuery::Directed(v) => {
                let start_node_id = v.lhs() as u32;
                let end_node_id = v.rhs() as u32;
                if let Some(node) = self.head_nodes.get_mut(&start_node_id) {
                    // notice that there are multiple edges between two nodes
                    node.end_nodes.retain(|_, &mut v| v != end_node_id);
                }
            }
            EdgeQuery::Undirected(v) => {
                panic!("remove undirected edge {v} is not supported in directed graph");
            }
        }
    }
    fn count_edges(&self) -> usize {
        self.head_nodes.iter().map(|(_, v)| v.end_nodes.len()).sum()
    }
}

impl UnGraph {
    fn new_edge_id(&mut self) -> u32 {
        let new = self.last_edge.saturating_add(1);
        self.last_edge = new;
        new
    }
    /// The low level interface for inserting a undirected edge
    ///
    /// Always from min to max, so just insert one
    pub fn insert_undirected_edge(&mut self, min: u32, max: u32) -> usize {
        debug_assert!(min <= max, "min index {} is larger than max index {}", min, max);
        let new_edge_id = self.new_edge_id();
        let min_node = self.head_nodes.entry(min).or_insert_with(|| NodeNeighbors { end_nodes: BTreeMap::new() });
        min_node.end_nodes.insert(new_edge_id, max);
        new_edge_id as usize
    }
}
