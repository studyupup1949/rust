use crate::utils::ShortEdge;
use graph_types::{
    placeholder::{PlaceholderEdgeIterator, PlaceholderNodeIterator},
    Edge, EdgeDirection, EdgeInsertID, EdgeQuery, GraphEngine, GraphKind, MutableGraph, NodeRangeVisitor, NodesVisitor,
};
use std::collections::{BTreeMap, BTreeSet};

mod one_way;
pub mod one_way_iter;
mod two_way;

#[doc = include_str!("AdjacencyEdgeList.html")]
#[derive(Debug)]
pub struct AdjacencyEdgeDict<const ONE_WAY: bool> {
    nodes: BTreeSet<u32>,
    edges: BTreeMap<u32, ShortEdge>,
}
