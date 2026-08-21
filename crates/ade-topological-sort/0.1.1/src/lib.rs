use ade_common::INVALID_KEY_SEQUENCE;
use ade_traits::{EdgeTrait, GraphViewTrait, NodeTrait};
use fixedbitset::FixedBitSet;

pub fn topological_sort<N, E, K, F>(
    graph: &impl GraphViewTrait<N, E>,
    key_fn: Option<F>,
) -> Result<Vec<u32>, String>
where
    N: NodeTrait,
    E: EdgeTrait,
    K: Ord,
    F: Fn(&N) -> K,
{
    fn dfs<N, E, K, F>(
        node_key: u32,
        graph: &impl GraphViewTrait<N, E>,
        visiting: &mut FixedBitSet,
        visited: &mut FixedBitSet,
        result: &mut Vec<u32>,
        key_fn: &Option<F>,
    ) -> Result<(), String>
    where
        N: NodeTrait,
        E: EdgeTrait,
        K: Ord,
        F: Fn(&N) -> K,
    {
        let idx = node_key as usize;

        if visiting[idx] {
            return Err("Graph contains a cycle".into());
        }
        if visited[idx] {
            return Ok(());
        }

        visiting.set(idx, true);

        match key_fn {
            Some(f) => {
                let mut successors = graph.get_successors(node_key).collect::<Vec<_>>();
                successors.sort_by_key(|n| std::cmp::Reverse(f(n)));
                for successor in successors {
                    dfs(successor.key(), graph, visiting, visited, result, key_fn)?;
                }
            }
            None => {
                for successor_key in graph.get_successors_keys(node_key) {
                    dfs(successor_key, graph, visiting, visited, result, key_fn)?;
                }
            }
        }

        visiting.set(idx, false);
        visited.set(idx, true);
        result.push(node_key);
        Ok(())
    }

    // Panic if the graph does not have sequential keys
    if !graph.has_sequential_keys() {
        panic!("{}", INVALID_KEY_SEQUENCE);
    }

    let node_count = graph.get_node_keys().count();
    let mut visiting = FixedBitSet::with_capacity(node_count);
    let mut visited = FixedBitSet::with_capacity(node_count);
    let mut result = Vec::new();

    match &key_fn {
        Some(f) => {
            let mut nodes: Vec<_> = graph.get_nodes().collect();
            nodes.sort_by_key(|n| std::cmp::Reverse(f(n)));
            for node in nodes {
                let idx = node.key() as usize;
                if !visited[idx] {
                    dfs(
                        node.key(),
                        graph,
                        &mut visiting,
                        &mut visited,
                        &mut result,
                        &key_fn,
                    )?;
                }
            }
        }
        None => {
            for node_key in graph.get_node_keys() {
                let idx = node_key as usize;
                if !visited[idx] {
                    dfs(
                        node_key,
                        graph,
                        &mut visiting,
                        &mut visited,
                        &mut result,
                        &key_fn,
                    )?;
                }
            }
        }
    }

    result.reverse();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ade_graph::implementations::Edge;
    use ade_graph::implementations::Graph;
    use ade_graph::implementations::Node;
    use ade_graph::utils::build::build_graph;
    use ade_graph_generators::generate_random_graph_data;
    //use hierarchy::node::{HierarchyNode, NodeType};

    #[test]
    fn test_topological_sort() {
        let n1 = Node::new(0);
        let n2 = Node::new(1);
        let n3 = Node::new(2);

        let e1 = Edge::new(0, 1);
        let e2 = Edge::new(1, 2);

        let graph = Graph::<Node, Edge>::new(vec![n1, n2, n3], vec![e1, e2]);
        let sorted = topological_sort::<Node, Edge, u32, fn(&Node) -> u32>(&graph, None).unwrap();

        assert_eq!(sorted, vec![0, 1, 2]);
    }

    #[test]
    fn test_topological_sort_non_sequential_keys() {
        use ade_common::assert_panics_with;

        let graph = build_graph(vec![1, 3, 5], vec![(1, 3), (3, 5), (5, 1)]);
        assert_panics_with!(
            topological_sort::<Node, Edge, u32, fn(&Node) -> u32>(&graph, None),
            ade_common::INVALID_KEY_SEQUENCE
        );
    }

    #[test]
    fn test_topological_sort_cycle() {
        let n1 = Node::new(0);
        let n2 = Node::new(1);

        let e1 = Edge::new(0, 1);
        let e2 = Edge::new(1, 0); // creates a cycle

        let graph = Graph::<Node, Edge>::new(vec![n1, n2], vec![e1, e2]);

        let result = topological_sort::<Node, Edge, u32, fn(&Node) -> u32>(&graph, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Graph contains a cycle");
    }

    #[test]
    fn test_topological_sort_with_compare_by_key_1() {
        let graph1 = build_graph(vec![0, 1, 2], vec![(0, 1), (0, 2)]);

        assert_eq!(
            topological_sort::<Node, Edge, u32, _>(&graph1, Some(|n: &Node| (n.key() as u32)),)
                .unwrap(),
            vec![0, 1, 2]
        );

        assert_eq!(
            topological_sort::<Node, Edge, i32, _>(&graph1, Some(|n: &Node| -(n.key() as i32)),)
                .unwrap(),
            vec![0, 2, 1]
        );

        let graph2 = build_graph(vec![0, 1, 2], vec![(0, 2), (1, 2)]);

        assert_eq!(
            topological_sort::<Node, Edge, u32, _>(&graph2, Some(|n: &Node| (n.key() as u32)),)
                .unwrap(),
            vec![0, 1, 2]
        );

        assert_eq!(
            topological_sort::<Node, Edge, i32, _>(&graph2, Some(|n: &Node| -(n.key() as i32)),)
                .unwrap(),
            vec![1, 0, 2]
        );

        let graph3 = build_graph(vec![0, 1, 2, 3, 4], vec![(0, 1), (0, 4), (2, 4), (2, 3)]);

        assert_eq!(
            topological_sort::<Node, Edge, u32, _>(&graph3, Some(|n: &Node| (n.key() as u32)),)
                .unwrap(),
            vec![0, 1, 2, 3, 4]
        );

        assert_eq!(
            topological_sort::<Node, Edge, i32, _>(&graph3, Some(|n: &Node| -(n.key() as i32)),)
                .unwrap(),
            vec![2, 3, 0, 4, 1]
        );
    }

    #[test]
    fn test_topological_sort_random_graph() {
        let (nodes, edges) = generate_random_graph_data(20, 20, 3);
        let graph = build_graph(nodes, edges);
        let sorting = topological_sort::<Node, Edge, u32, fn(&Node) -> u32>(&graph, None);
        assert!(sorting.is_ok());
    }

    // #[test]
    // fn test_topological_sort_with_compare_by_layer() {
    //     let mut n1 = HierarchyNode::new(1, NodeType::Base);
    //     let mut n2 = HierarchyNode::new(2, NodeType::Base);
    //     let mut n3 = HierarchyNode::new(3, NodeType::Base);

    //     n1.set_layer(3);
    //     n2.set_layer(2);
    //     n3.set_layer(1);

    //     let e1 = Edge::new(1, 2);
    //     let e2 = Edge::new(1, 3);

    //     let graph: Graph<HierarchyNode, Edge> =
    //         Graph::<HierarchyNode, Edge>::new(vec![n1, n2, n3], vec![e1, e2]);

    //     let sorted = topological_sort(
    //         &graph,
    //         Some(|a: &HierarchyNode, b: &HierarchyNode| a.layer().cmp(&b.layer())),
    //     )
    //     .unwrap();

    //     assert_eq!(sorted, vec![1, 3, 2]);
    // }
}
