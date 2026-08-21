use std::iter;

use abstractgraph;
use abstractgraph::DirectedGraph;
use abstractgraph::OutboundEdge;

// Trivial graph
//
//      A
//
// The simplest possible graph: one node, no edges
//
pub struct Trivial();

pub enum NoEdge {
}

impl OutboundEdge<()> for NoEdge {
    fn destination(&self) -> () {
        unreachable!()
    }
}

impl DirectedGraph for Trivial {
    type Node = ();
    type Edge = NoEdge;
    type Edges = iter::Empty<Self::Edge>;

    fn edges_from(&self, _from: Self::Node) -> Self::Edges {
        iter::empty()
    }
}
