#[cfg(any(test, feature = "test-utils"))]
pub mod utils;

use ade_common::INVALID_KEY_SEQUENCE;
use ade_strongly_connected_components::scc_iterative;
use ade_traits::{EdgeTrait, GraphViewTrait, NodeTrait};
use smallvec::SmallVec;

// Johnson's algorithm for finding all elementary circuits in a directed graph
// SIAM J. Comput., Vol. 4, No. 1, March 1975
// https://www.cs.tufts.edu/comp/150GA/homeworks/hw1/Johnson%2075.PDF
pub fn elementary_circuits<N: NodeTrait, E: EdgeTrait>(
    graph: &impl GraphViewTrait<N, E>,
) -> Vec<Vec<u32>> {
    // Panic if the graph does not have sequential keys
    if !graph.has_sequential_keys() {
        panic!("{}", INVALID_KEY_SEQUENCE);
    }

    // Here the algorithm starts
    let mut circuits: Vec<Vec<u32>> = Vec::new();
    let mut stack: Vec<u32> = Vec::new();

    let n = match graph.get_nodes().count() {
        0 => return circuits, // Return empty circuits if no nodes
        len => (len - 1) as u32,
    };

    let mut s: u32 = n; // Start with the maximum node and decrease it, so that graph nodes are always {0, 1, ..., s}
    let size = (n + 1) as usize;
    let mut blocked_set: Vec<bool> = vec![false; size];
    let mut blocked_map: Vec<SmallVec<[u32; 4]>> = vec![SmallVec::new(); size];

    loop {
        // Create the subgraph induced by {0, 1, ..., s}
        let remaining_nodes: Vec<u32> = (0..=s).collect();
        let subgraph = graph.filter(&remaining_nodes);

        // Find the strongly connected components of the subgraph
        let components = scc_iterative(&subgraph);

        // Choose the component containing the max vertex
        let component = components.iter().find(|c| c.contains(&s)).unwrap();

        // Create the subgraph induced by the component containing the max vertex
        let adj = subgraph.filter(component);

        // Find the elementary circuits in the subgraph adj
        if adj.get_nodes().next().is_some() {
            for key in adj.get_node_keys() {
                let k = key as usize;
                blocked_set[k] = false;
                blocked_map[k].clear();
            }

            find_circuit(
                s,
                s,
                &mut circuits,
                &mut stack,
                &mut blocked_set,
                &mut blocked_map,
                &adj,
            );
            if s == 0 {
                break;
            }
            s -= 1;
        } else {
            s = 0;
        }
    }

    circuits
}

fn find_circuit<N: NodeTrait, E: EdgeTrait>(
    s: u32,
    v: u32,
    circuits: &mut Vec<Vec<u32>>,
    stack: &mut Vec<u32>,
    blocked_set: &mut [bool],
    blocked_map: &mut [SmallVec<[u32; 4]>],
    adj: &impl GraphViewTrait<N, E>,
) -> bool {
    let mut f: bool = false;
    let v_us = v as usize;

    stack.push(v);
    blocked_set[v_us] = true;

    for w_key in adj.get_successors_keys(v) {
        if w_key == s {
            let mut circuit = Vec::with_capacity(stack.len() + 1);
            circuit.extend_from_slice(stack);
            circuit.push(s);
            circuits.push(circuit);
            f = true;
        } else if !blocked_set[w_key as usize]
            && find_circuit(s, w_key, circuits, stack, blocked_set, blocked_map, adj)
        {
            f = true;
        }
    }

    if f {
        unblock(v, blocked_set, blocked_map);
    } else {
        for w_key in adj.get_successors_keys(v) {
            let list = &mut blocked_map[w_key as usize];
            if !list.contains(&v) {
                list.push(v);
            }
        }
    }

    stack.pop();
    f
}

fn unblock(u: u32, blocked_set: &mut [bool], blocked_map: &mut [SmallVec<[u32; 4]>]) {
    let u_us = u as usize;
    blocked_set[u_us] = false;

    while let Some(w) = blocked_map[u_us].pop() {
        let w_us = w as usize;
        if blocked_set[w_us] {
            unblock(w, blocked_set, blocked_map);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::circuits_equal;
    use crate::utils::number_circuits;
    use ade_common::{self, assert_panics_with};
    use ade_graph::utils::build::build_graph;
    use ade_graph_generators::complete_graph_data;
    use ade_graph_generators::generate_random_graph_data;
    use graph_cycles::Cycles;
    use petgraph::graph::Graph as PetGraph;
    use rand::Rng;
    use std::collections::HashSet;

    #[test]
    fn test_elementary_circuits_1() {
        let graph = build_graph(vec![0, 1, 2], vec![(0, 1), (1, 2), (2, 1)]);
        let circuits = elementary_circuits(&graph);
        let expected = vec![vec![1, 2, 1]];

        assert!(circuits_equal(&circuits, &expected));
    }

    #[test]
    fn test_elementary_circuits_non_sequential_keys() {
        let graph = build_graph(vec![1, 3, 5], vec![(1, 3), (3, 5), (5, 1)]);
        assert_panics_with!(
            elementary_circuits(&graph),
            ade_common::INVALID_KEY_SEQUENCE
        );
    }

    #[test]
    fn test_elementary_circuits_3() {
        let graph = build_graph(vec![0, 1, 2], vec![(0, 1), (1, 2)]);
        let circuits = elementary_circuits(&graph);
        let expected = vec![];

        assert!(circuits_equal(&circuits, &expected));
    }

    #[test]
    fn test_elementary_circuits_4() {
        let graph = build_graph(vec![0, 1, 2], vec![]);
        let circuits = elementary_circuits(&graph);
        let expected = vec![];

        assert!(circuits_equal(&circuits, &expected));
    }

    #[test]
    fn test_elementary_circuits_5() {
        let graph = build_graph(vec![], vec![]);
        let circuits = elementary_circuits(&graph);
        let expected = vec![];

        assert!(circuits_equal(&circuits, &expected));
    }

    #[test]
    fn test_elementary_circuits_6() {
        let graph = build_graph(vec![0], vec![(0, 0)]);
        let circuits = elementary_circuits(&graph);
        let expected = vec![vec![0, 0]];

        assert!(circuits_equal(&circuits, &expected));
    }

    #[test]
    fn test_elementary_circuits_7() {
        let graph = build_graph(vec![0, 1], vec![(0, 0), (1, 1), (0, 1)]);
        let circuits = elementary_circuits(&graph);
        let expected = vec![vec![0, 0], vec![1, 1]];

        assert!(circuits_equal(&circuits, &expected));
    }

    #[test]
    fn test_elementary_circuits_2() {
        let graph = build_graph(
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            vec![
                (0, 1),
                (0, 7),
                (0, 4),
                (1, 2),
                (1, 6),
                (1, 8),
                (2, 1),
                (2, 0),
                (2, 3),
                (2, 5),
                (3, 4),
                (4, 1),
                (5, 3),
                (7, 8),
                (8, 7),
            ],
        );
        let circuits = elementary_circuits(&graph);
        let expected: Vec<Vec<u32>> = vec![
            vec![0, 1, 2, 0],
            vec![0, 4, 1, 2, 0],
            vec![1, 2, 1],
            vec![1, 2, 3, 4, 1],
            vec![1, 2, 5, 3, 4, 1],
            vec![7, 8, 7],
        ];

        assert!(circuits_equal(&circuits, &expected));
    }

    #[test]
    fn test_elementary_circuits_complete_graph() {
        let n: usize = 6;
        let (nodes, edges) = complete_graph_data(n);
        let graph = build_graph(nodes, edges);
        let circuits = elementary_circuits(&graph);

        assert_eq!(circuits.len(), number_circuits(n))
    }

    // #[test]
    // fn test_elementary_circuits_random_graph() {
    //     let (nodes, edges) = generate_random_graph_data(25, 125, 3);

    //     // Remove duplicate edges
    //     let mut seen = HashSet::new();
    //     let mut unique_edges = Vec::new();
    //     for edge in edges {
    //         if seen.insert(edge) {
    //             unique_edges.push(edge);
    //         }
    //     }

    //     println!("Edges: {:?}", unique_edges.len());

    //     let graph = build_graph(nodes.clone(), unique_edges.clone());
    //     let circuits = elementary_circuits(&graph);

    //     //let g = PetGraph::<(), ()>::from_edges(unique_edges.clone());
    //     //let circuits = g.cycles();

    //     println!("Circuits: {:?}", circuits.len());

    //     //assert_eq!(circuits.len(), cycles.len());
    // }

    #[test]
    fn test_elementary_circuits_multiple_random_graphs() {
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let node_count = rng.gen_range(5..20);
            let edge_count = rng.gen_range(node_count * 2..node_count * 5);
            let seed = rng.gen();

            let (nodes, edges) = generate_random_graph_data(node_count, edge_count, seed);

            // Remove duplicate edges
            let mut seen = HashSet::new();
            let mut unique_edges = Vec::new();
            for edge in edges {
                if seen.insert(edge) {
                    unique_edges.push(edge);
                }
            }

            let graph = build_graph(nodes.clone(), unique_edges.clone());
            let circuits = elementary_circuits(&graph);

            let g = PetGraph::<(), ()>::from_edges(unique_edges.clone());
            let cycles = g.cycles();

            assert_eq!(
                circuits.len(),
                cycles.len(),
                "Mismatch con seed={:?}, nodi={}, edges={}",
                seed,
                node_count,
                edge_count
            );
        }
    }
}
