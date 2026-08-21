use std::ops::Add;

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl From<(usize, usize)> for Point {
    fn from(p: (usize, usize)) -> Self {
        Self { x: p.0, y: p.1 }
    }
}
