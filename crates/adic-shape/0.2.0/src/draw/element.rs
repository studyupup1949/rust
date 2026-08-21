use itertools::Either;
use super::{
    shape::{Direction, Orientation},
    util::str_digits,
};


#[derive(Debug, Clone)]
/// Internal display-independent element so we can translate e.g. to raw svg or leptos component
pub enum AdicEl {
    /// Draw a circle element
    Circle(CircleEl),
    /// Draw a path element
    Path(PathEl),
    /// Draw a text element
    Text(TextEl),
}


#[derive(Debug, Clone)]
pub struct PathEl {
    pub class: Option<String>,
    pub d: Vec<PathDInstruction>,
}

#[derive(Debug, Clone, Copy)]
pub enum PathDInstruction {
    Move((f64, f64)),
    Line((f64, f64)),
}

impl From<PathDInstruction> for String {
    fn from(instruction: PathDInstruction) -> Self {
        match instruction {
            PathDInstruction::Move(m) => format!("M {} {}", str_digits(m.0, 5), str_digits(m.1, 5)),
            PathDInstruction::Line(l) => format!("L {} {}", str_digits(l.0, 5), str_digits(l.1, 5)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathGroup {
    /// Path color grouping
    pub color_group: PathColor,
    /// SVG path stroke
    pub stroke: PathStroke,
}

impl Default for PathGroup {
    fn default() -> Self {
        PathGroup { color_group: PathColor::Default, stroke: PathStroke::Solid }
    }
}

impl PartialOrd for PathGroup {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathGroup {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::{Equal, Less, Greater};
        match (self.color_group, other.color_group) {
            (PathColor::Default, PathColor::Combined) => Less,
            (PathColor::Combined, PathColor::Default) => Greater,
            (PathColor::Default, PathColor::Color(_)) => Less,
            (PathColor::Color(_), PathColor::Default) => Greater,
            (PathColor::Combined, PathColor::Color(_)) => Less,
            (PathColor::Color(_), PathColor::Combined) => Greater,
            (PathColor::Color(m), PathColor::Color(n)) => m.cmp(&n),
            _ => Equal
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathColor {
    /// Default tree path color
    Default,
    /// Color of multiple colored paths on a single branch
    Combined,
    /// Distinguished path, e.g. special colored paths, different per integer
    Color(u32),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PathStroke {
    #[default]
    NoStroke,
    Solid,
    Dashed,
}


#[derive(Debug, Clone)]
pub struct CircleEl {
    pub class: Option<String>,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}


#[derive(Debug, Clone)]
pub struct TextEl {
    pub content: String,
    pub class: Option<String>,
    pub style: Option<String>,
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
}



#[derive(Debug, Clone, Copy)]
pub (crate) enum Resize {
    NoChange,
    FitToWindow,
    FitAroundZero,
}

pub (crate) fn resize_elems_around_box(
    resize: Resize,
    width: f64, height: f64,
    direction: Direction, orientation: Orientation,
    elems: impl Iterator<Item=AdicEl>,
) -> impl Iterator<Item=AdicEl> {

    let mut elem_vec = elems.collect::<Vec<_>>();

    // Change direction and orientation, from the otherwise default of Right and CCW
    let or_fn = orientation_fn(direction, orientation);
    for elem in &mut elem_vec {
        transform_elem(elem, &or_fn);
    }

    match resize {
        Resize::NoChange => {

            // Just change so that (0, 0) is at the bottom-left
            let reanchor_fn = |x: f64, y: f64| (x, height + y);
            for elem in &mut elem_vec {
                transform_elem(elem, &reanchor_fn);
            }

        },
        Resize::FitToWindow => {

            // Calculate bounds of AdicEls
            let Some((min_x, max_x, min_y, max_y)) = elem_vec.iter().flat_map(|elem| match elem {
                AdicEl::Path(PathEl{ d, .. }) => Either::Left(d.iter().map(|pt| match pt {
                    PathDInstruction::Move((x, y)) => (*x, *x, *y, *y),
                    PathDInstruction::Line((x, y)) => (*x, *x, *y, *y),
                })),
                AdicEl::Circle(CircleEl { cx, cy, r, .. }) => Either::Right(Either::Left(std::iter::once((cx - r, cx + r, cy - r, cy + r)))),
                AdicEl::Text(_) => Either::Right(Either::Right(std::iter::empty())),
            }).reduce(|acc, (pt_min_x, pt_max_x, pt_min_y, pt_max_y)| (
                f64::min(acc.0, pt_min_x), f64::max(acc.1, pt_max_x),
                f64::min(acc.2, pt_min_y), f64::max(acc.3, pt_max_y),
            )) else {
                return elem_vec.into_iter();
            };

            // If the graph is zero-sized, return
            if min_x >= max_x && min_y >= max_y {
                return elem_vec.into_iter();
            }

            // Adjust all node x and y while preserving aspect ratio
            let (avg_x, avg_y) = (0.5 * (min_x + max_x), 0.5 * (min_y + max_y));
            let (new_avg_x, new_avg_y) = (0.5 * width, 0.5 * height);
            let mult = match ((max_x - min_x), (max_y - min_y)) {
                (x, y) if y <= 0.0 => width / x,
                (x, y) if x <= 0.0 => height / y,
                (x, y) => f64::min(width / x, height / y),
            };
            // Multiply to get a small padding around the elements
            let mult = 0.99 * mult;
            let recenter_fn = scaled_recentered_fn((avg_x, avg_y), mult, (new_avg_x, new_avg_y));
            let scale_fn = simple_rescaled(mult);
            for elem in &mut elem_vec {
                transform_elem(elem, &recenter_fn);
                scale_elem(elem, &scale_fn);
            }

        },
        Resize::FitAroundZero => {

            // Calculate bounds of AdicEls
            let (max_abs_x, max_abs_y) = elem_vec.iter().flat_map(|elem| match elem {
                AdicEl::Path(PathEl{ d, .. }) => Either::Left(d.iter().map(|pt| match pt {
                    PathDInstruction::Move((x, y)) => (x.abs(), y.abs()),
                    PathDInstruction::Line((x, y)) => (x.abs(), y.abs()),
                })),
                AdicEl::Circle(CircleEl { cx, cy, r, .. }) => Either::Right(Either::Left(std::iter::once(
                    (f64::max((cx - r).abs(), (cx + r).abs()), f64::max((cy - r).abs(), (cy + r).abs()))
                ))),
                AdicEl::Text(_) => Either::Right(Either::Right(std::iter::empty())),
            }).fold((0.0, 0.0), |acc, (pt_max_x, pt_max_y)| (
                f64::max(acc.0, pt_max_x),
                f64::max(acc.1, pt_max_y),
            ));

            // If the graph is zero-sized, return
            if max_abs_x <= 0.0 && max_abs_y <= 0.0 {
                return elem_vec.into_iter();
            }

            // Adjust all node x and y while preserving aspect ratio
            let (avg_x, avg_y) = (0.0, 0.0);
            let (new_avg_x, new_avg_y) = (0.5 * width, 0.5 * height);
            let mult = match (max_abs_x, max_abs_y) {
                (x, y) if y <= 0.0 => 0.5 * width / x,
                (x, y) if x <= 0.0 => 0.5 * height / y,
                (x, y) => f64::min(0.5 * width / x, 0.5 * height / y),
            };
            // Multiply to get a small padding around the elements
            let mult = 0.99 * mult;
            let recenter_fn = scaled_recentered_fn((avg_x, avg_y), mult, (new_avg_x, new_avg_y));
            let scale_fn = simple_rescaled(mult);
            for elem in &mut elem_vec {
                transform_elem(elem, &recenter_fn);
                scale_elem(elem, &scale_fn);
            }

        },
    }

    elem_vec.into_iter()

}


fn transform_elem(elem: &mut AdicEl, transform_fn: &impl Fn(f64, f64) -> (f64, f64)) {
    match elem {
        AdicEl::Path(path) => {
            for pt in &mut path.d {
                match pt {
                    PathDInstruction::Move(m) => {
                        (m.0, m.1) = transform_fn(m.0, m.1);
                    },
                    PathDInstruction::Line(l) => {
                        (l.0, l.1) = transform_fn(l.0, l.1);
                    },
                }
            }
        },
        AdicEl::Circle(c) => {
            (c.cx, c.cy) = transform_fn(c.cx, c.cy);
        },
        AdicEl::Text(t) => {
            (t.x, t.y) = transform_fn(t.x, t.y);
        },
    }
}

fn scale_elem(elem: &mut AdicEl, scale_fn: &impl Fn(f64) -> f64) {
    if let AdicEl::Circle(c) = elem {
        c.r = scale_fn(c.r);
    }
}

fn orientation_fn(direction: Direction, orientation: Orientation) -> impl Fn(f64, f64) -> (f64, f64) {
    match (direction, orientation) {
        (Direction::Right, Orientation::CCW) => |x: f64, y: f64| (x, -y),
        (Direction::Right, Orientation::CW) => |x: f64, y: f64| (x, y),
        (Direction::Up, Orientation::CCW) => |x: f64, y: f64| (-y, -x),
        (Direction::Up, Orientation::CW) => |x: f64, y: f64| (y, -x),
        (Direction::Left, Orientation::CCW) => |x: f64, y: f64| (-x, y),
        (Direction::Left, Orientation::CW) => |x: f64, y: f64| (-x, -y),
        (Direction::Down, Orientation::CCW) => |x: f64, y: f64| (y, x),
        (Direction::Down, Orientation::CW) => |x: f64, y: f64| (-y, x),
    }
}

fn scaled_recentered_fn(old_center: (f64, f64), scale: f64, new_center: (f64, f64)) -> impl Fn(f64, f64) -> (f64, f64) {
    move |x: f64, y: f64| (
        (x - old_center.0) * scale + new_center.0,
        (y - old_center.1) * scale + new_center.1,
    )
}

fn simple_rescaled(mult: f64) -> impl Fn(f64) -> f64 {
    move |s| mult * s
}
