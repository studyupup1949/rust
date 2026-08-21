use std::collections::HashSet;

use adic::{
    error::AdicError,
    traits::{CanTruncate, HasApproximateDigits},
};
use bon::bon;
use itertools::Itertools;

use crate::{
    draw::element::{PathColor, PathGroup, PathStroke},
    error::{AdicShapeError, AdicShapeResult},
    shape::{
        canvas_sealed,
        AdicCanvas, Direction,
    },
};
use super::{
    create::{create_split_tree_graph, create_tree_graph},
    graph::{TreeBranch, TreeGraph},
    shape::TreeShape,
};


#[derive(Debug, Clone)]
/// Canvas for drawing [`TreeShape`]
pub struct TreeCanvas {
    draw_full_tree: PathStroke,
    base: u32,
    depth: isize,
    direction: Direction,
    dangling_direction: Option<Direction>,
    show_val_levels: bool,
    twig_depth: u32,
    viewbox_width: u32,
    viewbox_height: u32,
}


#[bon]
impl TreeCanvas {

    #[builder]
    /// Start [`TreeCanvasBuilder`] to build a `TreeCanvas`
    pub fn new(
        #[builder(field)]
        /// Stroke to draw the full tree
        draw_full_tree: PathStroke,
        /// Number of branches upward for each branching
        base: u32,
        /// The depth/height of the tree
        depth: isize,
        #[builder(default = Direction::Up)]
        /// The orthogonal direction the tree is growing toward
        direction: Direction,
        #[builder(required, default = Some(Direction::Right))]
        /// The direction of the dangling root of the tree
        dangling_direction: Option<Direction>,
        #[builder(default = false)]
        /// Show valuation levels as dotted lines
        show_val_levels: bool,
        #[builder(default = 1)]
        /// The depth of twigs to draw off of the main branches
        twig_depth: u32,
        #[builder(default = 100)]
        /// Width of the tree window
        viewbox_width: u32,
        #[builder(default = 100)]
        /// Height of the tree window
        viewbox_height: u32,
    ) -> Self {

        Self {
            draw_full_tree,
            base,
            depth,
            direction,
            dangling_direction,
            show_val_levels,
            twig_depth,
            viewbox_width,
            viewbox_height,
        }

    }

}

impl TreeCanvas {

    /// Number of branches from each branch point
    pub fn base(&self) -> u32 {
        self.base
    }

}

impl<S: tree_canvas_builder::State> TreeCanvasBuilder<S> {

    /// Do not draw the full tree shape
    pub fn no_full_tree(mut self) -> Self {
        self.draw_full_tree = PathStroke::NoStroke;
        self
    }
    /// Draw the full tree shape with a solid line
    pub fn solid_full_tree(mut self) -> Self {
        self.draw_full_tree = PathStroke::Solid;
        self
    }
    /// Draw the full tree shape with a dashed line
    pub fn dashed_full_tree(mut self) -> Self {
        self.draw_full_tree = PathStroke::Dashed;
        self
    }

}


impl From<TreeCanvas> for AdicShapeResult<TreeCanvas> {
    fn from(value: TreeCanvas) -> Self {
        Ok(value)
    }
}


impl AdicCanvas for TreeCanvas {
    type Shape = TreeShape;
}

impl canvas_sealed::DrawSingleInteger for TreeCanvas {
    fn _draw_integer(
        &self,
        adic_integer: &(impl Clone + HasApproximateDigits<DigitIndex = usize>),
    ) -> AdicShapeResult<Self::Shape> {
        self.draw_integers([adic_integer])
    }
}

impl canvas_sealed::DrawIntegers for TreeCanvas {
    fn _draw_integers<'a, A>(
        &self,
        adic_integers: impl IntoIterator<Item=&'a A>,
    ) -> AdicShapeResult<Self::Shape>
    where A: Clone + HasApproximateDigits<DigitIndex = usize> + 'a {
        let adic_integers = adic_integers.into_iter().collect::<Vec<_>>();

        if let Some(a) = adic_integers.first() && u32::from(a.base()) != self.base() {
            Err(AdicShapeError::AdicError(AdicError::MixedCharacteristic))?;
        }

        let depth = usize::try_from(self.depth)?;
        let (mut graph, root_idx, dangling_idx) = match self.draw_full_tree {
            PathStroke::Solid | PathStroke::Dashed => create_tree_graph(self.base, depth, self.direction, self.dangling_direction, self.draw_full_tree)?,
            PathStroke::NoStroke => create_split_tree_graph(
                adic_integers.clone(), depth, self.direction, self.dangling_direction, self.twig_depth,
            )?,
        };

        let width = f64::from(self.viewbox_width);
        let height = f64::from(self.viewbox_height);
        adjust_size(&mut graph, self.direction, width, height);

        let colored_branches = adic_integers.into_iter().map(|adic| {

            if adic.certainty().finite().is_some_and(|c| c < depth) {
                return Err(AdicShapeError::AdicError(
                    AdicError::InappropriatePrecision("Integer is not precise enough to draw on tree".to_string())
                ));
            }

            Ok(TreeBranch::adic_num_on_tree_graph(&graph, adic, depth, root_idx, dangling_idx))

        }).collect::<Result<Vec<_>, _>>()?;

        color_graph_branches(&mut graph, &colored_branches)?;

        Ok(TreeShape::new(
            self.base,
            graph,
            colored_branches,
            0,
            self.depth,
            self.direction,
            self.dangling_direction,
            self.show_val_levels,
            false,
            self.viewbox_width,
            self.viewbox_height,
        ))

    }
}

impl canvas_sealed::DrawSingleNumber for TreeCanvas {
    fn _draw_number(
        &self,
        adic_number: &(impl Clone + HasApproximateDigits<DigitIndex = isize> + CanTruncate),
    ) -> AdicShapeResult<Self::Shape> {
        self.draw_numbers([adic_number])
    }
}

impl canvas_sealed::DrawNumbers for TreeCanvas {
    fn _draw_numbers<'a, A>(
        &self,
        adic_numbers: impl IntoIterator<Item=&'a A>,
    ) -> AdicShapeResult<Self::Shape>
    where A: Clone + HasApproximateDigits<DigitIndex = isize> + CanTruncate + 'a {
        let adic_numbers = adic_numbers.into_iter().collect::<Vec<_>>();

        if let Some(a) = adic_numbers.first() && u32::from(a.base()) != self.base() {
            Err(AdicShapeError::AdicError(AdicError::MixedCharacteristic))?;
        }

        let min_valuation = adic_numbers.iter()
            .map(|a| a.min_index())
            .min().and_then(|v| v.finite())
            .map_or(0, |v| if v < 0 { v } else { 0 });
        let max_valuation = self.depth;
        let num_levels = (max_valuation - min_valuation).try_into()?;
        let (mut graph, root_idx, dangling_idx) = match self.draw_full_tree {
            PathStroke::Solid | PathStroke::Dashed => create_tree_graph(self.base, num_levels, self.direction, self.dangling_direction, self.draw_full_tree)?,
            PathStroke::NoStroke => create_split_tree_graph(
                adic_numbers.clone(), num_levels, self.direction, self.dangling_direction, self.twig_depth,
            )?,
        };

        let width = f64::from(self.viewbox_width);
        let height = f64::from(self.viewbox_height);
        adjust_size(&mut graph, self.direction, width, height);

        let colored_branches = adic_numbers.into_iter().map(|adic| {

            if adic.certainty().finite().is_some_and(|c| c < max_valuation) {
                return Err(AdicShapeError::AdicError(
                    AdicError::InappropriatePrecision("Integer is not precise enough to draw on tree".to_string())
                ));
            }

            // "Multiply" by p^(min_valuation) to get an integer of choices, prepended with 0 as necessary
            let adic_int = adic.split(min_valuation).1;

            Ok(TreeBranch::adic_num_on_tree_graph(&graph, &adic_int, num_levels, root_idx, dangling_idx))

        }).collect::<Result<Vec<_>, _>>()?;

        color_graph_branches(&mut graph, &colored_branches)?;

        Ok(TreeShape::new(
            self.base,
            graph,
            colored_branches,
            min_valuation,
            max_valuation,
            self.direction,
            self.dangling_direction,
            self.show_val_levels,
            true,
            self.viewbox_width,
            self.viewbox_height,
        ))

    }
}

impl canvas_sealed::DrawFullSpace for TreeCanvas {
    fn _draw_full(
        &self
    ) -> AdicShapeResult<TreeShape> {

        let depth = usize::try_from(self.depth)?;
        let (mut graph, _, _) = create_tree_graph(self.base, depth, self.direction, self.dangling_direction, self.draw_full_tree)?;

        let width = f64::from(self.viewbox_width);
        let height = f64::from(self.viewbox_height);
        adjust_size(&mut graph, self.direction, width, height);

        Ok(TreeShape::new(
            self.base,
            graph,
            vec![],
            0,
            self.depth,
            self.direction,
            self.dangling_direction,
            self.show_val_levels,
            false,
            self.viewbox_width,
            self.viewbox_height,
        ))

    }
}



/// Adjust x & y for width and height
fn adjust_size(graph: &mut TreeGraph, direction: Direction, width: f64, height: f64) {
    match direction {
        Direction::Up | Direction::Down => {
            for n in graph.node_weights_mut() {
                n.x = width * n.x;
                n.y = height * n.y;
            }
        },
        Direction::Left | Direction::Right => {
            for n in graph.node_weights_mut() {
                n.x = height * n.x;
                n.y = width * n.y;
            }
        },
    }
}

fn color_graph_branches(petgraph: &mut TreeGraph, colored_branches: &[TreeBranch]) -> AdicShapeResult<()> {

    // Calculate shared edges to graph with combined color
    let mut shared_edges = HashSet::new();
    let mut edge_iter = colored_branches.iter().flat_map(|b| b.0.iter()).sorted().peekable();
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
        let edge = petgraph.edge_weight_mut(*e_idx).ok_or(AdicShapeError::PetGraph)?;
        edge.path_group = path_group;
    }

    // Recolor unique colored edges
    for (idx, branch) in colored_branches.iter().enumerate() {

        let color_idx = (u32::try_from(idx)? % MAX_BRANCH_COLORS);
        let path_group = PathGroup{ color_group: PathColor::Color(color_idx), ..Default::default() };

        let unique_edges = branch.0.iter().copied().collect::<HashSet<_>>();
        let unique_edges = unique_edges.difference(&shared_edges);
        for e_idx in unique_edges {
            let edge = petgraph.edge_weight_mut(*e_idx).ok_or(AdicShapeError::PetGraph)?;
            edge.path_group = path_group;
        }

    }

    Ok(())

}


// TODO: Take this out and switch to use palette crate
const MAX_BRANCH_COLORS: u32 = 6;
