use std::f64::consts::TAU;

use crate::{
    draw::element::AdicEl,
    error::AdicShapeResult,
    shape::DisplayShape,
};
use super::instruction;


type Coordinate = (f64, f64);

#[derive(Debug, Clone)]
/// Clock shape, created with [`ClockCanvas`](super::ClockCanvas)
///
/// ```
/// # use adic::EAdic;
/// # use adic_shape::{shape::{AdicCanvas, ClockCanvas}, svg::SvgDisplay};
/// let a = EAdic::new_repeating(5, vec![1, 2, 3, 4], vec![0, 3]);
/// let depth = 10;
/// let canvas = ClockCanvas::builder().base(5).depth(depth).show_val_circles(false).build();
/// let clock_shape = canvas.draw_integer(&a)?;
/// # let clock_string = clock_shape.create_svg_doc().to_string();
/// # let expected = std::fs::read_to_string("img/clock-shape-example.svg")?;
/// # assert_eq!(clock_string, expected);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[doc = ""]
#[doc = "<style>"]
#[doc = include_str!("../../../../img/rustdoc.css")]
#[doc = "</style>"]
#[doc = ""]
#[doc = include_str!("../../../../img/clock-shape-example.svg")]
#[doc = ""]
pub struct ClockShape {
    /// Number of ticks on the clock
    base: u32,
    /// Clock hand information
    clock_hands: Vec<ClockHand>,
    /// Minimum valuation of a clock hand
    min_valuation: isize,
    /// Maximum valuation of a clock hand
    max_valuation: isize,
    /// Enable to label tick marks
    show_tick_labels: bool,
    /// Enable to show dotted lines at the radius of the clock hand heads
    show_val_circles: bool,
    /// Enable to show a circle at valuation zero
    show_zero_val_circle: bool,
    /// Width of the clock window
    viewbox_width: u32,
    /// Height of the clock window
    viewbox_height: u32,
}


#[derive(Debug, Clone, Copy)]
/// Nature of clock hand movement
pub enum ClockMovement {
    /// Clock hands "tick", only pointing exactly to the tick marks of the clock
    Ticking,
    /// Clock hands "sweep", accounting for the positions of lower hands
    Sweeping,
}


impl ClockShape {

    /// Constructor
    #[allow(clippy::too_many_arguments)]
    pub (super) fn new(
        base: u32,
        clock_hands: Vec<ClockHand>,
        min_valuation: isize,
        max_valuation: isize,
        show_tick_labels: bool,
        show_val_circles: bool,
        show_zero_val_circle: bool,
        viewbox_width: u32,
        viewbox_height: u32,
    ) -> Self {

        Self {
            base,
            clock_hands,
            min_valuation,
            max_valuation,
            show_tick_labels,
            show_val_circles,
            show_zero_val_circle,
            viewbox_width,
            viewbox_height,
        }

    }

    /// Number of ticks on the clock
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Enable to show dotted lines at the radius of the clock hand heads
    pub (super) fn show_val_circles(&self) -> bool {
        self.show_val_circles
    }
    /// Enable to label tick marks
    pub (super) fn show_tick_labels(&self) -> bool {
        self.show_tick_labels
    }
    /// Enable to show a circle at valuation zero
    pub (super) fn show_zero_val_circle(&self) -> bool {
        self.show_zero_val_circle
    }

    /// Centerpoint for the clock
    pub (super) fn center(&self) -> (f64, f64) {
        (0.5 * f64::from(self.viewbox_width), 0.5 * f64::from(self.viewbox_height))
    }
    /// Radius for the clock
    pub (super) fn radius(&self) -> f64 {
        0.4 * f64::from(self.viewbox_width)
    }
    /// Index of the zero valuation clock hand
    pub (super) fn zero_valuation_idx(&self) -> Option<usize> {
        let min_valuation = self.min_valuation;
        let max_valuation = self.max_valuation;
        if min_valuation <= 0 && max_valuation >= 0 {
            Some(usize::try_from(-min_valuation).unwrap())
        } else {
            None
        }
    }

    /// Clock hands
    pub (super) fn hands(&self) -> &Vec<ClockHand> {
        &self.clock_hands
    }

    /// Number of clock hands in the shape
    pub (super) fn num_hands(&self) -> usize {
        self.hands().len()
    }

    /// Calculate positions of the heads of the clock hands
    pub (super) fn hand_positions(&self) -> Vec<ClockHandPosition> {
        self.clock_hands.iter().map(|hand| self.calc_hand_position(hand)).collect()
    }

    /// Position of the tick along the radius of the clock face
    pub (super) fn tick_positions(&self) -> Vec<Coordinate> {
        (0..self.base).map(|tick| {
            let tick_unit_coord = unit_coord(self.base, f64::from(tick));
            let x = self.center().0 + self.radius() * tick_unit_coord.0;
            let y = self.center().1 + self.radius() * tick_unit_coord.1;
            (x, y)
        }).collect()
    }

    fn calc_hand_position(
        &self,
        hand: &ClockHand,
    ) -> ClockHandPosition {

        let radius = self.hand_radius(hand);

        let tick_amount = f64::from(hand.tick) + hand.offset;
        let unit = unit_coord(self.base, tick_amount);
        let x = self.center().0 + radius * unit.0;
        let y = self.center().1 + radius * unit.1;

        ClockHandPosition {
            radius,
            head_position: (x, y),
        }

    }
    fn hand_radius(&self, hand: &ClockHand) -> f64 {
        // TODO: Move sophisticated clock head radius strategy of some sort
        let frac_radius = f64::from(hand.order + 1) / f64::from(u32::try_from(self.num_hands() + 1).unwrap());
        frac_radius * self.radius()
    }

}

impl From<ClockShape> for AdicShapeResult<ClockShape> {
    fn from(value: ClockShape) -> Self {
        Ok(value)
    }
}

impl DisplayShape for ClockShape {

    /// Internal SVG elements generated from this shape
    fn adic_els(&self) -> impl Iterator<Item=AdicEl> {

        // Draw the clock
        let clock_face_circle = instruction::clock_face_instructions(self);
        let clock_hand_paths = instruction::clock_hand_instructions(self);
        let clock_marks = instruction::clock_mark_instructions(self);
        let clock_labels = instruction::clock_label_instructions(self);

        clock_face_circle
            .chain(clock_hand_paths)
            .chain(clock_marks)
            .chain(clock_labels)

    }

    fn default_class(&self) -> String {
        "adic-clock".to_string()
    }

    fn viewbox_width(&self) -> u32 {
        self.viewbox_width
    }
    fn viewbox_height(&self) -> u32 {
        self.viewbox_height
    }
}


#[derive(Debug, Clone)]
/// Data for each clock hand node
pub struct ClockHand {
    /// Clock hand order, e.g. second hand -> 0, minute hand -> 1
    pub order: i32,
    /// Tick the clock hand is indicating
    pub tick: u32,
    /// Offset from the tick mark (0 <= offset < 1)
    pub offset: f64,
}

#[derive(Debug, Clone)]
/// Data for each clock hand edge
pub struct ClockHandPosition {
    /// Radius from center to clock head
    pub radius: f64,
    /// Position of clock head
    pub head_position: Coordinate,
}


fn unit_coord(base: u32, tick_amount: f64) -> Coordinate {
    let arc_fraction = tick_amount / f64::from(base);
    let x = (TAU * arc_fraction).sin();
    let y = - (TAU * arc_fraction).cos();
    (x, y)
}
