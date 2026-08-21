//! # ART
//!
//! A library for modeling

pub use self::kinds::PrimaryColor;
pub use self::kinds::SecondaryColor;
pub use self::utils::mix;


pub mod kinds{
    /// the primary colors
    pub enum PrimaryColor{
        Red,
        Yellow,
        Blue,
    }
    ///the secondary colors
    pub enum SecondaryColor{
        Orange,
        Green,
        Purple,
    }
}
pub mod utils{
    use crate::kinds::*;//使用kinds模块的所有枚举
    ///combine two primary color in equal amount to create
    /// a secondary color.
    pub fn mix(c1:PrimaryColor,c2:PrimaryColor)->SecondaryColor{
        unimplemented!();
    }

}
