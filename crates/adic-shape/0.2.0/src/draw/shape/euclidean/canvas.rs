use adic::{
    divisible::Prime,
    error::AdicError,
    traits::HasApproximateDigits,
};
use bon::bon;
use num::ToPrimitive;

use crate::{
    draw::element::{PathStroke, Resize},
    error::{AdicShapeError, AdicShapeResult},
    shape::{
        canvas_sealed,
        AdicCanvas, Direction, Orientation,
    },
};
use super::{
    graph::EuclideanGraph,
    option::EuclideanStructure,
    shape::EuclideanShape,
    visitor::{CharacteristicPAdicVisitor, ScaledHullsVisitor}, 
};


type Coordinate = (f64, f64);

#[derive(Debug, Clone)]
/// Canvas for drawing [`EuclideanShape`]
///
/// Currently we have two main "structures" for euclideans:
///  [`fixed_hulls`](EuclideanCanvasBuilder::fixed_hulls) and [`characteristic_p_adic`](EuclideanCanvasBuilder::characteristic_p_adic).
/// There are many ways to map p-adic digits to euclidean space, but we focus on these two.
///
/// The `fixed_hulls` structure maps each digit to a fixed euclidean direction, scaled appropriately with each branching.
/// This creates very regular fractals, like the Sierpinsky gasket or Koch snowflake.
/// Its simplicity makes it quite flexible.
/// It is based on visualizations in [Robert's A Course in p-adic Analysis](https://link.springer.com/book/10.1007/978-1-4757-3254-2), pg. 12.
///
/// To use it, provide the euclidean vectors it should use for each digit.
///
/// ```no_run
/// # use adic_shape::shape::EuclideanCanvas;
/// let euclidean_canvas = EuclideanCanvas::builder()
///     .fixed_hulls(vec![(0.2, 0.0), (0.8, 0.0), (1.0, 0.6), (0.5, 1.0), (0.0, 0.6)])
///     .scaling(2.5).depth(4)
///     .draw_scaled_hulls()
///     .build();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// The `characteristic_p_adic` structure is our default.
/// This is the structure described at the top of the documentation: sweeping clocks attached recursively to clock heads like a tree.
/// It is more complicated because it preserves structure of the p-adics within the visualization.
/// The sweeping clocks encode the "carry" operation of addition gradually rather than all at once from `04._5` to `10._5`.
/// See [Chistyakov (1996)](https://arxiv.org/abs/math/0202089) for descriptions of this visualization.
///
/// The only parameter needed for this structure is the prime.
///
/// ```no_run
/// # use adic_shape::shape::EuclideanCanvas;
/// let euclidean_canvas = EuclideanCanvas::builder()
///     .characteristic_p_adic(5)
///     .scaling(2.5).depth(4)
///     .draw_scaled_hulls()
///     .build();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct EuclideanCanvas {
    draw_full_tree: PathStroke,
    draw_scaled_hulls: bool,
    draw_scaled_dots: bool,
    enclosing_disks: Vec<isize>,
    show_val_hulls: bool,
    structure: EuclideanStructure,
    resize: Resize,
    scaling: f64,
    depth: isize,
    viewbox_width: u32,
    viewbox_height: u32,
    direction: Direction,
    orientation: Orientation,
}


#[bon]
impl EuclideanCanvas {

    #[builder]
    /// Start [`EuclideanCanvasBuilder`] to build a `EuclideanCanvas`
    pub fn new(
        #[builder(field)]
        /// Stroke to draw the full tree
        draw_full_tree: PathStroke,
        #[builder(field = false)]
        /// Draw all the smallest convex hulls for the fractal
        draw_scaled_hulls: bool,
        #[builder(field = false)]
        /// Draw dots at the smallest points in the fractal
        draw_scaled_dots: bool,
        #[builder(field = vec![])]
        /// Draw enclosing disks around elements at the specified depths
        enclosing_disks: Vec<isize>,
        #[builder(setters(vis = "", name = structure_internal))]
        /// Euclidean structure, e.g. `scaled_hulls` or `characteristic_p_adic`
        structure: EuclideanStructure,
        #[builder(setters(vis = "", name = resize_internal), default = Resize::FitToWindow)]
        /// How to resize the structure, e.g. fitting to the window
        resize: Resize,
        /// Scaling factor, `Euclidean` divides each vector digit by a power of this factor as part of projection
        scaling: f64,
        /// Depth of the tree, rooted at -1, then `0 -> depth`
        depth: isize,
        #[builder(default = false)]
        /// Show convex hulls corresponding to all valuation levels
        show_val_hulls: bool,
        #[builder(default = 100)]
        /// Svg viewbox width
        viewbox_width: u32,
        #[builder(default = 100)]
        /// Svg viewbox width
        viewbox_height: u32,
        #[builder(default = Direction::Right)]
        /// Direction the `Euclidean` is pointing toward; the direction of the `0` element
        direction: Direction,
        #[builder(default = Orientation::CCW)]
        /// Orientation of the `Euclidean`; the orientation from the `0` element toward the `1` element
        orientation: Orientation,
    ) -> Self {

        Self {
            draw_full_tree,
            draw_scaled_hulls,
            draw_scaled_dots,
            enclosing_disks,
            show_val_hulls,
            structure,
            resize,
            scaling,
            depth,
            viewbox_width,
            viewbox_height,
            direction,
            orientation,
        }

    }

}

impl EuclideanCanvas {

    /// Number of branches from each branch point
    pub fn base(&self) -> u32 {
        match &self.structure {
            EuclideanStructure::ScaledHulls(v) => v.len().to_u32().expect("usize -> u32 conversion"),
            EuclideanStructure::CharacteristicPAdic(p) => p.into(),
        }
    }

}

impl<S: euclidean_canvas_builder::State> EuclideanCanvasBuilder<S> {

    /// Vector of coordinates corresponding to digits, of length `base`
    pub fn fixed_hulls(self, vec_digits: impl Into<Vec<Coordinate>>) -> EuclideanCanvasBuilder<euclidean_canvas_builder::SetStructure<S>>
    where S::Structure: euclidean_canvas_builder::IsUnset {
        self.structure_internal(EuclideanStructure::ScaledHulls(vec_digits.into()))
    }

    /// Calculate coordinates using a scaled set of Prufer group characteristic functions
    pub fn characteristic_p_adic(self, p: impl Into<Prime>) -> EuclideanCanvasBuilder<euclidean_canvas_builder::SetStructure<S>>
    where S::Structure: euclidean_canvas_builder::IsUnset {
        self.structure_internal(EuclideanStructure::CharacteristicPAdic(p.into()))
    }

    /// Do not resize Euclidean
    pub fn no_resize(self) -> EuclideanCanvasBuilder<euclidean_canvas_builder::SetResize<S>>
    where S::Resize: euclidean_canvas_builder::IsUnset {
        self.resize_internal(Resize::NoChange)
    }

    /// Resize to fit in the viewbox window
    pub fn resize_to_window(self) -> EuclideanCanvasBuilder<euclidean_canvas_builder::SetResize<S>>
    where S::Resize: euclidean_canvas_builder::IsUnset {
        self.resize_internal(Resize::FitToWindow)
    }

    /// Resize to fit in the viewbox window while fixing (0.0, 0.0) to the ctner of the viewbox
    pub fn resize_around_zero(self) -> EuclideanCanvasBuilder<euclidean_canvas_builder::SetResize<S>>
    where S::Resize: euclidean_canvas_builder::IsUnset {
        self.resize_internal(Resize::FitAroundZero)
    }

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

    /// Draw all the smallest convex hulls for the fractal
    pub fn draw_scaled_hulls(mut self) -> Self {
        self.draw_scaled_hulls = true;
        self
    }

    /// Draw dots at all the smallest elements for the fractal
    pub fn draw_scaled_dots(mut self) -> Self {
        self.draw_scaled_dots = true;
        self
    }

    /// Draw enclosing disks around the specified depths
    pub fn draw_enclosing_disks(mut self, depths: impl IntoIterator<Item=isize>) -> Self {
        self.enclosing_disks.extend(depths);
        self
    }

}


impl AdicCanvas for EuclideanCanvas {
    type Shape = EuclideanShape;
}

impl canvas_sealed::DrawSingleInteger for EuclideanCanvas {
    fn _draw_integer(
        &self,
        adic_integer: &(impl Clone + HasApproximateDigits<DigitIndex = usize>),
    ) -> AdicShapeResult<Self::Shape> {
        self.draw_integers([adic_integer])
    }
}

impl canvas_sealed::DrawIntegers for EuclideanCanvas {
    fn _draw_integers<'a, A>(
        &self,
        adic_integers: impl IntoIterator<Item=&'a A>,
    ) -> AdicShapeResult<Self::Shape>
    where A: HasApproximateDigits<DigitIndex = usize> + 'a {

        let adic_integers = adic_integers.into_iter().collect::<Vec<_>>();

        let depth = usize::try_from(self.depth)?;
        if adic_integers.iter().any(|a| u32::from(a.base()) != self.base()) {
            Err(AdicShapeError::AdicError(AdicError::MixedCharacteristic))?;
        }

        let colored_branches = adic_integers.into_iter().map(|integer| {

            if integer.certainty().finite().is_some_and(|c| c < depth) {
                return Err(AdicShapeError::AdicError(
                    AdicError::InappropriatePrecision("Integer is not precise enough to draw on euclidean".to_string())
                ));
            }

            let branch = integer
                .digits()
                .chain(std::iter::repeat(0))
                .take(depth)
                .collect::<Vec<_>>();
            Ok(branch)

        }).collect::<Result<Vec<_>, _>>()?;

        draw_branches(self, colored_branches, 0, self.depth, false)

    }
}

impl canvas_sealed::DrawSingleNumber for EuclideanCanvas {
    fn _draw_number(
        &self,
        adic_number: &(impl Clone + HasApproximateDigits<DigitIndex = isize> + adic::traits::CanTruncate),
    ) -> AdicShapeResult<Self::Shape> {
        self.draw_numbers([adic_number])
    }
}

impl canvas_sealed::DrawNumbers for EuclideanCanvas {
    fn _draw_numbers<'a, A>(
        &self,
        adic_numbers: impl IntoIterator<Item=&'a A>,
    ) -> AdicShapeResult<Self::Shape>
    where A: Clone + HasApproximateDigits<DigitIndex = isize> + adic::traits::CanTruncate + 'a {

        let adic_numbers = adic_numbers.into_iter().collect::<Vec<_>>();

        if adic_numbers.iter().any(|a| u32::from(a.base()) != self.base()) {
            Err(AdicShapeError::AdicError(AdicError::MixedCharacteristic))?;
        }

        let min_valuation = adic_numbers.iter()
            .map(|a| a.min_index())
            .min().and_then(|v| v.finite())
            .map_or(0, |v| if v < 0 { v } else { 0 });
        let max_valuation = self.depth;
        let num_levels = (max_valuation - min_valuation).try_into()?;

        let colored_branches = adic_numbers.into_iter().map(|number| {

            if number.certainty().finite().is_some_and(|c| c < max_valuation) {
                return Err(AdicShapeError::AdicError(
                    AdicError::InappropriatePrecision("Number is not precise enough to draw on euclidean".to_string())
                ));
            }

            let branch = number
                .digits()
                .chain(std::iter::repeat(0))
                .take(num_levels)
                .collect::<Vec<_>>();
            Ok(branch)

        }).collect::<Result<Vec<_>, _>>()?;

        draw_branches(self, colored_branches, min_valuation, max_valuation, true)

    }
}

impl canvas_sealed::DrawFullSpace for EuclideanCanvas {
    fn _draw_full(
        &self
    ) -> AdicShapeResult<EuclideanShape> {
        draw_branches(self, [], 0, self.depth, false)
    }
}



/// Draw branches with specified branching choices
fn draw_branches(
    canvas: &EuclideanCanvas,
    colored_branches: impl Into<Vec<Vec<u32>>>,
    min_valuation: isize,
    max_valuation: isize,
    show_zero_val: bool,
) -> AdicShapeResult<EuclideanShape> {
    let colored_branches = colored_branches.into();

    let graph = match &canvas.structure {
        EuclideanStructure::ScaledHulls(vec_digits) => {
            let visitor = ScaledHullsVisitor::new(vec_digits.clone(), canvas.scaling, canvas.depth);
            EuclideanGraph::build(
                &visitor,
                colored_branches,
                min_valuation,
                canvas.draw_full_tree,
                canvas.draw_scaled_hulls,
                canvas.draw_scaled_dots,
                canvas.enclosing_disks.clone(),
                show_zero_val,
            )?
        },
        EuclideanStructure::CharacteristicPAdic(p) => {
            let visitor = CharacteristicPAdicVisitor::new(*p, canvas.scaling, canvas.depth);
            EuclideanGraph::build(
                &visitor,
                colored_branches,
                min_valuation,
                canvas.draw_full_tree,
                canvas.draw_scaled_hulls,
                canvas.draw_scaled_dots,
                canvas.enclosing_disks.clone(),
                show_zero_val,
            )?
        },
    };

    let shape = EuclideanShape::new(
        graph,
        canvas.resize,
        min_valuation,
        max_valuation,
        canvas.show_val_hulls,
        show_zero_val,
        canvas.viewbox_width,
        canvas.viewbox_height,
        canvas.direction,
        canvas.orientation,
    );

    Ok(shape)

}
