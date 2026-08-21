//! Art
//! 
//! Library for modelling artistic concepts

pub use self::kinds::PrimaryColor;
pub use self::kinds::SecondaryColor;
pub use self::utils::mix;
pub mod kinds{

    /// The primary color according the RYB color model
    pub enum PrimaryColor {
        Red,
        Yellow,
        Blue
    }

    pub enum SecondaryColor {
        Orange,
        Green,
        Purple
    }
}

pub mod utils {
    use crate::kinds::*;

    ///Combines two primary colors in equal amount to create 
    /// a secondary color
    pub fn mix(c1: PrimaryColor, c2: PrimaryColor) -> SecondaryColor {
        // --snip--
        // ANCHOR_END: here   
        SecondaryColor::Orange
        //ANCHOR: here
    }
}