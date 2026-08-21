#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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
    /// Return opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
    #[must_use]
    /// Return counterclockwise direction
    pub fn ccwise(&self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Right => Self::Up,
        }
    }
    #[must_use]
    /// Return clockwise direction
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
