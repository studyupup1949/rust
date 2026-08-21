use crate::Point;

pub struct Line {
    pub from: Point,
    pub to: Point,
}

impl Line {
    pub fn new<S: Into<Point>, E: Into<Point>>(from: S, to: E) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

pub struct Rect {
    pub position: Point,
    pub dimensions: Point,
}

impl Rect {
    pub fn new<S: Into<Point>, E: Into<Point>>(position: S, dimensions: E) -> Self {
        Self {
            position: position.into(),
            dimensions: dimensions.into(),
        }
    }

    pub fn top_left(&self) -> Point {
        self.position
    }

    pub fn bottom_right(&self) -> Point {
        self.position + self.dimensions
    }
}

pub enum Primitive {
    Line(Line),
    Rect(Rect),
}

impl From<Line> for Primitive {
    fn from(l: Line) -> Self {
        Primitive::Line(l)
    }
}

impl From<Rect> for Primitive {
    fn from(r: Rect) -> Self {
        Primitive::Rect(r)
    }
}
