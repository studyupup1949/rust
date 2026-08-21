//! # Add one boring
//!
//! `add_one_boring` is a collection of utilities to make performing certain
//! calculations more convenient.

pub use self::sum::add_one_boring;

pub mod sum {
    /// Adds one to the number given.
    // --snip--

    /// Adds one to the number given.
    ///
    /// # Examples
    ///
    /// ```
    /// let arg = 5;
    /// let answer = add_one_boring::add_one_boring(arg);
    ///
    /// assert_eq!(6, answer);
    /// ```
    pub fn add_one_boring(x: i32) -> i32 {
        x + 1
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            assert_eq!(add_one_boring(4), 5);
        }
    }
}
