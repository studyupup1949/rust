//! # Adder Crate
//!
//! `adder` is a collection of utilities to make performing certain
//! calculation more convenient.
pub use self::Kinds::PrimaryColor;
pub use self::Kinds::SecondaryColor;
pub use self::Utils::mix;

#[derive(Debug)]
pub struct Rectangle {
    width: u32,
    height: u32
}

impl Rectangle {
    pub fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

pub struct Guess {
    pub value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Guess {
        if value < 1 {
            panic!("Guess value must be greater than or equal 1, got {}.", value);
        } else if value > 100 {
            panic!("Guess value must be less than or equal 100, got {}.", value);
        }
        
        Guess {
            value
        }
    }
}
/// Adds one to the number given.
///
/// # Examples
///
/// ```
/// let arg = 5;
/// let answer = adder::add_one(arg);
///
/// assert_eq!(6, answer);
/// ```
pub fn add_one(a: i32) -> i32 {
    a + 1
}

pub fn add_two(a: i32) -> i32 {
    a + 2
}

pub mod Kinds {
    /// The primary colors according to the RYB color model
    pub enum PrimaryColor {
        Red,
        Yellow,
        Blue,
    }

    /// The secondary colors according to the RYB color model
    pub enum SecondaryColor {
        Orange,
        Green,
        Purple,
    }
}

pub mod Utils {
    use crate::Kinds::*;

    /// Combines two primary colors in equal amounts to create
    /// a secondary color.
    pub fn mix(c1: PrimaryColor, c2: PrimaryColor) -> SecondaryColor {
        println!("do something");
        SecondaryColor::Green
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn larger_can_hold_smaller() {
        let larger = Rectangle { width: 8, height: 7 };
        let smaller = Rectangle { width: 5, height: 1 };

        assert!(larger.can_hold(&smaller));
    }

    #[test]
    fn smaller_can_hold_larger() {
        let larger = Rectangle { width: 8, height: 7 };
        let smaller = Rectangle { width: 5, height: 1 };

        assert!(!smaller.can_hold(&larger));
    }

    #[test]
    fn it_adds_two() {
        assert_eq!(4, add_two(2));
    }

    #[test]
    #[should_panic(expected = "Guess value must be less than or equal 100")]
    fn greater_than_100() {
        Guess::new(200);
    }

    #[test]
    fn it_works() -> Result<(), String> {
        if 2 + 2 == 4 {
            Ok(())
        } else {
            Err(String::from("two plus two does not equal four"))
        }
    }
}
