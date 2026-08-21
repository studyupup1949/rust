//! # Adder
//!
//! `adder` is a collection of utilities

/// Adds one to the number given.
///
/// # Examples //这里的#是markdown语法
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