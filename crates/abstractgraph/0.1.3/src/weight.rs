use num;

use crate::DirectedGraph;
use crate::OutboundEdge;

/// The trait for types representing a single weighted graph edge from
/// a known graph node
///
/// This could be a reference to some internal part of your data
/// structure, an index or whatever else is suitable for your graph
/// representation.
pub trait WeightedOutboundEdge<N, W: Ord + std::ops::Add>: OutboundEdge<N> {
    /// Returns the weight / cost of this edge
    fn weight(&self) -> W;
    fn into_destination_and_weight(self) -> (N, W)
    where
        Self: Sized
    {
        (self.destination(), self.weight())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WeightedEdge<N, W> {
    destination: N,
    weight: W,
}

impl<N, W> WeightedEdge<N, W> {
    /// Return a new `WeightedEdge` with destination node `to` and
    /// weight w
    pub fn from_destination(to: N, w: W) -> Self {
        WeightedEdge {
            destination: to,
            weight: w,
        }
    }
}

impl<N: Clone, W> OutboundEdge<N> for WeightedEdge<N, W> {
    fn destination(&self) -> N {
        self.destination.clone()
    }
    fn into_destination(self) -> N {
        self.destination
    }
}

impl<N, W> WeightedOutboundEdge<N, W> for WeightedEdge<N, W>
where
    N: Clone,
    W: Clone + Ord + std::ops::Add
{
    fn weight(&self) -> W {
        self.weight.clone()
    }
    fn into_destination_and_weight(self) -> (N, W) {
        (self.destination, self.weight)
    }
}

pub trait WeightedDirectedGraph<W>: DirectedGraph
where
    W: Ord + std::ops::Add
{
    // Marker trait
}

impl<G, W> WeightedDirectedGraph<W> for G
where
    G: DirectedGraph,
    W: Ord + std::ops::Add,
    G::Edge: WeightedOutboundEdge<G::Node, W>
{
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct UnitWeightEdge<E> ( E );

impl<N, E> OutboundEdge<N> for UnitWeightEdge<E>
where
    E: OutboundEdge<N>,
{
    fn destination(&self) -> N {
        self.0.destination()
    }
}

impl<N, W, E>  WeightedOutboundEdge<N, W> for UnitWeightEdge<E>
where
    W: Ord+std::ops::Add+num::One,
    E: OutboundEdge<N>,
{
    fn weight(&self) -> W {
        W::one()
    }
}

pub struct UnitWeightEdges<I> ( I );

impl<I> Iterator for UnitWeightEdges<I>
where
    I: Iterator,
{
    type Item = UnitWeightEdge<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| UnitWeightEdge(e))
    }
}

#[derive(Debug)]
pub struct UnitWeightGraph<G, W> {
    g: G,
    phantom_: std::marker::PhantomData<W>,
}

impl<G, W> UnitWeightGraph<G, W> {
    pub fn new(g: G) -> Self {
        UnitWeightGraph {
            g,
            phantom_: std::marker::PhantomData,
        }
    }
}

impl<G, W> DirectedGraph for UnitWeightGraph<G, W>
where
    G: DirectedGraph,
{
    type Node = G::Node;
    type Edge = UnitWeightEdge<G::Edge>;
    type Edges = UnitWeightEdges<<G::Edges as IntoIterator>::IntoIter>;

    fn edges_from(&self, from: Self::Node) -> Self::Edges {
        UnitWeightEdges(self.g.edges_from(from).into_iter())
    }
}
