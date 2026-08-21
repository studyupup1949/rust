//! Structures for the graph representation of the tree, mostly petgraph-based

use std::iter::repeat;
use adic::traits::HasDigits;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::{EdgeRef, NodeRef},
    Directed, Graph,
};
use crate::draw::element::PathGroup;


/// Petgraph structure for `TreeShape`
pub (super) type TreeGraph = Graph<TreeNode, TreeEdge, Directed>;

#[derive(Debug, Clone)]
/// Data for each tree node, i.e. branch splitting
pub (super) struct TreeNode {
    /// x-value for node
    pub x: f64,
    /// y-value for node
    pub y: f64,
    /// Depth into the directed tree, i.e. displacement from root node
    pub depth: isize,
    /// Width of outgoing branch fan
    pub branch_width: f64,
    /// Length of outgoing branch fan
    pub branch_length: f64,
}

#[derive(Debug, Clone)]
/// Data for each tree edge, i.e. branch
pub (super) struct TreeEdge {
    /// Which branch is selected, going from the source to target
    pub branch_choice: u32,
    /// Group to combine into a SVG path draw action
    pub path_group: PathGroup,
}


#[derive(Debug, Clone)]
/// A collection of edge indices, assumed to be connected tip to tail
pub (super) struct TreeBranch(pub Vec<EdgeIndex>);


impl TreeBranch {

    /// Path instructions (Move and Line) to draw the colored branches
    ///
    /// # Errors
    /// Errors if graph gets into a bad state
    pub fn adic_num_on_tree_graph(
        graph: &TreeGraph, adic_data: &impl HasDigits, num_choices: usize,
        root_idx: NodeIndex<u32>, dangling_idx: Option<NodeIndex<u32>>,
    ) -> Self {

        let adic_branch_choices = adic_data.digits().take(num_choices);

        let (mut node_id, choices) = if let Some(d_idx) = dangling_idx {
            let mut choices = vec![0];
            choices.extend(adic_branch_choices);
            (d_idx, choices)
        } else {
            (root_idx, adic_branch_choices.collect::<Vec<_>>())
        };

        let mut instructions = vec![];
        for choice in choices.into_iter().chain(repeat(0)) {
            let mut edges = graph.edges(node_id);
            let edge = edges.find(|e| e.weight().branch_choice == choice);
            if let Some(e) = edge {
                node_id = e.target().id();
                instructions.push(e.id());
            } else {
                break;
            }
        }

        Self(instructions)

    }

}
