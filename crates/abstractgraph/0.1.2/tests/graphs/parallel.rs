use std;

use abstractgraph;

use abstractgraph::DirectedGraph;
use abstractgraph::graphs::SimpleOutboundEdge;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Node { A, B }

// Parallel graph
//
//          --
//         /  \
//      (A)    => (B)
//         \  /
//          --
//
// Two nodes (A & B), with nlinks edges all from A->B.
//
pub struct Parallel {
    nlinks: usize,
}

impl Parallel {
    pub fn new(nlinks: usize) -> Parallel {
        Parallel{ nlinks: nlinks }
    }
}

impl DirectedGraph for Parallel {
    type Node = Node;
    type Edge = SimpleOutboundEdge<Self::Node>;
    type Edges = std::iter::Take<std::iter::Repeat<Self::Edge>>;

    fn edges_from(&self, from: Self::Node) -> Self::Edges {
        let n = match from {
            Node::A => self.nlinks,
            Node::B => 0,
        };

        std::iter::repeat(SimpleOutboundEdge::from_destination(Node::B)).take(n)
    }
}
