//! # abhtest
//!
//! Experiment in crate publishing.

mod contents;

pub use self::contents::Arange;

#[cfg(test)]
mod tests {
    use super::contents::*;

    #[test]
    fn test_iter() {
        let n = 10;
        let arange = Arange::new(n);
        let mut i = 0;
        for _ in arange {
            i += 1;
        }
        assert_eq!(i, n);
    }

    #[test]
    fn test_empty() {
        let n = 0;
        let arange = Arange::new(n);
        let mut i = 0;
        for _ in arange {
            i += 1;
        }
        assert_eq!(i, n);
    }

    #[test]
    fn test_sum() {
        let n = 5;
        let arange = Arange::new(n);
        let s: i32 = arange.sum();
        assert_eq!(s, 10);
    }
}
