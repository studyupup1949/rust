use crate::{
    draw::element::AdicEl,
    error::AdicShapeResult,
    shape::{Direction, DisplayShape},
};
use super::{
    graph::{TreeBranch, TreeGraph},
    instruction,
};


#[derive(Debug, Clone)]
/// Tree shape, created with [`TreeCanvas`](super::TreeCanvas)
///
/// ```
/// # use adic::EAdic;
/// # use adic_shape::{shape::{AdicCanvas, Direction, TreeCanvas}, svg::SvgDisplay};
/// let a = EAdic::new_repeating(5, vec![1, 2, 3, 4], vec![0, 3]);
/// let depth = 9;
/// let tree_canvas = TreeCanvas::builder()
///     .base(5).depth(depth)
///     .direction(Direction::Up)
///     .twig_depth(2)
///     .build();
/// let tree_shape = tree_canvas.draw_integer(&a)?;
/// # let tree_string = tree_shape.create_svg_doc().to_string();
/// # let expected = std::fs::read_to_string("img/tree-shape-example.svg")?;
/// # assert_eq!(tree_string, expected);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[doc = ""]
#[doc = "<style>"]
#[doc = include_str!("../../../../img/rustdoc.css")]
#[doc = "</style>"]
#[doc = ""]
#[doc = include_str!("../../../../img/tree-shape-example.svg")]
#[doc = ""]
pub struct TreeShape {
    /// Number of branches upward for each branching
    base: u32,
    /// Stores the tree information
    petgraph: TreeGraph,
    #[allow(dead_code, reason="TODO: remove when we can more fully replace TreeShape with EuclideanShape")]
    /// Special branches colored differently, e.g. to indicate a path through the tree
    colored_branches: Vec<TreeBranch>,
    /// Minimum valuation
    min_valuation: isize,
    /// Maximum valuation
    max_valuation: isize,
    /// The orthogonal direction the tree is growing toward
    direction: Direction,
    /// The direction of the dangling root of the tree
    dangling_direction: Option<Direction>,
    /// Enable to show dotted lines at the branchings for each valuation
    show_val_levels: bool,
    /// Enable to show a level at valuation zero
    show_zero_val_level: bool,
    /// Width of the tree window
    viewbox_width: u32,
    /// Height of the tree window
    viewbox_height: u32,
}


impl TreeShape {

    /// Constructor
    #[allow(clippy::too_many_arguments)]
    pub (super) fn new(
        base: u32,
        petgraph: TreeGraph,
        colored_branches: Vec<TreeBranch>,
        min_valuation: isize,
        max_valuation: isize,
        direction: Direction,
        dangling_direction: Option<Direction>,
        show_val_levels: bool,
        show_zero_val_level: bool,
        viewbox_width: u32,
        viewbox_height: u32,
    ) -> Self {

        Self {
            base,
            petgraph,
            colored_branches,
            min_valuation,
            max_valuation,
            direction,
            dangling_direction,
            show_val_levels,
            show_zero_val_level,
            viewbox_width,
            viewbox_height,
        }

    }


    /// Number of branches from each branch point
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Petgraph structure for the tree
    pub (super) fn tree_graph(&self) -> &TreeGraph {
        &self.petgraph
    }

    #[cfg(test)]
    /// Special colored branches in the tree
    pub (super) fn colored_branches(&self) -> impl Iterator<Item=&TreeBranch> {
        self.colored_branches.iter()
    }

    pub (super) fn min_valuation(&self) -> isize {
        self.min_valuation
    }

    pub (super) fn max_valuation(&self) -> isize {
        self.max_valuation
    }

    pub (super) fn direction(&self) -> Direction {
        self.direction
    }

    pub (super) fn dangling_direction(&self) -> Option<Direction> {
        self.dangling_direction
    }

    pub (super) fn show_val_levels(&self) -> bool {
        self.show_val_levels
    }
    /// Enable to show a line at valuation zero
    pub (super) fn show_zero_val_level(&self) -> bool {
        self.show_zero_val_level
    }

    /// Index of the zero valuation branching
    pub (super) fn zero_valuation_idx(&self) -> Option<usize> {
        let min_valuation = self.min_valuation();
        let max_valuation = self.max_valuation();
        if min_valuation <= 0 && max_valuation >= 0 {
            Some(usize::try_from(-min_valuation).unwrap())
        } else {
            None
        }
    }

}

impl From<TreeShape> for AdicShapeResult<TreeShape> {
    fn from(value: TreeShape) -> Self {
        Ok(value)
    }
}

impl DisplayShape for TreeShape {

    /// Internal SVG elements generated from this shape
    fn adic_els(&self) -> impl Iterator<Item=AdicEl> {

        // Draw the tree
        let tree_paths = instruction::tree_paths(self).expect("Error converting tree paths to SVG");

        // Draw valuation levels
        let val_levels = instruction::valuation_levels(self).expect("Error converting valuation paths to SVG");

        // Draw the labels
        // let labels = labeller.labels(&tree_diagram, num_depth, num_branch, Default::default());

        // Wrap in svg
        tree_paths
            .chain(val_levels)
        //     .chain(labels)

    }

    fn default_class(&self) -> String {
        "adic-tree".to_string()
    }

    fn viewbox_width(&self) -> u32 {
        self.viewbox_width
    }
    fn viewbox_height(&self) -> u32 {
        self.viewbox_height
    }
}
