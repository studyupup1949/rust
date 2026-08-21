//! Just testing out

/// # Examples
///
/// ```
/// use playground::add_two;
///
/// let res = add_two(5);
/// assert_eq!(res, 7);
/// ```
pub fn add_two(n: i32) -> i32 {
    add(n, 2)
}

fn add(n1: i32, n2: i32) -> i32 {
    n1 + n2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_two() {
        assert_eq!(add_two(5), 7);
    }

    #[test]
    fn test_add() {
        assert_eq!(add(5, 5), 10);
    }
}
