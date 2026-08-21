/// add one
pub fn add_one(source: isize) -> isize {
    source + 1
}

/// add two
pub fn add_two(source: isize) -> isize {
    source + 2
}

#[cfg(test)]
mod tests_mod {
    use super::*;

    #[test]
    fn it_works() {
        let result = add_one(2);
        assert_eq!(result, 3);
    }

    #[test]
    fn it_works2() {
        let result = add_two(2);
        assert_eq!(result, 4);
    }
}
