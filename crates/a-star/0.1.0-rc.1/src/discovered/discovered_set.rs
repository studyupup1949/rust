use crate::NodeCost;

/// Responsible for keeping track of the discovered nodes.
pub trait DiscoveredSet<Node, Cost>
where
    Node: Copy,
    Cost: Copy + Ord,
{
    /// Pushes the `node` with its associated `f(n)` cost.
    fn push(&mut self, node: NodeCost<Node, Cost>);

    /// Pops the node with the lowest `f(n)` cost.
    fn pop(&mut self) -> Option<NodeCost<Node, Cost>>;

    /// Removes all the nodes from the set.
    fn clear(&mut self);
}
