pub fn abs<T>(a: T, b: T) -> T
where
    T: PartialOrd + std::ops::Sub<Output = T>,
{
    match a > b {
        true => a - b,
        false => b - a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_for_f64_1() {
        assert_ne!(abs(3.1f64, 2.5f64), 0.6f64);
        assert_eq!(abs(3.1f64, 2.5f64), 0.6000000000000001f64);
    }

    #[test]
    fn it_works_for_i32() {
        assert_eq!(abs(42i32, 69i32), 27i32);
    }

    #[test]
    fn it_works_for_u8() {
        assert_eq!(abs(42u8, 69u8), 27u8);
    }
}
