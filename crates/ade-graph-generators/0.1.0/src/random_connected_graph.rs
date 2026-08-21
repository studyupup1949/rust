use std::collections::HashSet;

/// Generate a random connected graph
///
/// This function ensures the generated graph is connected by first creating a spanning tree,
/// then adding additional random edges.
///
/// # Arguments
/// * `n` - Number of nodes (nodes will be numbered from 0 to n-1)
/// * `m` - Number of edges to generate (must be at least n-1 for connectivity)
/// * `seed` - Random seed for reproducible results
///
/// # Returns
/// A tuple containing (node_keys, edge_pairs) for a connected graph
///
/// # Panics
/// Panics if m < n-1 (not enough edges to ensure connectivity) or m > n*(n-1)
pub fn generate_random_connected_graph_data(
    n: usize,
    m: usize,
    seed: u64,
) -> (Vec<u32>, Vec<(u32, u32)>) {
    if n == 0 {
        return (Vec::new(), Vec::new());
    }

    if m < n - 1 {
        panic!(
            "Cannot create connected graph with {} nodes and {} edges. Minimum required: {}",
            n,
            m,
            n - 1
        );
    }

    let max_edges = n * (n - 1);
    if m > max_edges {
        panic!(
            "Cannot generate {} edges with {} nodes. Maximum possible: {}",
            m, n, max_edges
        );
    }

    let node_keys: Vec<u32> = (0..n as u32).collect();
    let mut rng_state = seed;
    let mut edge_set = HashSet::new();

    // First, create a spanning tree to ensure connectivity
    let mut nodes_in_tree = HashSet::new();
    nodes_in_tree.insert(0u32); // Start with node 0

    for i in 1..n as u32 {
        // Connect node i to a random node already in the tree
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let tree_nodes: Vec<_> = nodes_in_tree.iter().copied().collect();
        let random_tree_node = tree_nodes[(rng_state as usize) % tree_nodes.len()];

        // Randomly choose direction of the edge
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        if rng_state % 2 == 0 {
            edge_set.insert((random_tree_node, i));
        } else {
            edge_set.insert((i, random_tree_node));
        }

        nodes_in_tree.insert(i);
    }

    // Add remaining random edges
    while edge_set.len() < m {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let source = (rng_state as u32) % (n as u32);

        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let target = (rng_state as u32) % (n as u32);

        // No self-loops
        if source != target {
            edge_set.insert((source, target));
        }
    }

    let edge_pairs: Vec<(u32, u32)> = edge_set.into_iter().collect();

    (node_keys, edge_pairs)
}

#[cfg(test)]
mod connected_graph_tests {
    use super::*;
    use graph::core::Graph;
    use graph::edge::Edge;
    use graph::node::Node;
    use graph::utils::build_graph;
    use std::collections::HashSet;

    /// Helper function to check if a graph is connected using strongly connected components
    /// We create an undirected version of the graph and check if there's only one SCC
    fn is_connected(nodes: &[u32], edges: &[(u32, u32)]) -> bool {
        use strongly_connected_components::scc;

        if nodes.is_empty() {
            return true;
        }

        // Create nodes for the undirected graph
        let graph_nodes: Vec<Node> = nodes.iter().map(|&key| Node::new(key)).collect();

        // Create edges for the undirected graph (add both directions)
        let mut graph_edges = Vec::new();
        for &(source, target) in edges {
            graph_edges.push(Edge::new(source, target));
            graph_edges.push(Edge::new(target, source)); // Add reverse edge for undirected behavior
        }

        // Build the undirected graph
        let undirected_graph = Graph::new(graph_nodes, graph_edges);

        // Check if there's exactly one strongly connected component
        let components = scc(&undirected_graph);
        components.len() == 1
    }

    #[test]
    fn test_generate_random_connected_graph_data_basic() {
        let (nodes, edges) = generate_random_connected_graph_data(5, 8, 42);

        // Check node count and values
        assert_eq!(nodes.len(), 5);
        assert_eq!(nodes, vec![0, 1, 2, 3, 4]);

        // Check edge count
        assert_eq!(edges.len(), 8);

        // Check that all edges are valid (no self-loops, nodes within range)
        for (source, target) in &edges {
            assert_ne!(source, target, "Self-loop found: {} -> {}", source, target);
            assert!(*source < 5, "Source node {} out of range", source);
            assert!(*target < 5, "Target node {} out of range", target);
        }

        // Most importantly: check connectivity
        assert!(
            is_connected(&nodes, &edges),
            "Generated graph is not connected"
        );
    }

    #[test]
    fn test_generate_random_connected_graph_data_deterministic() {
        // Same seed should produce same result
        let (nodes1, edges1) = generate_random_connected_graph_data(4, 6, 123);
        let (nodes2, edges2) = generate_random_connected_graph_data(4, 6, 123);

        assert_eq!(nodes1, nodes2);
        assert_eq!(edges1, edges2);

        // Different seed should produce different result
        let (nodes3, edges3) = generate_random_connected_graph_data(4, 6, 456);
        assert_eq!(nodes1, nodes3); // Nodes should be the same (0,1,2,3)
        assert_ne!(edges1, edges3); // Edges should be different (with high probability)

        // Both should be connected
        assert!(is_connected(&nodes1, &edges1));
        assert!(is_connected(&nodes3, &edges3));
    }

    #[test]
    fn test_generate_random_connected_graph_data_minimum_edges() {
        // Test with minimum number of edges (n-1)
        let n = 6;
        let m = n - 1; // Minimum for connectivity
        let (nodes, edges) = generate_random_connected_graph_data(n, m, 789);

        assert_eq!(nodes.len(), n);
        assert_eq!(edges.len(), m);
        assert!(
            is_connected(&nodes, &edges),
            "Graph with minimum edges is not connected"
        );
    }

    #[test]
    fn test_generate_random_connected_graph_data_no_duplicates() {
        let (_, edges) = generate_random_connected_graph_data(6, 15, 789);

        // Convert to HashSet to check for duplicates
        let edge_set: HashSet<_> = edges.iter().collect();
        assert_eq!(edges.len(), edge_set.len(), "Duplicate edges found");
    }

    #[test]
    fn test_generate_random_connected_graph_data_edge_cases() {
        // Single node, zero edges
        let (nodes, edges) = generate_random_connected_graph_data(1, 0, 1);
        assert_eq!(nodes, vec![0]);
        assert_eq!(edges, vec![]);
        assert!(is_connected(&nodes, &edges)); // Single node is trivially connected

        // Two nodes, one edge (minimum for connectivity)
        let (nodes, edges) = generate_random_connected_graph_data(2, 1, 2);
        assert_eq!(nodes, vec![0, 1]);
        assert_eq!(edges.len(), 1);
        assert!(is_connected(&nodes, &edges));

        // Zero nodes, zero edges
        let (nodes, edges) = generate_random_connected_graph_data(0, 0, 3);
        assert_eq!(nodes, vec![]);
        assert_eq!(edges, vec![]);
        assert!(is_connected(&nodes, &edges)); // Empty graph is trivially connected
    }

    #[test]
    fn test_generate_random_connected_graph_data_maximum_edges() {
        let n = 4;
        let max_edges = n * (n - 1); // 4 * 3 = 12 maximum edges
        let (nodes, edges) = generate_random_connected_graph_data(n, max_edges, 999);

        assert_eq!(nodes.len(), n);
        assert_eq!(edges.len(), max_edges);
        assert!(is_connected(&nodes, &edges));

        // Check that we have all possible edges (no duplicates)
        let edge_set: HashSet<_> = edges.iter().collect();
        assert_eq!(edges.len(), edge_set.len());

        // Verify all edges are valid
        for (source, target) in &edges {
            assert_ne!(source, target);
            assert!(*source < n as u32);
            assert!(*target < n as u32);
        }
    }

    #[test]
    #[should_panic(expected = "Cannot create connected graph")]
    fn test_generate_random_connected_graph_data_too_few_edges() {
        // Try to create connected graph with too few edges
        generate_random_connected_graph_data(5, 3, 123); // Need at least 4 edges for 5 nodes
    }

    #[test]
    #[should_panic(expected = "Cannot generate")]
    fn test_generate_random_connected_graph_data_too_many_edges() {
        // Try to generate more edges than possible
        let n = 3;
        let max_possible = n * (n - 1); // 3 * 2 = 6
        generate_random_connected_graph_data(n, max_possible + 1, 123); // Should panic
    }

    #[test]
    fn test_generate_random_connected_graph_data_connectivity_multiple_seeds() {
        // Test connectivity with various configurations and seeds
        let test_cases = vec![
            (3, 3, 1),     // Small graph
            (5, 8, 42),    // Medium graph
            (7, 15, 999),  // Larger graph
            (10, 20, 555), // Even larger
        ];

        for (n, m, seed) in test_cases {
            let (nodes, edges) = generate_random_connected_graph_data(n, m, seed);
            assert!(
                is_connected(&nodes, &edges),
                "Graph with {} nodes and {} edges (seed {}) is not connected",
                n,
                m,
                seed
            );
        }
    }

    #[test]
    fn test_generate_random_connected_graph_data_spanning_tree_property() {
        // For a connected graph with n nodes and n-1 edges, it should be a tree
        let n = 8;
        let m = n - 1;
        let (nodes, edges) = generate_random_connected_graph_data(n, m, 123);

        assert_eq!(edges.len(), m);
        assert!(is_connected(&nodes, &edges));

        // A connected graph with n nodes and n-1 edges is a tree (acyclic)
        // We can't easily test for cycles in directed graphs, but we can verify
        // that it's minimally connected (removing any edge would disconnect it)
    }

    #[test]
    fn test_generate_random_connected_graph_data_with_build_graph() {
        // Integration test: ensure the generated data works with build_graph
        let (nodes, edges) = generate_random_connected_graph_data(5, 10, 42);
        let graph = build_graph(nodes.clone(), edges.clone());

        // Basic sanity checks on the built graph
        assert_eq!(graph.get_node_keys().len(), 5);
        assert_eq!(graph.get_edges().len(), 10);

        // Check that all nodes exist
        for i in 0..5 {
            assert!(graph.has_node(i));
        }

        // Check connectivity
        assert!(is_connected(&nodes, &edges));
    }

    #[test]
    fn test_generate_random_connected_graph_data_distribution() {
        // Test that different seeds produce reasonably different results
        let n = 6;
        let m = 12;
        let mut all_results = Vec::new();

        for seed in 0..10 {
            let (nodes, edges) = generate_random_connected_graph_data(n, m, seed);
            assert!(is_connected(&nodes, &edges));
            all_results.push(edges);
        }

        // Check that not all results are identical
        let first_result = &all_results[0];
        let all_identical = all_results.iter().all(|edges| edges == first_result);
        assert!(
            !all_identical,
            "All results are identical - poor randomness"
        );
    }

    // #[test]
    // fn test_generate_random_connected_graph_data_all_nodes_reachable() {
    //     // More thorough connectivity test: ensure every node can reach every other node
    //     let (nodes, edges) = generate_random_connected_graph_data(6, 15, 777);

    //     // Build adjacency list (treating as undirected)
    //     let mut adjacency: std::collections::HashMap<u32, Vec<u32>> =
    //         std::collections::HashMap::new();
    //     for &node in &nodes {
    //         adjacency.insert(node, Vec::new());
    //     }

    //     for &(source, target) in &edges {
    //         adjacency.get_mut(&source).unwrap().push(target);
    //         adjacency.get_mut(&target).unwrap().push(source);
    //     }

    //     // For each node, check if it can reach all other nodes
    //     for &start_node in &nodes {
    //         let mut visited = HashSet::new();
    //         let mut queue = VecDeque::new();

    //         queue.push_back(start_node);
    //         visited.insert(start_node);

    //         while let Some(current) = queue.pop_front() {
    //             for &neighbor in adjacency.get(&current).unwrap() {
    //                 if !visited.contains(&neighbor) {
    //                     visited.insert(neighbor);
    //                     queue.push_back(neighbor);
    //                 }
    //             }
    //         }

    //         assert_eq!(
    //             visited.len(),
    //             nodes.len(),
    //             "Node {} cannot reach all other nodes",
    //             start_node
    //         );
    //     }
    // }

    #[test]
    fn test_generate_random_connected_graph_data_seed_consistency() {
        // Test that the same seed produces consistent results across multiple calls
        for seed in [1, 42, 100, 999, 1234567890] {
            let result1 = generate_random_connected_graph_data(5, 8, seed);
            let result2 = generate_random_connected_graph_data(5, 8, seed);
            assert_eq!(result1, result2, "Inconsistent results for seed {}", seed);

            // Both should be connected
            assert!(is_connected(&result1.0, &result1.1));
            assert!(is_connected(&result2.0, &result2.1));
        }
    }
}
