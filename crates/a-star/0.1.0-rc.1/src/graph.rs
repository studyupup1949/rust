use crate::NodeCost;

/// Responsible for providing the graph structure for `A*` search.
pub trait Graph<Node, Cost> {
    /// Adds the neighbors of the `node` with their edge costs to the `neighbors` buffer.
    fn neighbors(&self, node: Node, neighbors: &mut Vec<NodeCost<Node, Cost>>);

    /// Estimates the lower limit of the cost from the `node` to the `goal`.
    fn heuristic(&self, node: Node, goal: Node) -> Cost;
}
