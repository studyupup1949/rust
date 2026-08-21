//! # Arc
//! 
//! A library for model arc

pub use self::kinds::PrimaColor;
pub use self::kinds::SecendColor;
pub use self::utils::mix;

pub mod kinds {
    /// The primary colors according to the RYB
    pub enum PrimaColor {
        Red,
        Yellow,
        Blue,
    }

    /// The secoend
    pub enum SecendColor {
        Orge,
        Green,
        Purple,
    }
}

pub mod utils {
    use crate::kinds::*;

    /// Mix func
    pub fn mix(c1: PrimaColor, c2: PrimaColor) -> SecendColor {
        SecendColor::Green
    }
}