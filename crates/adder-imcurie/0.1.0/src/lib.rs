//! # My Crate
//!
//! `my_crate` is a collection of utilities to make performing certain
//! calculations more convenient.

pub use self::utils::add_two;

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
pub fn add_one(a: usize) -> usize {
    a + 1
}

pub mod utils {
    pub fn add_two(a: usize) -> usize {
        a + 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_adds_one() {
        let result = add_one(2);
        assert_eq!(result, 3);
    }
}
