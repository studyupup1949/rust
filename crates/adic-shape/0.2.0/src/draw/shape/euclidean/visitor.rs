use std::{
    f64::consts::TAU,
    fmt::Debug,
};
use adic::divisible::{Divisible, Prime};
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
};
use crate::{
    draw::element::PathGroup,
    error::{AdicShapeError, AdicShapeResult},
};
use super::graph::{EuclideanEdge, EuclideanNode, EuclideanPetgraph};



pub (super) trait EuclideanCtorVisitor: Debug + Clone {

    fn grow_all_branches(
        &self, graph: &mut EuclideanPetgraph,
        root_idx: NodeIndex,
    ) -> AdicShapeResult<()> {

        let mut visiting = vec![root_idx];
        while !visiting.is_empty() {

            let new_visiting = visiting.into_iter()
                .map(|nx| self.grow_child_stems(graph, nx))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .map(|(_eix, nix)| nix)
                .collect::<Vec<_>>();

            visiting = new_visiting;

        }

        Ok(())

    }

    fn grow_zero_valuation_stems(
        &self, graph: &mut EuclideanPetgraph,
    ) -> AdicShapeResult<()> {

        for nx in graph.node_indices().filter(|nx| graph[*nx].depth == -1).collect::<Vec<_>>() {
            self.grow_child_stems(graph, nx)?;
        }
        Ok(())

    }

    fn grow_full_branch(
        &self, graph: &mut EuclideanPetgraph,
        root_idx: NodeIndex,
        choices: &Vec<u32>,
    ) -> AdicShapeResult<Vec<(EdgeIndex, NodeIndex)>> {
        let mut nix = root_idx;
        let mut branch = vec![];
        for &choice in choices {
            let next = graph.edges(nix).find(|e| e.weight().branch_choice == choice).map(|e| (e.id(), e.target()));
            let next = next.map_or_else(|| self.grow_new_stem(graph, nix, choice), Ok)?;
            branch.push(next);
            nix = next.1;
        }
        Ok(branch)
    }

    fn grow_child_stems(
        &self, graph: &mut EuclideanPetgraph,
        prev_node: NodeIndex,
    ) -> AdicShapeResult<Vec<(EdgeIndex, NodeIndex)>> {

        let node = graph[prev_node].clone();
        let mut new_visiting = vec![];

        // If at or past depth, do not append to graph
        if node.depth >= self.max_depth() {
            return Ok(vec![]);
        }

        for choice in self.all_stem_choices() {
            new_visiting.push(self.grow_new_stem(graph, prev_node, choice)?);
        }

        Ok(new_visiting)

    }

    fn grow_new_stem(
        &self, graph: &mut EuclideanPetgraph,
        prev_node: NodeIndex,
        choice: u32,
    ) -> AdicShapeResult<(EdgeIndex, NodeIndex)>;

    fn all_stem_choices(&self) -> impl Iterator<Item=u32>;

    fn max_depth(&self) -> isize;

    fn scaling_multiplier(&self) -> f64;

}

#[derive(Debug, Clone)]
pub (super) struct ScaledHullsVisitor {
    vec_digits: Vec<(f64, f64)>,
    scaling: f64,
    depth: isize,
}

impl ScaledHullsVisitor {
    pub fn new(vec_digits: Vec<(f64, f64)>, scaling: f64, depth: isize) -> Self {
        ScaledHullsVisitor { vec_digits, scaling, depth }
    }
}

impl EuclideanCtorVisitor for ScaledHullsVisitor {

    fn grow_new_stem(
        &self, graph: &mut EuclideanPetgraph,
        prev_node: NodeIndex,
        choice: u32,
    ) -> AdicShapeResult<(EdgeIndex, NodeIndex)> {

        let node = graph[prev_node].clone();
        let choice_size = usize::try_from(choice)?;
        let position = self.vec_digits.get(choice_size).ok_or(AdicShapeError::PetGraph)?;
        let new_scaling = node.scaling * self.scaling;

        let mut new_choices = node.choices.clone();
        new_choices.push(choice);

        let new_node = graph.add_node(EuclideanNode{
            x: node.x + position.0 / new_scaling,
            y: node.y + position.1 / new_scaling,
            depth: node.depth + 1,
            scaling: new_scaling,
            choices: new_choices,
        });
        let new_edge = graph.add_edge(prev_node, new_node, EuclideanEdge {
            branch_choice: choice,
            // Path group will be set in paint
            path_group: PathGroup::default(),
        });
        Ok((new_edge, new_node))

    }

    fn all_stem_choices(&self) -> impl Iterator<Item=u32> {
        let len = self.vec_digits.len().try_into().expect("usize -> u32 conversion");
        0..len
    }
    fn max_depth(&self) -> isize { self.depth }
    fn scaling_multiplier(&self) -> f64 { self.scaling }

}



#[derive(Debug, Clone)]
pub (super) struct CharacteristicPAdicVisitor {
    p: Prime,
    scaling: f64,
    depth: isize,
}

impl CharacteristicPAdicVisitor {
    pub fn new(p: Prime, scaling: f64, depth: isize) -> Self {
        CharacteristicPAdicVisitor { p, scaling, depth }
    }
}

impl EuclideanCtorVisitor for CharacteristicPAdicVisitor {

    fn grow_new_stem(
        &self, graph: &mut EuclideanPetgraph,
        prev_node: NodeIndex,
        choice: u32,
    ) -> AdicShapeResult<(EdgeIndex, NodeIndex)> {

        let node = graph[prev_node].clone();
        let new_scaling = node.scaling * self.scaling;
        let fp = f64::from(u32::from(self.p));

        let mut new_choices = node.choices.clone();
        new_choices.push(choice);

        // Calculate the angle with characteristic function `exp((2 pi i) / p^(n+1) * x)`
        // Calculate with: x_0 -> /p -> + x_1 -> /p -> ...
        // This should avoid large integers
        let angle = new_choices.iter().fold(0.0, |acc, d| acc / fp + f64::from(*d)) / fp;
        let angle = TAU * angle;

        let new_node = graph.add_node(EuclideanNode{
            x: node.x + f64::cos(angle) * (self.scaling - 1.0) / new_scaling,
            y: node.y + f64::sin(angle) * (self.scaling - 1.0) / new_scaling,
            depth: node.depth + 1,
            scaling: new_scaling,
            choices: new_choices,
        });
        let new_edge = graph.add_edge(prev_node, new_node, EuclideanEdge {
            branch_choice: choice,
            // Path group will be set in paint
            path_group: PathGroup::default(),
        });
        Ok((new_edge, new_node))

    }

    fn all_stem_choices(&self) -> impl Iterator<Item=u32> {
        self.p.digit_range()
    }
    fn max_depth(&self) -> isize { self.depth }
    fn scaling_multiplier(&self) -> f64 { self.scaling }

}
