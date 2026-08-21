//! Cardinal direction

use crate::error::AdicShapeError;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The four cardinal directions
pub enum Direction {
    /// Up direction
    Up,
    /// Down direction
    Down,
    /// Left direction
    Left,
    /// Right direction
    Right,
}

impl Direction {
    #[must_use]
    /// Opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
    #[must_use]
    /// Counterclockwise direction
    pub fn ccwise(&self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Right => Self::Up,
        }
    }
    #[must_use]
    /// Clockwise direction
    pub fn cwise(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Right => Self::Down,
        }
    }
}

// TODO: Add double-direction, single/double-direction, and multi-direction


impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Direction::Up => write!(f, "Up"),
            Direction::Down => write!(f, "Down"),
            Direction::Left => write!(f, "Left"),
            Direction::Right => write!(f, "Right"),
        }
    }
}

impl std::str::FromStr for Direction {
    type Err = AdicShapeError;
    fn from_str(s: &str) -> Result<Self, AdicShapeError> {
        match s {
            "Up" | "up" => Ok(Direction::Up),
            "Down" | "down" => Ok(Direction::Down),
            "Left" | "left" => Ok(Direction::Left),
            "Right" | "right" => Ok(Direction::Right),
            _ => Err(AdicShapeError::Parse("Direction parse error".to_string()))
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The two-dimensional orientations
pub enum Orientation {
    /// Clockwise orientation
    CW,
    /// Counterclockwise orientation
    CCW,
}

impl Orientation {
    #[must_use]
    /// Opposite orientation
    pub fn opposite(&self) -> Self {
        match self {
            Self::CW => Self::CCW,
            Self::CCW => Self::CW,
        }
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Orientation::CW => write!(f, "CW"),
            Orientation::CCW => write!(f, "CCW"),
        }
    }
}

impl std::str::FromStr for Orientation {
    type Err = AdicShapeError;
    fn from_str(s: &str) -> Result<Self, AdicShapeError> {
        match s {
            "CW" | "cw" => Ok(Orientation::CW),
            "CCW" | "ccw" => Ok(Orientation::CCW),
            _ => Err(AdicShapeError::Parse("Orientation parse error".to_string()))
        }
    }
}
