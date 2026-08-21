use crate::implementations::FilteredGraph;
use ade_traits::{EdgeTrait, GraphViewTrait, NodeTrait};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Graph<N, E> {
    nodes: HashMap<u32, N>,
    edges: HashMap<(u32, u32), E>,
}

impl<N: NodeTrait, E: EdgeTrait> Graph<N, E> {
    pub fn new(nodes: Vec<N>, edges: Vec<E>) -> Self {
        let mut graph = Graph {
            nodes: HashMap::with_capacity(nodes.len()),
            edges: HashMap::with_capacity(edges.len()),
        };

        for node in nodes {
            graph.add_node(node);
        }

        for edge in edges {
            graph.add_edge(edge);
        }

        graph
    }

    pub fn add_node(&mut self, node: N) -> bool {
        self.nodes.insert(node.key(), node).is_none()
    }

    pub fn remove_node(&mut self, key: u32) {
        // Collect edge keys to remove before mutating
        let edges_to_remove: Vec<(u32, u32)> = if let Some(node) = self.nodes.get(&key) {
            let mut edges = Vec::new();

            // Incoming edges (predecessor -> this node)
            for predecessor in node.predecessors() {
                edges.push((*predecessor, key));
            }

            // Outgoing edges (this node -> successor)
            for successor in node.successors() {
                edges.push((key, *successor));
            }

            edges
        } else {
            Vec::new()
        };

        // Remove all connected edges
        for (source, target) in edges_to_remove {
            self.remove_edge(source, target);
        }

        // Remove the node itself
        self.nodes.remove(&key);
    }

    pub fn add_edge(&mut self, edge: E) -> bool {
        if !self.nodes.contains_key(&edge.source()) || !self.nodes.contains_key(&edge.target()) {
            return false;
        }

        // Update successors and predecessors
        if let Some(source_node) = self.nodes.get_mut(&edge.source()) {
            source_node.add_successor(edge.target());
        }
        if let Some(target_node) = self.nodes.get_mut(&edge.target()) {
            target_node.add_predecessor(edge.source());
        }

        self.edges
            .insert((edge.source(), edge.target()), edge)
            .is_none()
    }

    pub fn remove_edge(&mut self, source: u32, target: u32) {
        let edge_key = (source, target);

        if self.edges.remove(&edge_key).is_some() {
            // Update node connections
            if let Some(source_node) = self.nodes.get_mut(&source) {
                source_node.remove_successor(target);
            }
            if let Some(target_node) = self.nodes.get_mut(&target) {
                target_node.remove_predecessor(source);
            }
        }
    }

    // pub fn get_subgraph_graph(&self, node_keys: &[u32]) -> Graph<N, E> {
    //     let mut subgraph = Graph::<N, E>::new(Vec::new(), Vec::new());
    //     let node_key_set: HashSet<u32> = node_keys.iter().copied().collect();

    //     // Add nodes that are in the key set
    //     for node in self.get_nodes() {
    //         if node_key_set.contains(&node.key()) {
    //             subgraph.add_node(node.fresh_copy());
    //         }
    //     }

    //     // Add edges where both source and target are in the key set
    //     for edge in self.get_edges() {
    //         let source = edge.source();
    //         let target = edge.target();
    //         if node_key_set.contains(&source) && node_key_set.contains(&target) {
    //             subgraph.add_edge(edge.clone());
    //         }
    //     }

    //     subgraph
    // }
}

impl<N: NodeTrait, E: EdgeTrait> GraphViewTrait<N, E> for Graph<N, E> {
    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn has_sequential_keys(&self) -> bool {
        let size = self.nodes.len();
        if size == 0 {
            return true;
        }

        // Quick checks first
        if !self.nodes.contains_key(&0) || !self.nodes.contains_key(&(size as u32 - 1)) {
            return false;
        }

        // Check if all keys are sequential
        (0..size as u32).all(|i| self.nodes.contains_key(&i))
    }

    fn get_node(&self, key: u32) -> &N {
        self.nodes
            .get(&key)
            .unwrap_or_else(|| panic!("Node {} not found", key))
    }

    fn has_node(&self, key: u32) -> bool {
        self.nodes.contains_key(&key)
    }

    fn get_edge(&self, source: u32, target: u32) -> &E {
        let edge_key = (source, target);
        self.edges
            .get(&edge_key)
            .unwrap_or_else(|| panic!("Edge {}→{} not found", source, target))
    }

    fn has_edge(&self, source: u32, target: u32) -> bool {
        let edge_key = (source, target);
        self.edges.contains_key(&edge_key)
    }

    fn get_nodes<'a>(&'a self) -> impl Iterator<Item = &'a N>
    where
        N: 'a,
    {
        self.nodes.values()
    }

    fn get_node_keys(&self) -> impl Iterator<Item = u32> {
        self.nodes.keys().copied()
    }

    fn get_edges<'a>(&'a self) -> impl Iterator<Item = &'a E>
    where
        E: 'a,
    {
        self.edges.values()
    }

    fn get_predecessors<'a>(&'a self, node_key: u32) -> impl Iterator<Item = &'a N>
    where
        N: 'a,
    {
        self.get_node(node_key)
            .predecessors()
            .iter()
            .map(|pred_key| self.get_node(*pred_key))
    }

    fn get_predecessors_keys(&self, node_key: u32) -> impl Iterator<Item = u32> {
        self.get_node(node_key).predecessors().iter().copied()
    }

    fn get_successors<'a>(&'a self, node_key: u32) -> impl Iterator<Item = &'a N>
    where
        N: 'a,
    {
        self.get_node(node_key)
            .successors()
            .iter()
            .map(|succ_key| self.get_node(*succ_key))
    }

    fn get_successors_keys(&self, node_key: u32) -> impl Iterator<Item = u32> {
        self.get_node(node_key).successors().iter().copied()
    }

    fn filter(&self, node_keys: &[u32]) -> impl GraphViewTrait<N, E> {
        FilteredGraph::new(self, node_keys.iter().copied())
    }
}

use std::fmt;

impl<N: NodeTrait, E: EdgeTrait> fmt::Display for Graph<N, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Nodes:")?;
        for node in self.get_nodes() {
            writeln!(
                f,
                "- {} (pred: {:?}, succ: {:?})",
                node.key(),
                node.predecessors(),
                node.successors()
            )?;
        }

        writeln!(f, "\nEdges:")?;
        for edge in self.get_edges() {
            writeln!(f, "- {} -> {}", edge.source(), edge.target())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::edge::Edge;
    use crate::implementations::node::Node;

    #[test]
    fn test_is_empty() {
        let graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());
        assert!(graph.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        assert!(graph.add_node(Node::new(1))); // Adding a new node should return true
        assert!(!graph.add_node(Node::new(1))); // Adding the same node again should return false
    }

    #[test]
    fn test_get_node() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));

        let node = graph.get_node(1);
        assert_eq!(node.key(), 1);
        assert_eq!(node.predecessors().len(), 0);
        assert_eq!(node.successors().len(), 0);
    }

    #[test]
    #[should_panic(expected = "Node 2 not found")]
    fn test_get_node_panic() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());
        graph.add_node(Node::new(1));
        graph.get_node(2);
    }

    #[test]
    fn test_get_nodes() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));

        assert_eq!(graph.get_nodes().count(), 2);

        // assert it contains both nodes
        assert!(graph.get_nodes().any(|n| n.key() == 1));
        assert!(graph.get_nodes().any(|n| n.key() == 2));
    }

    #[test]
    fn test_get_node_keys() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));

        assert_eq!(graph.get_node_keys().count(), 2);

        let node_keys = graph.get_node_keys().collect::<Vec<_>>();
        // assert it contains both keys
        assert!(node_keys.contains(&1));
        assert!(node_keys.contains(&2));
    }

    #[test]
    fn test_add_predecessor() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_edge(Edge::new(1, 2));

        assert!(graph.get_node(2).predecessors().contains(&1));
    }

    #[test]
    fn test_add_successor() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_edge(Edge::new(1, 2));

        assert!(graph.get_node(1).successors().contains(&2));
    }

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));

        assert!(graph.add_edge(Edge::new(1, 2))); // Adding a new edge should return true
        assert!(!graph.add_edge(Edge::new(1, 2))); // Adding the same edge again should return false

        assert!(graph.has_edge(1, 2));

        // Check predecessors and successors
        assert!(graph.get_node(1).successors().contains(&2));
        assert!(graph.get_node(2).predecessors().contains(&1));
    }

    #[test]
    fn test_get_edge() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));

        graph.add_edge(Edge::new(1, 2));

        let edge = graph.get_edge(1, 2);
        assert_eq!(edge.source(), 1);
        assert_eq!(edge.target(), 2);
    }

    #[test]
    #[should_panic(expected = "Edge 2→1 not found")]
    fn test_get_edge_panic() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());
        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_edge(Edge::new(1, 2));
        graph.get_edge(2, 1);
    }

    #[test]
    fn test_predecessors() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));

        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(3, 2));

        let predecessors = graph.get_node(2).predecessors();
        assert_eq!(predecessors.len(), 2);
        assert!(predecessors.contains(&1));
        assert!(predecessors.contains(&3));
    }

    #[test]
    fn test_successors() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));

        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(1, 3));

        let successors = graph.get_node(1).successors();
        assert_eq!(successors.len(), 2);
        assert!(successors.contains(&2));
        assert!(successors.contains(&3));
    }

    #[test]
    fn test_remove_edge() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));

        graph.add_edge(Edge::new(1, 2));
        assert!(graph.has_edge(1, 2));

        graph.remove_edge(1, 2);
        assert!(!graph.has_edge(1, 2));

        // Check that predecessors and successors are also removed
        assert!(!graph.get_node(1).successors().contains(&2));
        assert!(!graph.get_node(2).predecessors().contains(&1));
    }

    #[test]
    fn test_remove_node() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));

        graph.add_edge(Edge::new(1, 2));
        assert!(graph.has_edge(1, 2));

        graph.add_edge(Edge::new(2, 3));
        assert!(graph.has_edge(2, 3));

        graph.add_edge(Edge::new(3, 1));
        assert!(graph.has_edge(3, 1));

        graph.remove_node(1);
        assert!(!graph.has_node(1));
        assert!(!graph.has_edge(1, 2));
        assert!(!graph.has_edge(3, 1));
        assert!(graph.has_edge(2, 3));

        let node_b = graph.get_node(2);
        assert!(!node_b.predecessors().contains(&1));
        assert!(node_b.successors().contains(&3));

        let node_c = graph.get_node(3);
        assert!(node_c.predecessors().contains(&2));
        assert!(!node_c.successors().contains(&1));
    }

    #[test]
    fn test_get_predecessors() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));

        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(3, 2));

        assert_eq!(graph.get_predecessors(2).count(), 2);
        assert!(graph.get_predecessors(2).any(|n| n.key() == 1));
        assert!(graph.get_predecessors(2).any(|n| n.key() == 3));
    }

    #[test]
    fn test_get_successors() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));

        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(1, 3));

        assert_eq!(graph.get_successors(1).count(), 2);
        assert!(graph.get_successors(1).any(|n| n.key() == 2));
        assert!(graph.get_successors(1).any(|n| n.key() == 3));
    }

    #[test]
    fn test_get_successors_keys() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());
        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));
        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(1, 3));

        let successors_keys: Vec<u32> = graph.get_successors_keys(1).collect();
        assert_eq!(successors_keys.len(), 2);
        assert!(successors_keys.contains(&2));
        assert!(successors_keys.contains(&3));
    }

    #[test]
    fn test_filter() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));

        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(1, 3));

        // 2 nodes and 1 edge
        let subgraph1 = graph.filter(&[1, 2]);
        assert_eq!(subgraph1.get_nodes().count(), 2);
        assert!(subgraph1.has_node(1));
        assert!(subgraph1.has_node(2));
        assert!(!subgraph1.has_node(3));
        assert!(subgraph1.has_edge(1, 2));
        assert!(!subgraph1.has_edge(1, 3));

        // 1 node and 0 edges
        let subgraph2 = graph.filter(&[1]);
        assert_eq!(subgraph2.get_nodes().count(), 1);
        assert!(subgraph2.has_node(1));
        assert!(!subgraph2.has_node(2));
        assert!(!subgraph2.has_node(3));
        assert!(!subgraph2.has_edge(1, 2));
        assert!(!subgraph2.has_edge(1, 3));

        // 0 nodes and 0 edges (node 4 doesn't exist)
        let subgraph3 = graph.filter(&[4]);
        assert_eq!(subgraph3.get_nodes().count(), 0);
        assert!(!subgraph3.has_node(1));
        assert!(!subgraph3.has_node(2));
        assert!(!subgraph3.has_node(3));
        assert!(!subgraph3.has_edge(1, 2));
        assert!(!subgraph3.has_edge(1, 3));
    }

    #[test]
    fn test_has_node() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        assert!(graph.has_node(1));
        assert!(!graph.has_node(2));
    }

    #[test]
    fn test_has_edge() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));

        assert!(!graph.has_edge(1, 2));
        graph.add_edge(Edge::new(1, 2));
        assert!(graph.has_edge(1, 2));
        assert!(!graph.has_edge(2, 1)); // Directed graph
    }

    #[test]
    fn test_get_edges() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));
        graph.add_node(Node::new(3));

        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(2, 3));

        assert_eq!(graph.get_edges().count(), 2);
        assert!(graph
            .get_edges()
            .any(|e| e.source() == 1 && e.target() == 2));
        assert!(graph
            .get_edges()
            .any(|e| e.source() == 2 && e.target() == 3));
    }

    #[test]
    fn test_has_sequential_keys() {
        let mut graph = Graph::<Node, Edge>::new(Vec::new(), Vec::new());

        graph.add_node(Node::new(0));
        graph.add_node(Node::new(1));
        graph.add_node(Node::new(2));

        assert!(graph.has_sequential_keys());
    }
}
