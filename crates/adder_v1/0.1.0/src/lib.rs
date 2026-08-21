//! # My adder
//!
//! `adder` is a collection of utilities to make performing certain
//! calculations more convenient.

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
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
pub fn add_one(x: i32) -> i32 {
    x + 1
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn greeting(name: &str) -> String{
    format!("Hello {}!", name)
    // String::from("Hello")
}

pub struct Guess {
    value: i32,
}

pub fn add_two(value: i32) -> i32 { 
    2 + value
}

impl Guess {
    pub fn new(value: i32) -> Guess { 
        // if value < 1 || value > 100 {
        if value < 1 {
            // panic!("Guess value must be between 1 and 100, got {}", value);
            panic!(
                "Guess value must be greater than or equal to 1, got {}",
                value
            );
        } else if value > 100{
            panic!("Guess value must be less than or equal to 100, got {}", value);
        }

        Guess { value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exploration() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    // #[test]
    // fn another() {
    //     panic!("make this test fail");
    // }

    #[test]
    fn larger_can_hold_smaller() {
        let larger = Rectangle {
            width: 8,
            height: 7,
        };
        let smaller = Rectangle {
            width: 5,
            height: 1,
        };

        assert!(larger.can_hold(&smaller));
    }

    #[test]
    fn greeting_contains_name(){
        let result = greeting("Cargo");
        assert!(result.contains("Cargo"),
        "Greeting did not contain name, values was `{}`",
        result
    );
    }

    #[test]
    #[should_panic(expected = "less than or equal to 100")]
    fn greater_than_100(){
        Guess::new(200);
    }

    #[test]
    fn it_works() -> Result<(), String> {
        if 2 + 2 == 4{
            Ok(())
        }else {
            Err(String::from("two plus two does not equal four"))
        }
    }
}
