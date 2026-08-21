use std::cmp::Ordering;

/// A node with an associated cost.
///
/// Only the `cost` is used for comparison.
#[derive(Copy, Clone, Debug)]
pub struct NodeCost<Node, Cost> {
    pub node: Node,
    pub cost: Cost,
}

impl<Node, Cost> NodeCost<Node, Cost> {
    //! Construction

    /// Creates a new node cost.
    pub const fn new(node: Node, cost: Cost) -> Self {
        Self { node, cost }
    }
}

impl<Node, Cost> Ord for NodeCost<Node, Cost>
where
    Cost: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}

impl<Node, Cost> PartialOrd for NodeCost<Node, Cost>
where
    Cost: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

impl<Node, Cost> Eq for NodeCost<Node, Cost> where Cost: Eq {}

impl<Node, Cost> PartialEq for NodeCost<Node, Cost>
where
    Cost: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}
