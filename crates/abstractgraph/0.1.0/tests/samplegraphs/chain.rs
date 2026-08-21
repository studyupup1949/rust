use abstractgraph;
use abstractgraph::DirectedGraph;
use abstractgraph::SimpleEdge;
use abstractgraph::SimpleEdges;

// Chain graph
//
//  --> --> -->
// A   B   C   D
//  <-- <-- <--
//
// nnodes nodes arranged in a linear sequence, edges from each node to
// the previous and next
//
pub struct Chain {
    nnodes: usize,
}

impl Chain {
    pub fn new(nnodes: usize) -> Chain {
        Chain{ nnodes: nnodes }
    }
}

impl DirectedGraph for Chain {
    type Node = usize;
    type Edge = SimpleEdge<Self::Node>;
    type Edges = SimpleEdges<<Vec<Self::Node> as IntoIterator>::IntoIter>;

    fn edges_from(&self, from: Self::Node) -> Self::Edges {
        if from == 0 {
            SimpleEdges::new(vec![1])
        } else if from == self.nnodes - 1 {
            SimpleEdges::new(vec![self.nnodes - 2])
        } else {
            SimpleEdges::new(vec![from - 1, from + 1])
        }
    }
}
