use std;

use abstractgraph;
use abstractgraph::DirectedGraph;
use abstractgraph::graphs::SimpleOutboundEdge;
use abstractgraph::graphs::SimpleOutboundEdges;

// Full graph
//
// n nodes with an edge from every node to every other node (including
// itself)
//
pub struct Full {
    nnodes: usize,
}

impl Full {
    pub fn new(nnodes: usize) -> Full {
        Full{ nnodes: nnodes }
    }
}

impl DirectedGraph for Full {
    type Node = usize;
    type Edge = SimpleOutboundEdge<Self::Node>;
    type Edges = SimpleOutboundEdges<std::ops::Range<Self::Node>>;

    fn edges_from(&self, _from: Self::Node) -> Self::Edges {
        SimpleOutboundEdges::new(0..self.nnodes)
    }
}
