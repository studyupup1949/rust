use a_star::*;

/// A simple grid graph for testing. Nodes are (x, y) coordinates on a grid.
/// Blocked tiles are impassable. All edges have cost 1. Heuristic is manhattan distance.
struct GridGraph {
    width: i32,
    height: i32,
    blocked: Vec<(i32, i32)>,
}

impl GridGraph {
    fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            blocked: Vec::new(),
        }
    }

    fn block(&mut self, x: i32, y: i32) {
        self.blocked.push((x, y));
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.blocked.contains(&(x, y))
    }
}

impl Graph<(i32, i32), u32> for GridGraph {
    fn neighbors(&self, node: (i32, i32), neighbors: &mut Vec<NodeCost<(i32, i32), u32>>) {
        let (x, y): (i32, i32) = node;
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny): (i32, i32) = (x + dx, y + dy);
            if self.in_bounds(nx, ny) && !self.is_blocked(nx, ny) {
                neighbors.push(NodeCost::new((nx, ny), 1));
            }
        }
    }

    fn heuristic(&self, node: (i32, i32), goal: (i32, i32)) -> u32 {
        let dx: u32 = (node.0 - goal.0).unsigned_abs();
        let dy: u32 = (node.1 - goal.1).unsigned_abs();
        dx + dy
    }
}

fn search(
    graph: &GridGraph,
    start: (i32, i32),
    goal: (i32, i32),
) -> Option<(Vec<(i32, i32)>, u32)> {
    let discovered: MinHeap<(i32, i32), u32> = MinHeap::default();
    let mut a_star: AStar<(i32, i32), u32, MinHeap<(i32, i32), u32>, GridGraph> =
        AStar::new(graph, discovered);
    let path: &Path<(i32, i32), u32> = a_star.search(start, goal)?;
    let mut nodes: Vec<(i32, i32)> = path.nodes().to_vec();
    nodes.reverse();
    Some((nodes, path.cost()))
}

#[test]
fn direct_path() {
    let graph: GridGraph = GridGraph::new(5, 5);
    let (nodes, cost) = search(&graph, (0, 0), (4, 0)).unwrap();
    assert_eq!(cost, 4);
    assert_eq!(nodes.len(), 5);
    assert_eq!(nodes[0], (0, 0));
    assert_eq!(nodes[4], (4, 0));
}

#[test]
fn same_start_and_goal() {
    let graph: GridGraph = GridGraph::new(5, 5);
    let (nodes, cost) = search(&graph, (2, 2), (2, 2)).unwrap();
    assert_eq!(cost, 0);
    assert_eq!(nodes, vec![(2, 2)]);
}

#[test]
fn path_around_wall() {
    let mut graph: GridGraph = GridGraph::new(5, 5);
    // Wall across the middle, blocking y=2 except at x=4.
    graph.block(0, 2);
    graph.block(1, 2);
    graph.block(2, 2);
    graph.block(3, 2);

    let (nodes, cost) = search(&graph, (0, 0), (0, 4)).unwrap();
    // Must go around: right to x=4, down past wall, left to x=0.
    assert_eq!(nodes[0], (0, 0));
    assert_eq!(*nodes.last().unwrap(), (0, 4));
    // Shortest is: right 4, down 1, down past wall, down 1, left 4 = some path >= 12.
    assert!(cost >= 12);
}

#[test]
fn no_path() {
    let mut graph: GridGraph = GridGraph::new(5, 5);
    // Completely wall off the goal.
    graph.block(1, 0);
    graph.block(0, 1);
    graph.block(1, 1);

    // (0,0) is isolated from (4,4) if (0,0) is boxed in.
    // Actually (0,0) can only reach itself — neighbors are (1,0) blocked and (0,1) blocked.
    let result = search(&graph, (0, 0), (4, 4));
    assert!(result.is_none());
}

#[test]
fn diagonal_path() {
    let graph: GridGraph = GridGraph::new(4, 4);
    let (nodes, cost) = search(&graph, (0, 0), (3, 3)).unwrap();
    // Manhattan distance = 6, path must have 7 nodes.
    assert_eq!(cost, 6);
    assert_eq!(nodes.len(), 7);
    assert_eq!(nodes[0], (0, 0));
    assert_eq!(*nodes.last().unwrap(), (3, 3));
}

#[test]
fn reuse_across_searches() {
    let graph: GridGraph = GridGraph::new(5, 5);
    let discovered: MinHeap<(i32, i32), u32> = MinHeap::default();
    let mut a_star: AStar<(i32, i32), u32, MinHeap<(i32, i32), u32>, GridGraph> =
        AStar::new(&graph, discovered);

    // First search.
    let path: &Path<(i32, i32), u32> = a_star.search((0, 0), (4, 0)).unwrap();
    assert_eq!(path.cost(), 4);

    // Second search reuses the same instance.
    let path: &Path<(i32, i32), u32> = a_star.search((0, 0), (0, 4)).unwrap();
    assert_eq!(path.cost(), 4);
}

#[test]
fn optimal_cost_with_detour() {
    let mut graph: GridGraph = GridGraph::new(5, 1);
    // Straight line 0→4, but block 2.
    graph.block(2, 0);

    // 1D graph with a gap — no path.
    let result = search(&graph, (0, 0), (4, 0));
    assert!(result.is_none());
}
