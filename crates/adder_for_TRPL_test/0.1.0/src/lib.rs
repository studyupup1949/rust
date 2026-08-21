//! # adder
//! 
//! This crate is a example of testing.

/// object mod contains common gematric object.
pub mod object;

/// dice_game mod contains necessary item for dice game.
pub mod dice_game;


pub use self::object::Rectangle;
pub use self::dice_game::Guess;



pub fn add(left: u64, right: u64) -> u64 {
    left + right
}


/// Adds two to the number given
/// 
/// # Example
/// 
/// ```
/// let arg = 5;
/// let answer = adder::add_two(arg);
/// 
/// assert_eq!(7, answer);
/// ```
pub fn add_two(a:i32) -> i32 {
    a + 2
}

pub fn greeting(name: &str) -> String {
    format!("Hello")
}

pub fn prints_and_return_10(a: i32) -> i32 {
    println!("I got the value {a}");
    10
}



#[cfg(test)]
mod tests {
    use crate::prints_and_return_10;

    #[test]
    fn it_works() {
        let value = prints_and_return_10(5);
        assert_eq!(10, value);
    }

    #[test]
    fn it_works_aswell() {
        let value = prints_and_return_10(7);
        assert_eq!(10, value);
    }

    #[test]
    #[ignore]
    fn it_fails() {
        let value = prints_and_return_10(5);
        assert_eq!(11, value);
    }

}


// struct Rectangle {
//     width: u32,
//     height: u32,
// }

// impl Rectangle {
//     fn can_hold(&self, other: &Rectangle) -> bool {
//         self.width > other.width && self.height > other.height
//     }
// }

// pub struct Guess {
//     value: i32,
// }

// impl Guess {
//     pub fn new(value: i32) -> Guess {
//         if value < 1 {
//             panic!("Guess value must be greater than or equal to 1, got {value}.");
//         } else if value > 100 {
//             panic!("Guess value must be less than or equal to 100, got {value}.");
//         }

//         Guess { value }
//     }
// }