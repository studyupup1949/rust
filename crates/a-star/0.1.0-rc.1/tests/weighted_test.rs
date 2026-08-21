use a_star::*;

/// A graph with explicit edges and weights. Nodes are u32 IDs.
struct WeightedGraph {
    edges: Vec<(u32, u32, u32)>, // (from, to, cost)
}

impl WeightedGraph {
    fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Adds a bidirectional edge.
    fn edge(&mut self, a: u32, b: u32, cost: u32) {
        self.edges.push((a, b, cost));
        self.edges.push((b, a, cost));
    }

    /// Adds a one-way edge.
    fn one_way(&mut self, from: u32, to: u32, cost: u32) {
        self.edges.push((from, to, cost));
    }
}

impl Graph<u32, u32> for WeightedGraph {
    fn neighbors(&self, node: u32, neighbors: &mut Vec<NodeCost<u32, u32>>) {
        for &(from, to, cost) in &self.edges {
            if from == node {
                neighbors.push(NodeCost::new(to, cost));
            }
        }
    }

    /// No heuristic — behaves as Dijkstra's.
    fn heuristic(&self, _node: u32, _goal: u32) -> u32 {
        0
    }
}

fn search(graph: &WeightedGraph, start: u32, goal: u32) -> Option<(Vec<u32>, u32)> {
    let discovered: MinHeap<u32, u32> = MinHeap::default();
    let mut a_star: AStar<u32, u32, MinHeap<u32, u32>, WeightedGraph> =
        AStar::new(graph, discovered);
    let path: &Path<u32, u32> = a_star.search(start, goal)?;
    let mut nodes: Vec<u32> = path.nodes().to_vec();
    nodes.reverse();
    Some((nodes, path.cost()))
}

#[test]
fn prefers_cheap_path() {
    //   0 --1-- 1 --1-- 2
    //   |               |
    //   10              10
    //   |               |
    //   3 --1-- 4 --1-- 5
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.edge(0, 1, 1);
    graph.edge(1, 2, 1);
    graph.edge(0, 3, 10);
    graph.edge(2, 5, 10);
    graph.edge(3, 4, 1);
    graph.edge(4, 5, 1);

    let (nodes, cost) = search(&graph, 0, 5).unwrap();
    // Cheap path: 0→1→2→5 = cost 12, or 0→3→4→5 = cost 12. Both are optimal.
    // Actually: 0→1 (1) + 1→2 (1) + 2→5 (10) = 12 and 0→3 (10) + 3→4 (1) + 4→5 (1) = 12.
    assert_eq!(cost, 12);
    assert_eq!(nodes.len(), 4);
}

#[test]
fn short_expensive_vs_long_cheap() {
    //   0 --100-- 1
    //   |         |
    //   1         1
    //   |         |
    //   2 ---1--- 3
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.edge(0, 1, 100);
    graph.edge(0, 2, 1);
    graph.edge(2, 3, 1);
    graph.edge(3, 1, 1);

    let (nodes, cost) = search(&graph, 0, 1).unwrap();
    // Direct: 0→1 = 100. Detour: 0→2→3→1 = 3. Should take detour.
    assert_eq!(cost, 3);
    assert_eq!(nodes, vec![0, 2, 3, 1]);
}

#[test]
fn one_way_edges() {
    //   0 →5→ 1 →5→ 2
    //         ↑
    //   3 →1→ 4
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.one_way(0, 1, 5);
    graph.one_way(1, 2, 5);
    graph.one_way(3, 4, 1);
    graph.one_way(4, 1, 1);

    // 0→2 via 0→1→2 = 10.
    let (nodes, cost) = search(&graph, 0, 2).unwrap();
    assert_eq!(cost, 10);
    assert_eq!(nodes, vec![0, 1, 2]);

    // 3→2 via 3→4→1→2 = 7.
    let (nodes, cost) = search(&graph, 3, 2).unwrap();
    assert_eq!(cost, 7);
    assert_eq!(nodes, vec![3, 4, 1, 2]);
}

#[test]
fn one_way_no_path() {
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.one_way(0, 1, 1);
    // Can go 0→1 but not 1→0.
    let result = search(&graph, 1, 0);
    assert!(result.is_none());
}

#[test]
fn diamond_picks_cheapest() {
    //       1
    //      / \
    //   2     3
    //      \ /
    //       4
    // Top path: 0→1 (2) + 1→3 (3) = 5.
    // Bottom path: 0→2 (1) + 2→3 (1) = 2.
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.one_way(0, 1, 2);
    graph.one_way(0, 2, 1);
    graph.one_way(1, 3, 3);
    graph.one_way(2, 3, 1);

    let (nodes, cost) = search(&graph, 0, 3).unwrap();
    assert_eq!(cost, 2);
    assert_eq!(nodes, vec![0, 2, 3]);
}

#[test]
fn many_hops_cheaper_than_direct() {
    // 0 --100-- 5
    // 0 --1-- 1 --1-- 2 --1-- 3 --1-- 4 --1-- 5
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.edge(0, 5, 100);
    graph.edge(0, 1, 1);
    graph.edge(1, 2, 1);
    graph.edge(2, 3, 1);
    graph.edge(3, 4, 1);
    graph.edge(4, 5, 1);

    let (nodes, cost) = search(&graph, 0, 5).unwrap();
    assert_eq!(cost, 5);
    assert_eq!(nodes, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn zero_cost_edges() {
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.edge(0, 1, 0);
    graph.edge(1, 2, 0);

    let (nodes, cost) = search(&graph, 0, 2).unwrap();
    assert_eq!(cost, 0);
    assert_eq!(nodes, vec![0, 1, 2]);
}

#[test]
fn single_node_graph() {
    let graph: WeightedGraph = WeightedGraph::new();
    let (nodes, cost) = search(&graph, 0, 0).unwrap();
    assert_eq!(cost, 0);
    assert_eq!(nodes, vec![0]);
}

#[test]
fn disconnected_nodes() {
    // Two separate components: 0↔1 and 2↔3.
    let mut graph: WeightedGraph = WeightedGraph::new();
    graph.edge(0, 1, 1);
    graph.edge(2, 3, 1);

    let result = search(&graph, 0, 3);
    assert!(result.is_none());
}
