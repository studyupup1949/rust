use crate::{
    draw::element::{AdicEl, Resize, resize_elems_around_box},
    error::AdicShapeResult,
    shape::{Direction, DisplayShape, Orientation},
};
use super::{
    graph::EuclideanGraph,
    instruction,
};


#[derive(Debug, Clone)]
/// Euclidean shape, created with [`EuclideanCanvas`](super::EuclideanCanvas)
///
/// <div class="warning">
/// These euclideans are explicitly two-dimensional, even if the concept applies to any dimension.
/// We stick to 2d so we can keep SVG-like output.
/// We hope to build support for higher dimensional data, with a projection at the end.
/// The `EuclideanShape` must be converted to two-dimensional at some point: projected onto the screen.
/// </div>
///
/// ```
/// # use adic::{traits::PrimedFrom, EAdic};
/// # use adic_shape::{shape::{AdicCanvas, EuclideanCanvas}, svg::SvgDisplay};
/// let adic_numbers = vec![EAdic::primed_from(3, 1), EAdic::primed_from(3, -1)];
/// let depth = 4;
/// let euclidean_canvas = EuclideanCanvas::builder()
///     .characteristic_p_adic(3)
///     .scaling(2.5).depth(depth)
///     .solid_full_tree()
///     .draw_scaled_hulls()
///     .build();
/// let euclidean_shape = euclidean_canvas.draw_integers(&adic_numbers)?;
/// # let euclidean_string = euclidean_shape.create_svg_doc().to_string();
/// # let expected = std::fs::read_to_string("img/euclidean-shape-example.svg")?;
/// # assert_eq!(euclidean_string, expected);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[doc = ""]
#[doc = "<style>"]
#[doc = include_str!("../../../../img/rustdoc.css")]
#[doc = "</style>"]
#[doc = ""]
#[doc = include_str!("../../../../img/euclidean-shape-example.svg")]
#[doc = ""]
pub struct EuclideanShape {
    /// Stores the Euclidean `petgraph` information
    graph: EuclideanGraph,
    /// How to resize graph around viewbox
    resize: Resize,
    /// Minimum valuation
    min_valuation: isize,
    /// Maximum valuation
    max_valuation: isize,
    /// Enable to show dotted line convex hulls at all valuations
    show_val_hulls: bool,
    /// Enable to show a convex hull at valuation zero
    show_zero_val_hull: bool,
    /// Width of the euclidean window
    viewbox_width: u32,
    /// Height of the euclidean window
    viewbox_height: u32,
    /// Direction the `Euclidean` is pointing toward, default `Right`
    direction: Direction,
    /// Orientation of the `Euclidean`, default CCW
    orientation: Orientation,
}


impl EuclideanShape {

    /// Constructor
    #[allow(clippy::too_many_arguments)]
    pub (super) fn new(
        graph: EuclideanGraph,
        resize: Resize,
        min_valuation: isize,
        max_valuation: isize,
        show_val_hulls: bool,
        show_zero_val_hull: bool,
        viewbox_width: u32,
        viewbox_height: u32,
        direction: Direction,
        orientation: Orientation,
    ) -> Self {

        Self {
            graph,
            resize,
            min_valuation,
            max_valuation,
            show_val_hulls,
            show_zero_val_hull,
            viewbox_width,
            viewbox_height,
            direction,
            orientation,
        }

    }

    pub (super) fn graph(&self) -> &EuclideanGraph {
        &self.graph
    }

    pub (super) fn min_valuation(&self) -> isize {
        self.min_valuation
    }

    pub (super) fn max_valuation(&self) -> isize {
        self.max_valuation
    }

    pub (super) fn show_val_hulls(&self) -> bool {
        self.show_val_hulls
    }
    pub (super) fn show_zero_val_hull(&self) -> bool {
        self.show_zero_val_hull
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


impl From<EuclideanShape> for AdicShapeResult<EuclideanShape> {
    fn from(value: EuclideanShape) -> Self {
        Ok(value)
    }
}

impl DisplayShape for EuclideanShape {

    /// Internal SVG elements generated from this shape
    fn adic_els(&self) -> impl Iterator<Item=AdicEl> {

        // Draw enclosing disks around the given depths
        let euclidean_enclosing_disks = instruction::enclosing_disks(&self.graph).expect("Error converting Euclidean enclosing disks to SVG");

        // Draw the convex hulls for valuation levels
        let euclidean_valuation_hulls = instruction::valuation_hulls(self).expect("Error converting Euclidean valuation hulls to SVG");

        // Draw the fully depth-scaled convex hulls
        let euclidean_scaled_hulls = instruction::scaled_hulls(&self.graph).expect("Error converting Euclidean scaled hulls to SVG");

        // Draw the tree connecting the graph
        let euclidean_tree = instruction::tree_paths(&self.graph).expect("Error converting Euclidean tree paths to SVG");

        // Draw dots at the fully depth-scaled elements
        let euclidean_scaled_dots = instruction::scaled_dots(&self.graph).expect("Error converting Euclidean scaled dots to SVG");

        let elems = euclidean_enclosing_disks
            .chain(euclidean_valuation_hulls)
            .chain(euclidean_scaled_hulls)
            .chain(euclidean_tree)
            .chain(euclidean_scaled_dots);

        resize_elems_around_box(
            self.resize,
            self.viewbox_width.into(),
            self.viewbox_height.into(),
            self.direction,
            self.orientation,
            elems
        )

    }

    fn default_class(&self) -> String {
        "adic-euclidean".to_string()
    }

    fn viewbox_width(&self) -> u32 {
        self.viewbox_width
    }
    fn viewbox_height(&self) -> u32 {
        self.viewbox_height
    }
}
