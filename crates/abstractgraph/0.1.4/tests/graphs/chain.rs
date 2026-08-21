use abstractgraph;
use abstractgraph::graphs::SimpleOutboundEdge;
use abstractgraph::graphs::SimpleOutboundEdges;
use abstractgraph::DirectedGraph;

// Chain graph
//
//  --> --> -->
// A   B   C   D
//  <-- <-- <--
//
// nnodes nodes arranged in a linear sequence, edges from each node to
// the previous and next
//
#[derive(Debug)]
pub struct Chain {
    nnodes: usize,
}

impl Chain {
    pub fn new(nnodes: usize) -> Chain {
        Chain { nnodes: nnodes }
    }
}

impl DirectedGraph for Chain {
    type Node = usize;
    type Edge = SimpleOutboundEdge<Self::Node>;
    type Edges = SimpleOutboundEdges<<Vec<Self::Node> as IntoIterator>::IntoIter>;

    fn edges_from(&self, from: &Self::Node) -> Self::Edges {
        if *from == 0 {
            SimpleOutboundEdges::new(vec![1])
        } else if *from == self.nnodes - 1 {
            SimpleOutboundEdges::new(vec![self.nnodes - 2])
        } else {
            SimpleOutboundEdges::new(vec![from - 1, from + 1])
        }
    }
}
