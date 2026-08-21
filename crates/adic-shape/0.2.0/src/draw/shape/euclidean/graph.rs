//! `petgraph` structures for the graph representation of the euclidean

use std::{
    collections::HashSet,
    fmt::Debug,
};
use itertools::Itertools;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    Directed, Graph,
};
use crate::{
    draw::element::{PathColor, PathGroup, PathStroke},
    error::{AdicShapeError, AdicShapeResult},
};
use super::visitor::EuclideanCtorVisitor;


pub (super) type EuclideanPetgraph = Graph<EuclideanNode, EuclideanEdge, Directed>;


#[derive(Debug, Clone)]
/// Petgraph structure for `EuclideanShape`
pub (super) struct EuclideanGraph {
    petgraph: EuclideanPetgraph,
    root_idx: NodeIndex,
    colored_branches: Vec<EuclideanBranch>,
    scaling: f64,
    depth: isize,
    draw_full_tree: PathStroke,
    draw_scaled_hulls: bool,
    draw_scaled_dots: bool,
    enclosing_disks: Vec<isize>,
}

impl EuclideanGraph {

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        visitor: &impl EuclideanCtorVisitor,
        colored_branches: Vec<Vec<u32>>,
        min_valuation: isize,
        draw_full_tree: PathStroke,
        draw_scaled_hulls: bool,
        draw_scaled_dots: bool,
        enclosing_disks: Vec<isize>,
        show_zero_val: bool,
    ) -> AdicShapeResult<Self> {

        let mut graph = Graph::new();

        // Add root node
        // Start from 0; this will be adjusted in EuclideanShape
        let (x, y) = (0.0, 0.0);
        let root_idx = graph.add_node(EuclideanNode{x, y, depth: min_valuation - 1, scaling: 1.0, choices: vec![]});

        if draw_full_tree != PathStroke::NoStroke || draw_scaled_hulls || draw_scaled_dots {
            visitor.grow_all_branches(&mut graph, root_idx)?;
        }

        let mut colored_branch_edges = vec![];
        for colored_branch in colored_branches {
            let branch = visitor.grow_full_branch(&mut graph, root_idx, &colored_branch)?;
            colored_branch_edges.push(EuclideanBranch(
                branch.into_iter().map(|(eix, _nix)| eix).collect::<Vec<_>>()
            ));
        }

        // Grow the zero valuation stems if showing the zero valuation convex hull
        if show_zero_val {
            visitor.grow_zero_valuation_stems(&mut graph)?;
        }

        let mut s = Self {
            petgraph: graph,
            root_idx,
            colored_branches: colored_branch_edges,
            scaling: visitor.scaling_multiplier(),
            depth: visitor.max_depth(),
            draw_full_tree,
            draw_scaled_hulls,
            draw_scaled_dots,
            enclosing_disks,
        };

        s.paint()?;

        Ok(s)

    }


    pub fn petgraph(&self) -> &EuclideanPetgraph {
        &self.petgraph
    }

    pub fn root_idx(&self) -> NodeIndex {
        self.root_idx
    }

    pub fn scaling_multiplier(&self) -> f64 {
        self.scaling
    }

    pub fn depth(&self) -> isize {
        self.depth
    }

    pub fn draw_scaled_hulls(&self) -> bool {
        self.draw_scaled_hulls
    }

    pub fn draw_scaled_dots(&self) -> bool {
        self.draw_scaled_dots
    }

    pub fn enclosing_disks(&self) -> &Vec<isize> {
        &self.enclosing_disks
    }


    fn paint(&mut self) -> AdicShapeResult<()> {

        // Reset graph color to default
        for edge_weight in self.petgraph.edge_weights_mut() {
            edge_weight.path_group = PathGroup{ stroke: self.draw_full_tree, ..Default::default() };
        }

        // Calculate shared edges to graph with combined color
        let mut shared_edges = HashSet::new();
        let mut edge_iter = self.colored_branches.iter().flat_map(|b| b.0.iter()).sorted().peekable();
        while let Some(edge) = edge_iter.next() {
            while let Some(&next_edge) = edge_iter.peek() {
                if next_edge == edge {
                    shared_edges.insert(*next_edge);
                    edge_iter.next();
                } else {
                    break;
                }
            }
        }

        // Recolor shared edges
        let path_group = PathGroup{ color_group: PathColor::Combined, ..Default::default() };
        for e_idx in &shared_edges {
            let edge = self.petgraph.edge_weight_mut(*e_idx).ok_or(AdicShapeError::PetGraph)?;
            edge.path_group = path_group;
        }

        // Recolor unique colored edges
        for (idx, branch) in self.colored_branches.iter().enumerate() {

            let color_idx = (u32::try_from(idx)? % MAX_BRANCH_COLORS);
            let path_group = PathGroup{ color_group: PathColor::Color(color_idx), ..Default::default() };

            let unique_edges = branch.0.iter().copied().collect::<HashSet<_>>();
            let unique_edges = unique_edges.difference(&shared_edges);
            for e_idx in unique_edges {
                let edge = self.petgraph.edge_weight_mut(*e_idx).ok_or(AdicShapeError::PetGraph)?;
                edge.path_group = path_group;
            }

        }

        Ok(())

    }

}


#[derive(Debug, Clone)]
/// Data for each euclidean node, i.e. branch splitting
pub (super) struct EuclideanNode {
    /// x-value for node
    pub x: f64,
    /// y-value for node
    pub y: f64,
    /// Depth into the directed euclidean, i.e. displacement from root node
    pub depth: isize,
    /// Total scaling for this node of the euclidean, the product of all branch scalings
    pub scaling: f64,
    /// Choices chosen to arrive at this Node, from `root_idx`
    pub choices: Vec<u32>,
}

#[derive(Debug, Clone)]
/// Data for each euclidean edge, i.e. branch
pub (super) struct EuclideanEdge {
    /// Which branch is selected, going from the source to target
    pub branch_choice: u32,
    /// Group to combine into a SVG path draw action
    pub path_group: PathGroup,
}


#[derive(Debug, Clone)]
/// A collection of edge indices, assumed to be connected tip to tail
pub (super) struct EuclideanBranch(Vec<EdgeIndex>);



// TODO: Take this out and switch to use palette crate
const MAX_BRANCH_COLORS: u32 = 6;
