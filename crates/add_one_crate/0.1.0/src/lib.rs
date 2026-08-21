//! # My Crate
//!
//! `my_crate` is a collection of utilities to make performing certain
//! calculations more convenient.

/// Adds one to the number given.
///
/// # Examples
///
/// ```
/// let mut x = 5;
/// my_lib::add_one(&mut x);
///
/// assert_eq!(x, 6);
/// ```
pub fn add_one(x: &mut i32) {
    *x += 1;
}

#[cfg(test)]
mod tests {
    use crate::add_one;

    #[test]
    fn it_works() {
        let mut x = 2;
        add_one(&mut x);
        assert_eq!(x, 3);
    }
}
