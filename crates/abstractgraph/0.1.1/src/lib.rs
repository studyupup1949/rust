//! This crate implements standard graph algorithms without requiring
//! a specific graph representation.
//!
//! It's quite common for graph structures to be defined implicitly by
//! other data structures.  This crate lets you run standard graph
//! algorithms without having to translate your data structures into a
//! specific graph representation.
//!
//! # Usage
//!
//! To define a graph, implement the `DirectedGraph` trait on some
//! data structure can be treated as agraph.  You can then run the
//! various generic graph algorithms which take a generic type
//! implementing `DirectedGraph`.
//!
//! # History
//!
//! This crate began as a port of the [`aga`][1] and [`agar`][2] modules
//! from ccan.
//!
//! [1]: https://ccodearchive.net/info/aga.html
//! [2]: https://ccodearchive.net/info/agar.html

pub mod dfs;
pub mod bfs;

/// The trait for types representing a single graph edge from a known
/// graph node
///
/// This could be a reference to some internal part of your data
/// structure, an index or whatever else is suitable for your graph
/// representation.
pub trait OutEdge<N> {
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
///    `SimpleEdge<Self::Node>`, which is a trivial representation of
///    an (unweighted, directed) edge by its destination.  
///
///  * Set the `Edges` associated type to a convenient representation
///    of a list of edges (usually an iterator or collection).  For
///    simple cases you can use `SimpleEdges` which transforms an
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
    Self::Edge : OutEdge<Self::Node>,
    Self::Edges : std::iter::IntoIterator<Item=Self::Edge>,
{
    /// Represents a node in the graph.
    type Node;

    /// Represents an edge in the graph.
    ///
    /// Implements the `OutEdge<Self::Node>` trait.  Only needs to
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
    fn bfs(&self, start: Self::Node) -> bfs::BreadthFirstSearch<'_, Self>
    where
        Self::Node: Eq+std::hash::Hash,
    {
        let mut bfs = bfs::BreadthFirstSearch::new(self);
        bfs.search_from(start);
        bfs
    }


    /// Traverse `self` in depth-first order from `start`
    fn dfs(&self, start: Self::Node) -> dfs::DepthFirstSearch<'_, Self>
    where
        Self::Node: Eq+std::hash::Hash,
    {
        let mut dfs = dfs::DepthFirstSearch::new(self);
        dfs.search_from(start);
        dfs
    }
}

/// A trivial graph edge representation, whose only information is the
/// destination node
///
/// A single outgoing edge from a known node to another node of type
/// `N`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimpleEdge<N> ( N );

impl<N> SimpleEdge<N> {
    /// Return a new `SimpleEdge` with destination node `to`
    pub fn from_destination(to: N) -> Self {
        SimpleEdge(to)
    }
}

impl<N: Clone> OutEdge<N> for SimpleEdge<N> {
    fn destination(&self) -> N {
        self.0.clone()
    }
    fn into_destination(self) -> N {
        self.0
    }
}

/// A trivial wrapper around an iterator across nodes, changing it to
/// an iterator across `SimpleEdge`s with those nodes as their
/// destinations.
pub struct SimpleEdges<I: Iterator> {
    i: I,
}

impl<I: Iterator> SimpleEdges<I> {
    /// Return a new `SimpleEdges` which for each node yielded by `i`
    /// will yield a `SimpleEdge` to that node.
    pub fn new(i: impl IntoIterator<IntoIter=I, Item=I::Item>) -> SimpleEdges<I> {
        SimpleEdges {
            i: i.into_iter()
        }
    }
}

impl<I: Iterator> Iterator for SimpleEdges<I> {
    type Item = SimpleEdge<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(SimpleEdge::from_destination(self.i.next()?))
    }
}
