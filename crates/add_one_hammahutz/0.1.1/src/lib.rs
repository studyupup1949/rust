//! # My Crate
//!
//! `my_crate` is a collection of utilities to make performing certain
//! calculations more convenient.
/// Adds one to the number given.
///
/// # Examples
///
/// ## Use
/// ```
/// let arg = 5;
/// let answer = my_crate::add_one(arg);
///
/// assert_eq!(6, answer)
/// ```
/// ## Panic
///
/// ## Errors
///
/// ## Safety
pub fn add_one(number: i32) -> i32 {
    number + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add_one(3);
        assert_eq!(result, 4);
    }
}
