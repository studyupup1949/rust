fn lcg_next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state
}

fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

pub fn generate_random_graph_data(n: usize, m: usize, seed: u64) -> (Vec<u32>, Vec<(u32, u32)>) {
    if n == 0 {
        return (Vec::new(), Vec::new());
    }

    let node_keys: Vec<u32> = (0..n as u32).collect();
    let mut rng_state = seed;
    let mut edges = Vec::with_capacity(m);

    for _ in 0..m {
        let from = (mix64(lcg_next(&mut rng_state)) % (n as u64)) as u32;
        let mut to = (mix64(lcg_next(&mut rng_state)) % (n as u64)) as u32;
        if to == from {
            to = ((to as usize + 1) % n) as u32;
        }
        edges.push((from, to));
    }

    (node_keys, edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_graph_data_basic() {
        let (nodes, edges) = generate_random_graph_data(5, 8, 42);

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
    }

    #[test]
    fn test_generate_random_graph_data_deterministic() {
        // Same seed should produce same result
        let (nodes1, edges1) = generate_random_graph_data(4, 6, 123);
        let (nodes2, edges2) = generate_random_graph_data(4, 6, 123);

        assert_eq!(nodes1, nodes2);
        assert_eq!(edges1, edges2);

        // Different seed should produce different result
        let (nodes3, edges3) = generate_random_graph_data(4, 6, 456);
        assert_eq!(nodes1, nodes3); // Nodes should be the same
        assert_ne!(edges1, edges3); // Edges should be different
    }

    #[test]
    fn test_generate_random_graph_data_edge_cases() {
        // Single node, zero edges
        let (nodes, edges) = generate_random_graph_data(1, 0, 1);
        assert_eq!(nodes, vec![0]);
        assert_eq!(edges, vec![]);

        // Two nodes, one edge
        let (nodes, edges) = generate_random_graph_data(2, 1, 2);
        assert_eq!(nodes, vec![0, 1]);
        assert_eq!(edges.len(), 1);
        let edge = edges[0];
        assert!(edge == (0, 1) || edge == (1, 0));

        // Zero nodes, zero edges
        let (nodes, edges) = generate_random_graph_data(0, 0, 3);
        assert_eq!(nodes, vec![]);
        assert_eq!(edges, vec![]);
    }

    #[test]
    fn test_generate_random_graph_data_distribution() {
        // Test that different seeds produce reasonably different results
        let n = 5;
        let m = 10;
        let mut all_results = Vec::new();

        for seed in 0..10 {
            let (_, edges) = generate_random_graph_data(n, m, seed);
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

    #[test]
    fn test_generate_random_graph_data_node_coverage() {
        let (_nodes, edges) = generate_random_graph_data(10, 30, 555);

        let mut nodes_in_edges = std::collections::HashSet::new();
        for (source, target) in &edges {
            nodes_in_edges.insert(*source);
            nodes_in_edges.insert(*target);
        }

        assert!(
            nodes_in_edges.len() >= 7,
            "Only {} out of 10 nodes appear in edges",
            nodes_in_edges.len()
        );
    }

    #[test]
    fn test_generate_random_graph_data_seed_consistency_across_calls() {
        for seed in [1, 42, 100, 999, 1234567890] {
            let result1 = generate_random_graph_data(5, 8, seed);
            let result2 = generate_random_graph_data(5, 8, seed);
            assert_eq!(result1, result2, "Inconsistent results for seed {}", seed);
        }
    }
}
