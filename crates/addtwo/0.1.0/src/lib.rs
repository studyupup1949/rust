#[cfg(test)]
mod tests {
    use crate::two;

    #[test]
    fn it_works() {
        let result = two(2);
        assert_eq!(result, 3);
    }
}

pub fn two(x :i32) -> i32{
    x+1
}