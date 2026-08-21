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

pub mod directedgraph;
pub mod weight;

pub use directedgraph::OutboundEdge;
pub use directedgraph::DirectedGraph;
pub use directedgraph::Neighbors;

pub use weight::WeightedDirectedGraph;

pub mod graphs;
pub mod algorithms;
