use crate::implementations::Edge;
use crate::implementations::Graph;
use crate::implementations::Node;

/// Build a graph from node keys and edge pairs
pub fn build_graph(node_keys: Vec<u32>, edge_pairs: Vec<(u32, u32)>) -> Graph<Node, Edge> {
    let nodes: Vec<Node> = node_keys.into_iter().map(Node::new).collect();
    let edges: Vec<Edge> = edge_pairs
        .into_iter()
        .map(|(source, target)| Edge::new(source, target))
        .collect();

    Graph::new(nodes, edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ade_traits::{GraphViewTrait, NodeTrait};

    #[test]
    fn test_build_empty_graph() {
        let graph = build_graph(vec![], vec![]);

        assert_eq!(graph.get_nodes().count(), 0);
        assert_eq!(graph.get_edges().count(), 0);
    }

    #[test]
    fn test_build_graph_single_node() {
        let graph = build_graph(vec![1], vec![]);

        assert_eq!(graph.get_nodes().count(), 1);
        assert_eq!(graph.get_edges().count(), 0);
        assert!(graph.has_node(1));

        let node = graph.get_node(1);
        assert_eq!(node.key(), 1);
        assert!(node.predecessors().is_empty());
        assert!(node.successors().is_empty());
    }

    #[test]
    fn test_build_graph_multiple_nodes_no_edges() {
        let graph = build_graph(vec![1, 2, 3], vec![]);

        assert_eq!(graph.get_nodes().count(), 3);
        assert_eq!(graph.get_edges().count(), 0);
        assert!(graph.has_node(1));
        assert!(graph.has_node(2));
        assert!(graph.has_node(3));
    }

    #[test]
    fn test_build_graph_with_edges() {
        let graph = build_graph(vec![1, 2, 3], vec![(1, 2), (2, 3)]);

        assert_eq!(graph.get_nodes().count(), 3);
        assert_eq!(graph.get_edges().count(), 2);

        // Check nodes exist
        assert!(graph.has_node(1));
        assert!(graph.has_node(2));
        assert!(graph.has_node(3));

        // Check edges exist
        assert!(graph.has_edge(1, 2));
        assert!(graph.has_edge(2, 3));
        assert!(!graph.has_edge(1, 3));
    }

    #[test]
    fn test_build_graph_node_connections() {
        let graph = build_graph(vec![1, 2, 3], vec![(1, 2), (2, 3), (3, 1)]);

        // Check node 1: successor=2, predecessor=3
        let node1 = graph.get_node(1);
        assert!(node1.successors().contains(&2));
        assert!(node1.predecessors().contains(&3));
        assert_eq!(node1.successors().len(), 1);
        assert_eq!(node1.predecessors().len(), 1);

        // Check node 2: successor=3, predecessor=1
        let node2 = graph.get_node(2);
        assert!(node2.successors().contains(&3));
        assert!(node2.predecessors().contains(&1));

        // Check node 3: successor=1, predecessor=2
        let node3 = graph.get_node(3);
        assert!(node3.successors().contains(&1));
        assert!(node3.predecessors().contains(&2));
    }

    #[test]
    fn test_build_graph_multiple_edges_same_node() {
        let graph = build_graph(vec![1, 2, 3], vec![(1, 2), (1, 3)]);

        let node1 = graph.get_node(1);
        assert_eq!(node1.successors().len(), 2);
        assert!(node1.successors().contains(&2));
        assert!(node1.successors().contains(&3));
        assert_eq!(node1.predecessors().len(), 0);

        let node2 = graph.get_node(2);
        assert_eq!(node2.predecessors().len(), 1);
        assert!(node2.predecessors().contains(&1));

        let node3 = graph.get_node(3);
        assert_eq!(node3.predecessors().len(), 1);
        assert!(node3.predecessors().contains(&1));
    }

    #[test]
    fn test_build_graph_with_self_loop() {
        let graph = build_graph(vec![1, 2], vec![(1, 1), (1, 2)]);

        assert_eq!(graph.get_nodes().count(), 2);
        assert_eq!(graph.get_edges().count(), 2);

        // Check self-loop
        assert!(graph.has_edge(1, 1));
        assert!(graph.has_edge(1, 2));

        let node1 = graph.get_node(1);
        assert!(node1.successors().contains(&1)); // self-loop
        assert!(node1.successors().contains(&2));
        assert!(node1.predecessors().contains(&1)); // self-loop
    }

    #[test]
    fn test_build_graph_large_keys() {
        let large_key = u32::MAX - 1;
        let graph = build_graph(vec![0, large_key], vec![(0, large_key)]);

        assert_eq!(graph.get_nodes().count(), 2);
        assert_eq!(graph.get_edges().count(), 1);
        assert!(graph.has_node(0));
        assert!(graph.has_node(large_key));
        assert!(graph.has_edge(0, large_key));
    }

    #[test]
    fn test_build_complete_triangle() {
        let graph = build_graph(
            vec![1, 2, 3],
            vec![(1, 2), (2, 3), (3, 1), (1, 3), (3, 2), (2, 1)],
        );

        assert_eq!(graph.get_nodes().count(), 3);
        assert_eq!(graph.get_edges().count(), 6);

        // Every node should have 2 predecessors and 2 successors
        for key in [1, 2, 3] {
            let node = graph.get_node(key);
            assert_eq!(node.predecessors().len(), 2);
            assert_eq!(node.successors().len(), 2);
        }
    }
}
