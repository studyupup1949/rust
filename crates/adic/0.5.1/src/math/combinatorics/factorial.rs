use num::One;
use std::ops::{Add, Mul};

/// Factorial function
pub fn factorial<T>(n: u32) -> T
where T:One + Add<Output = T> + Mul<Output = T> + Clone{
    let mut t = T::one();
    let mut ans = t.clone();
    for _i in 2..= n{
        t = t + T::one();
        ans = ans * t.clone();
    }
    ans
}

#[cfg(test)]
mod test{
use std::str::FromStr;
use num::{BigUint};

    use super::factorial;

    #[test]
    fn factorial_u32(){
        let expected = [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800, 479001600];
        for i in 0..13{
            let t = factorial::<u32>(i);
            assert_eq!(t, expected[i as usize]);
        }
    }

    #[test]
    #[should_panic]
    fn factorial_u32_overflow(){
        factorial::<u32>(13);
    }

    #[test]
    fn factorial_big_int(){
        let t = factorial::<BigUint>(13);
        assert_eq!(t, BigUint::from_str("6227020800").unwrap());
        let t = factorial::<BigUint>(100);
        assert!(t.to_string().len() > 100);
    }

}
