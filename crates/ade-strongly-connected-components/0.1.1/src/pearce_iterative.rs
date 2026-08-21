use ade_common::INVALID_KEY_SEQUENCE;
use ade_traits::{EdgeTrait, GraphViewTrait, NodeTrait};
use std::collections::HashMap;

// Struct containing mutable state of the SCC algorithm
struct SccState {
    v_s_front: Vec<u32>,
    v_s_back: Vec<u32>,
    i_s: Vec<usize>,
    rindex: Vec<u32>,
    index: usize,
    root: Vec<bool>,
    c: isize,
}

impl SccState {
    fn new(n: usize) -> Self {
        Self {
            v_s_front: Vec::with_capacity(n),
            v_s_back: Vec::with_capacity(n),
            i_s: Vec::with_capacity(n),
            rindex: vec![0; n],
            index: 1,
            root: vec![false; n],
            c: n as isize - 1,
        }
    }
}

// D.J. Pearce's algorithm for finding strongly connected components (SCCs).
// Information Processing Letters 116 (2016) 47-52
// Iterative implementation.
pub fn scc_iterative<N: NodeTrait, E: EdgeTrait>(
    graph: &impl GraphViewTrait<N, E>,
) -> Vec<Vec<u32>> {
    // Panic if the graph does not have sequential keys
    if !graph.has_sequential_keys() {
        panic!("{}", INVALID_KEY_SEQUENCE);
    }

    let nodes = graph.get_nodes();
    let n: usize = graph.get_nodes().count();
    let mut state = SccState::new(n);

    for v in nodes {
        if state.rindex[v.key() as usize] == 0 {
            visit(v, &mut state, graph)
        }
    }

    fn visit<N: NodeTrait, E: EdgeTrait>(
        v: &N,
        state: &mut SccState,
        graph: &impl GraphViewTrait<N, E>,
    ) {
        begin_visiting(v.key(), state);
        while !state.v_s_front.is_empty() {
            visit_loop(state, graph);
        }
    }

    fn visit_loop<N: NodeTrait, E: EdgeTrait>(
        state: &mut SccState,
        graph: &impl GraphViewTrait<N, E>,
    ) {
        let v = *state.v_s_front.last().unwrap();
        let mut i = *state.i_s.last().unwrap();

        let mut prev: Option<u32> = None;
        for (k, w) in graph.get_successors_keys(v).enumerate() {
            if k + 1 < i {
                prev = Some(w);
                continue;
            }

            if let Some(p) = prev {
                finish_edge(v, p, state);
            }

            if begin_edge(k + 1, w, state) {
                return;
            }

            prev = Some(w);
            i += 1;
        }

        if let Some(p) = prev {
            finish_edge(v, p, state);
        }

        finish_visiting(v, state);
    }

    #[inline(always)]
    fn finish_visiting(v: u32, state: &mut SccState) {
        state.v_s_front.pop();
        state.i_s.pop();
        if state.root[v as usize] {
            state.index -= 1;
            while !state.v_s_back.is_empty()
                && state.rindex[v as usize]
                    <= state.rindex[*state.v_s_back.last().unwrap() as usize]
            {
                let w = state.v_s_back.pop().unwrap();
                state.rindex[w as usize] = state.c as u32;
                state.index -= 1;
            }
            state.rindex[v as usize] = state.c as u32;
            state.c -= 1;
        } else {
            state.v_s_back.push(v);
        }
    }

    #[inline(always)]
    fn finish_edge(v: u32, w: u32, state: &mut SccState) {
        if state.rindex[w as usize] < state.rindex[v as usize] {
            state.rindex[v as usize] = state.rindex[w as usize];
            state.root[v as usize] = false;
        }
    }

    #[inline(always)]
    fn begin_edge(k: usize, w: u32, state: &mut SccState) -> bool {
        if state.rindex[w as usize] == 0 {
            if let Some(last) = state.i_s.last_mut() {
                *last = k + 1;
            }
            begin_visiting(w, state);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn begin_visiting(v: u32, state: &mut SccState) {
        state.v_s_front.push(v);
        state.i_s.push(0);
        state.root[v as usize] = true;
        state.rindex[v as usize] = state.index as u32;
        state.index += 1;
    }

    fn get_components(rindex: &[u32]) -> Vec<Vec<u32>> {
        let mut map: HashMap<u32, Vec<u32>> = HashMap::new();

        for (i, &val) in rindex.iter().enumerate() {
            map.entry(val).or_default().push(i as u32);
        }

        map.into_values().collect()
    }

    get_components(&state.rindex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scc;
    use ade_common::{self, assert_panics_with};
    use ade_graph::utils::build::build_graph;
    use ade_graph_generators::generate_random_graph_data;

    fn sort_components(components: &mut Vec<Vec<u32>>) {
        for component in components.iter_mut() {
            component.sort_unstable();
        }
        components.sort_unstable_by_key(|g| g[0]);
    }

    #[test]
    fn test_scc_iterative_5() {
        let graph = build_graph(
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![
                (0, 1),
                (0, 4),
                (1, 2),
                (2, 3),
                (4, 7),
                (3, 1),
                (4, 0),
                (4, 5),
                (5, 6),
                (6, 4),
                (8, 9),
                (9, 8),
            ],
        );
        let mut components = scc_iterative(&graph);
        sort_components(&mut components);
        assert_eq!(
            components,
            vec![vec![0, 4, 5, 6], vec![1, 2, 3], vec![7], vec![8, 9]]
        );
    }

    #[test]
    fn test_scc_iterative_6() {
        let graph = build_graph(
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![
                (1, 0),
                (2, 1),
                (2, 6),
                (2, 7),
                (3, 1),
                (3, 6),
                (4, 0),
                (5, 0),
                (5, 6),
                (8, 2),
                (8, 7),
                (8, 9),
                (9, 4),
                (9, 6),
            ],
        );
        let mut components = scc_iterative(&graph);
        sort_components(&mut components);
        assert_eq!(
            components,
            vec![
                vec![0],
                vec![1],
                vec![2],
                vec![3],
                vec![4],
                vec![5],
                vec![6],
                vec![7],
                vec![8],
                vec![9],
            ]
        );
    }

    #[test]
    fn test_scc_on_random_graph() {
        let (nodes, edges) = generate_random_graph_data(10000, 20000, 123);
        let graph = build_graph(nodes, edges);

        let mut components = scc(&graph);
        sort_components(&mut components);

        let mut components1 = scc_iterative(&graph);
        sort_components(&mut components1);

        assert_eq!(components, components1)
    }

    #[test]
    fn test_scc_on_multiple_fixed_random_graphs() {
        let graph_sizes = [
            (0, 0, 123),
            (1, 0, 12),
            (2, 1, 1),
            (10, 20, 5),
            (15, 30, 7),
            (20, 50, 11),
            (25, 60, 13),
            (30, 70, 17),
            (35, 80, 19),
            (40, 100, 23),
            (45, 120, 29),
            (50, 140, 31),
            (55, 160, 37),
            (60, 180, 41),
            (65, 200, 43),
            (70, 220, 47),
            (75, 240, 53),
            (80, 260, 59),
            (85, 280, 61),
            (90, 300, 67),
            (95, 320, 71),
            (100, 340, 73),
            (105, 360, 79),
            (110, 380, 83),
            (115, 400, 89),
            (120, 420, 97),
            (125, 440, 101),
            (130, 460, 103),
            (135, 480, 107),
            (11, 11, 5),
            (17, 17, 7),
            (21, 21, 11),
            (27, 27, 13),
            (33, 33, 17),
            (37, 37, 19),
            (41, 41, 23),
            (47, 471, 29),
            (51, 511, 31),
            (57, 577, 37),
            (59, 591, 41),
            (67, 671, 43),
            (71, 709, 47),
            (76, 1022, 53),
            (83, 2606, 59),
            (85, 280, 61),
            (131, 3107, 67),
            (985, 32010, 71),
        ];

        for &(nodes_count, edges_count, seed) in &graph_sizes {
            let (nodes, edges) = generate_random_graph_data(nodes_count, edges_count, seed);
            let graph = build_graph(nodes, edges);

            let mut components = scc(&graph);
            sort_components(&mut components);

            let mut components_iter = scc_iterative(&graph);
            sort_components(&mut components_iter);

            assert_eq!(
                components, components_iter,
                "Mismatch for graph with {} nodes, {} edges, seed {}",
                nodes_count, edges_count, seed
            );
        }
    }

    #[test]
    fn test_scc_on_multiple_random_graphs() {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let nodes_count = rng.gen_range(1..200);
            let edges_count = rng.gen_range(0..(nodes_count * 10));
            let seed = rng.gen();

            let (nodes, edges) = generate_random_graph_data(nodes_count, edges_count, seed);
            let graph = build_graph(nodes, edges);

            let mut components = scc(&graph);
            sort_components(&mut components);

            let mut components_iter = scc_iterative(&graph);
            sort_components(&mut components_iter);

            assert_eq!(
                components, components_iter,
                "Mismatch con seed={:?}, nodi={}, edges={}",
                seed, nodes_count, edges_count
            );
        }
    }

    #[test]
    fn test_scc_iterative_non_sequential_keys() {
        use ade_common::assert_panics_with;

        let graph = build_graph(vec![1, 3, 5], vec![(1, 3), (3, 5), (5, 1)]);
        assert_panics_with!(scc_iterative(&graph), ade_common::INVALID_KEY_SEQUENCE);
    }
}
