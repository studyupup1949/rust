//! Concrete implementations for abstract graphs

use crate::directedgraph::OutboundEdge;

/// A trivial graph edge representation, whose only information is the
/// destination node
///
/// A single outgoing edge from a known node to another node of type
/// `N`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimpleOutboundEdge<N> ( N );

impl<N> SimpleOutboundEdge<N> {
    /// Return a new `SimpleOutboundEdge` with destination node `to`
    pub fn from_destination(to: N) -> Self {
        SimpleOutboundEdge(to)
    }
}

impl<N: Clone> OutboundEdge<N> for SimpleOutboundEdge<N> {
    fn destination(&self) -> N {
        self.0.clone()
    }
    fn into_destination(self) -> N {
        self.0
    }
}

/// A trivial wrapper around an iterator across nodes, changing it to
/// an iterator across `SimpleOutboundEdge`s with those nodes as their
/// destinations.
pub struct SimpleOutboundEdges<I: Iterator> {
    i: I,
}

impl<I: Iterator> SimpleOutboundEdges<I> {
    /// Return a new `SimpleOutboundEdges` which for each node yielded by `i`
    /// will yield a `SimpleOutboundEdge` to that node.
    pub fn new(i: impl IntoIterator<IntoIter=I, Item=I::Item>) -> SimpleOutboundEdges<I> {
        SimpleOutboundEdges {
            i: i.into_iter()
        }
    }
}

impl<I: Iterator> Iterator for SimpleOutboundEdges<I> {
    type Item = SimpleOutboundEdge<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(SimpleOutboundEdge::from_destination(self.i.next()?))
    }
}
