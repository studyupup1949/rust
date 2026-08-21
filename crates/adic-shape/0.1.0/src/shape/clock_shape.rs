use std::{
    f64::consts::TAU,
    iter::repeat,
};
use adic::AdicInteger;

use crate::{AdicShapeError, DisplayShape};
use super::element::{
    AdicEl,
    PathDInstruction,
    CircleEl, PathEl, TextEl,
};


type Coordinate = (f64, f64);

#[derive(Debug, Clone)]
/// Clock shape for drawing
///
/// ```
/// # use adic::radic;
/// # use adic_shape::{ClockShape, ClockShapeOptions};
/// let a = radic!(5, [1, 2, 3, 4], [0, 3]);
/// let depth = 20;
/// let shape_options = ClockShapeOptions{
///     show_hand_radii: false,
///     ..Default::default()
/// };
/// let clock_shape = ClockShape::new(&a, depth, shape_options);
/// ```
#[doc = ""]
#[doc = "<style>"]
#[doc = include_str!("../../img/rustdoc.css")]
#[doc = "</style>"]
#[doc = ""]
#[doc = include_str!("../../img/clock-shape-example.svg")]
#[doc = ""]
pub struct ClockShape {
    /// Number of ticks on the clock
    p: u32,
    /// Options for creating `ClockShape`
    options: ClockShapeOptions,
    /// Clock position
    position: ClockPosition,
    /// Clock hand information
    clock_hands: Vec<ClockHand>,
}


#[derive(Debug, Clone, Copy)]
/// Options for creating tree shape
pub struct ClockShapeOptions {
    /// Ticking or sweeping clock hands
    pub clock_movement: ClockMovement,
    /// Enable to show dotted lines at the radius of the clock hand heads
    pub show_hand_radii: bool,
    /// Width of the clock window
    pub viewbox_width: u32,
    /// Height of the clock window
    pub viewbox_height: u32,
}

impl Default for ClockShapeOptions {
    fn default() -> Self {
        ClockShapeOptions {
            clock_movement: ClockMovement::Sweeping,
            show_hand_radii: true,
            viewbox_width: 100,
            viewbox_height: 100,
        }
    }
}


#[derive(Debug, Clone, Copy)]
/// Clock layout information
pub struct ClockPosition {
    /// Position of center of the clock
    pub center: Coordinate,
    /// Radius of clock face
    pub radius: f64,
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
    ///
    /// # Errors
    /// Error if integer conversion fails
    pub fn new(
        adic_data: &impl AdicInteger,
        num_digits: usize,
        options: ClockShapeOptions,
    ) -> Result<Self, AdicShapeError> {

        let p = adic_data.p();
        let mut hands = Vec::with_capacity(num_digits);
        let mut offset = 0.0;
        for (i, d) in adic_data.digits().copied().chain(repeat(0)).take(num_digits).enumerate() {
            hands.push(ClockHand{
                order: i.try_into()?,
                tick: d,
                offset,
            });
            offset = match options.clock_movement {
                ClockMovement::Ticking => 0.0,
                ClockMovement::Sweeping => {
                    (f64::from(d) + offset) / f64::from(p)
                }
            };
        }

        let width = f64::from(options.viewbox_width);
        let height = f64::from(options.viewbox_height);
        Ok(Self {
            p,
            options,
            position: ClockPosition {
                center: (0.5*width, 0.5*height),
                radius: 0.4*width,
            },
            clock_hands: hands,
        })

    }

    /// Number of ticks on the clock
    pub fn p(&self) -> u32 {
        self.p
    }

    /// Centerpoint for the clock
    pub fn center(&self) -> (f64, f64) {
        self.position.center
    }

    /// Clock hands
    pub fn hands(&self) -> &Vec<ClockHand> {
        &self.clock_hands
    }

    /// Number of clock hands in the shape
    pub fn num_hands(&self) -> usize {
        self.hands().len()
    }

    /// Calculate positions of the heads of the clock hands
    pub fn hand_positions(&self) -> Vec<ClockHandPosition> {
        self.clock_hands.iter().map(|hand| self.calc_hand_position(hand)).collect()
    }

    /// Position of the tick along the radius of the clock face
    pub fn tick_positions(&self) -> Vec<Coordinate> {
        (0..self.p).map(|tick| {
            let tick_unit_coord = unit_coord(self.p, f64::from(tick));
            let x = self.position.center.0 + self.position.radius * tick_unit_coord.0;
            let y = self.position.center.1 + self.position.radius * tick_unit_coord.1;
            (x, y)
        }).collect()
    }

    fn calc_hand_position(
        &self,
        hand: &ClockHand,
    ) -> ClockHandPosition {

        // TODO: Move sophisticated clock head radius strategy of some sort
        let frac_radius = f64::from(hand.order + 1) / f64::from(u32::try_from(self.num_hands() + 1).unwrap());
        let radius = frac_radius * self.position.radius;

        let tick_amount = f64::from(hand.tick) + hand.offset;
        let unit = unit_coord(self.p, tick_amount);
        let x = self.position.center.0 + radius * unit.0;
        let y = self.position.center.1 + radius * unit.1;

        ClockHandPosition {
            radius,
            head_position: (x, y),
        }

    }

    fn clock_face_instructions(&self) -> impl Iterator<Item=AdicEl> {

        let (cx, cy) = self.position.center;
        let face_radius = self.position.radius;

        let mut face_views = vec![
            AdicEl::Circle(CircleEl{
                class: Some("clock-circle".to_string()),
                cx, cy, r: face_radius,
            })
        ];

        if self.options.show_hand_radii {
            self.hand_positions().iter().map(|pos| pos.radius).for_each(|r| {
                face_views.push(
                    AdicEl::Circle(CircleEl{
                        class: Some("clock-sub-circle".to_string()),
                        cx, cy, r,
                    })
                );
            });
        }

        face_views.into_iter()

    }

    fn clock_hand_instructions(&self) -> impl Iterator<Item=AdicEl> {

        // let path = svg_el::Path::new()
        //     .set("fill", "none")
        //     .set("stroke", "black")
        //     .set("stroke-width", 3)
        //     .set("d", data);

        let (cx, cy) = self.position.center;
        self.hand_positions().into_iter().flat_map(move |hand_pos| {
            let target = hand_pos.head_position;
            let hand_data = vec![
                PathDInstruction::Move((cx, cy)),
                PathDInstruction::Line(target),
            ];
            [
                AdicEl::Path(PathEl{
                    class: Some("clock-hand-path".to_string()),
                    d: hand_data,
                }),
                AdicEl::Circle(CircleEl{
                    class: Some("clock-head-circle".to_string()),
                    cx: target.0, cy: target.1, r: 0.5,
                })
            ]
        })

    }

    fn clock_mark_instructions(&self) -> impl Iterator<Item=AdicEl> {

        let (cx, cy) = self.position.center;
        self.tick_positions().into_iter().map(move |tick_pos| {
            let source = (cx + (tick_pos.0 - cx) * 0.95, cy + (tick_pos.1 - cy) * 0.95);
            let target = tick_pos;
            let mark_instructions = vec![
                PathDInstruction::Move(source),
                PathDInstruction::Line(target),
            ];
            AdicEl::Path(PathEl{
                class: Some("clock-hand-path".to_string()),
                d: mark_instructions
            })
        })

    }

    fn clock_label_instructions(&self) -> impl Iterator<Item=AdicEl> {

        let label_font_size = 4.;
        // Font is in pt and non-font is not, so possibly include a magic number multiplier
        let magic_font_multiplier = 1.;
        let label_size = label_font_size * magic_font_multiplier;
        let label_style = format!("position: fixed; font-size: {label_size}pt;");

        let (cx, cy) = self.position.center;
        self.tick_positions().into_iter().enumerate().map(move |(tick, tick_pos)| {
            let adjusted = (
                (tick_pos.0 - cx) * 0.1,
                (tick_pos.1 - cy) * 0.1
            );
            // let adjusted = (0., 0.);
            AdicEl::Text(TextEl{
                content: tick.to_string(),
                class: Some("tick-label".to_string()),
                style: Some(label_style.clone()),
                x: tick_pos.0, y: tick_pos.1,
                dx: adjusted.0, dy: adjusted.1,
            })
        })

    }

}

impl DisplayShape for ClockShape {

    /// Internal SVG elements generated from this shape
    fn adic_els(&self) -> impl Iterator<Item=AdicEl> {

        // Draw the clock
        let clock_face_circle = self.clock_face_instructions();
        let clock_hand_paths = self.clock_hand_instructions();
        let clock_marks = self.clock_mark_instructions();
        let clock_labels = self.clock_label_instructions();

        clock_face_circle
            .chain(clock_hand_paths)
            .chain(clock_marks)
            .chain(clock_labels)

    }

    fn default_class(&self) -> String {
        "adic-clock".to_string()
    }

    fn viewbox_width(&self) -> u32 {
        self.options.viewbox_width
    }
    fn viewbox_height(&self) -> u32 {
        self.options.viewbox_height
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



fn unit_coord(p: u32, tick_amount: f64) -> Coordinate {
    let arc_fraction = tick_amount / f64::from(p);
    let x = (TAU * arc_fraction).sin();
    let y = - (TAU * arc_fraction).cos();
    (x, y)
}



#[cfg(test)]
mod test {
    use adic::uadic;
    use super::{ClockMovement, ClockShape, ClockShapeOptions};

    use super::super::test_util::assert_diff_lt;

    #[test]
    fn correct_numbers() {

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let shape_options = ClockShapeOptions::default();

        let clock_shape = ClockShape::new(&adic_data, 6, shape_options).unwrap();

        assert_eq!(5, clock_shape.p);
        assert_eq!(6, clock_shape.num_hands());

        let hands = clock_shape.clock_hands;
        assert_eq!(0, hands[0].order);
        assert_eq!(3, hands[0].tick);
        assert_eq!(1, hands[1].order);
        assert_eq!(2, hands[1].tick);
        assert_eq!(2, hands[2].order);
        assert_eq!(4, hands[2].tick);
        assert_eq!(3, hands[3].order);
        assert_eq!(1, hands[3].tick);
        assert_eq!(4, hands[4].order);
        assert_eq!(4, hands[4].tick);
        assert_eq!(5, hands[5].order);
        assert_eq!(1, hands[5].tick);

    }

    #[test]
    fn correct_bounds() {

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let shape_options = ClockShapeOptions {
            clock_movement: ClockMovement::Ticking,
            ..Default::default()
        };
        let clock_shape = ClockShape::new(&adic_data, 6, shape_options).unwrap();
        let hand_pos = clock_shape.hand_positions();
        assert!(0. <= hand_pos.iter().map(|hand| hand.head_position.0).min_by(f64::total_cmp).unwrap());
        assert!(100. >= hand_pos.iter().map(|hand| hand.head_position.0).max_by(f64::total_cmp).unwrap());
        assert!(0. <= hand_pos.iter().map(|hand| hand.head_position.1).min_by(f64::total_cmp).unwrap());
        assert!(100. >= hand_pos.iter().map(|hand| hand.head_position.1).max_by(f64::total_cmp).unwrap());
        assert!(0. <= hand_pos.iter().map(|hand| hand.radius).min_by(f64::total_cmp).unwrap());
        assert!(50. >= hand_pos.iter().map(|hand| hand.radius).max_by(f64::total_cmp).unwrap());

    }

    #[test]
    fn correct_offsets() {

        let adic_data = uadic!(5, [3, 2, 4, 1, 4, 1, 2]);
        let shape_options = ClockShapeOptions {
            clock_movement: ClockMovement::Ticking,
            ..Default::default()
        };
        let clock_shape = ClockShape::new(&adic_data, 6, shape_options).unwrap();

        for hand in clock_shape.clock_hands {
            assert_diff_lt!(0., hand.offset, 0.1);
        }

        let shape_options = ClockShapeOptions {
            clock_movement: ClockMovement::Sweeping,
            ..Default::default()
        };
        let clock_shape = ClockShape::new(&adic_data, 6, shape_options).unwrap();

        let hands = clock_shape.clock_hands;
        assert_diff_lt!(0., hands[0].offset, 0.1);
        assert_diff_lt!(0.6, hands[1].offset, 0.1);
        assert_diff_lt!(0.52, hands[2].offset, 0.1);
        assert_diff_lt!(0.90, hands[3].offset, 0.1);
        assert_diff_lt!(0.38, hands[4].offset, 0.1);
        assert_diff_lt!(0.88, hands[5].offset, 0.1);


    }

}
