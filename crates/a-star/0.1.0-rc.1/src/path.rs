use std::fmt::Debug;
use std::hash::Hash;

/// A node path.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Path<Node, Cost>
where
    Cost: Copy + Default,
{
    nodes: Vec<Node>,
    cost: Cost,
}

impl<Node, Cost> Path<Node, Cost>
where
    Cost: Copy + Default,
{
    //! Construction

    /// Creates a new path with the given initial `capacity`.
    pub fn new(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            cost: Cost::default(),
        }
    }
}

impl<Node, Cost> Path<Node, Cost>
where
    Cost: Copy + Default,
{
    //! Properties

    /// Gets the nodes.
    pub fn nodes(&self) -> &[Node] {
        self.nodes.as_slice()
    }

    /// Gets the path cost.
    pub fn cost(&self) -> Cost {
        self.cost
    }
}

impl<Node, Cost> Path<Node, Cost>
where
    Cost: Copy + Default,
{
    //! Mutations

    /// Clears the path.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.cost = Cost::default();
    }

    /// Sets the cost.
    pub fn set_cost(&mut self, cost: Cost) {
        self.cost = cost;
    }

    /// Adds the node.
    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Reverses the nodes.
    pub fn reverse(&mut self) {
        self.nodes.reverse();
    }
}
