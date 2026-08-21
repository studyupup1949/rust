//! Basic definitions for directed graphs
//!

use crate::algorithms;

/// The trait for types representing a single graph edge from a known
/// graph node
///
/// This could be a reference to some internal part of your data
/// structure, an index or whatever else is suitable for your graph
/// representation.
pub trait OutboundEdge<N> {
    /// Returns the destination node of `self`
    fn destination(&self) -> N;

    /// Consume `self` and return its destination node instead
    fn into_destination(self) -> N
    where
        Self: Sized
    {
        self.destination()
    }
}

/// An iterator through the neighbors of a given node (nodes reachable
/// by a single edge transition from the given node) in a graph.
pub struct Neighbors<G: DirectedGraph> {
    i: <G::Edges as IntoIterator>::IntoIter,
}

impl<G: DirectedGraph> Iterator for Neighbors<G> {
    type Item = G::Node;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.i.next()?.into_destination())
    }
}

/// The trait for types (implicitly or explicitly) representing a
/// directed graph structure
///
/// To implement this trait:
///
///  * Set the `Node` associated type to a convenient representation
///    for graph nodes
///
///  * Set the `Edge` associated types to a convenient representation
///    for an edge in your graph.  For simple cases you can use
///    `SimpleOutboundEdge<Self::Node>`, which is a trivial representation of
///    an (unweighted, directed) edge by its destination.  
///
///  * Set the `Edges` associated type to a convenient representation
///    of a list of edges (usually an iterator or collection).  For
///    simple cases you can use `SimpleOutboundEdges` which transforms an
///    iterator over nodes (an adjacency list) into an iterator over
///    edges.
///
///  * Implement `edges_from()` to return the list of edges
///    originating at a specific node.
///
/// Theoretically infinite graphs are permitted, though you obviously
/// won't be able to fully traverse them.
pub trait DirectedGraph where
    Self: Sized,
    Self::Node : Clone,
    Self::Edge : OutboundEdge<Self::Node>,
    Self::Edges : std::iter::IntoIterator<Item=Self::Edge>,
{
    /// Represents a node in the graph.
    type Node;

    /// Represents an edge in the graph.
    ///
    /// Implements the `OutboundEdge<Self::Node>` trait.  Only needs to
    /// represent an edge given it's origin node, so it need not be
    /// globally unique (or even locally, you can return the same
    /// `Edge` twice from `edges_from` if you want and the algorithms
    /// will treat them as distinct edges).
    type Edge;

    /// Represents a list of graph edges (with a common origin) in the
    /// graph
    ///
    /// Implements `IntoIterator<Item=Self::Edge>`, with that iterator
    /// stepping through each ednge outgoing from a single node.
    type Edges;

    /// Returns the list of edges originating from node `from`
    fn edges_from(&self, from: Self::Node) -> Self::Edges;

    /// Returns the (outgoing) adjacency list for node `from`
    ///
    /// This is the list (represented by an iterator) of nodes
    /// reachable from `from` by exactly one edge transition.
    fn neighbors(&self, from: Self::Node) -> Neighbors<Self> {
        Neighbors {
            i: self.edges_from(from).into_iter(),
        }
    }

    /// Traverse `self` in breadth-first order from `start`
    fn bfs(&self, start: Self::Node) -> algorithms::bfs::BreadthFirstSearch<'_, Self>
    where
        Self::Node: Eq+std::hash::Hash,
    {
        let mut bfs = algorithms::bfs::BreadthFirstSearch::new(self);
        bfs.search_from(start);
        bfs
    }


    /// Traverse `self` in depth-first order from `start`
    fn dfs(&self, start: Self::Node) -> algorithms::dfs::DepthFirstSearch<'_, Self>
    where
        Self::Node: Eq+std::hash::Hash,
    {
        let mut dfs = algorithms::dfs::DepthFirstSearch::new(self);
        dfs.search_from(start);
        dfs
    }
}
