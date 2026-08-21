pub mod array;

use std::{
    cmp::PartialOrd,
    convert::From,
    ops::{Add, Div, Mul, Neg, Rem},
};

#[inline]
pub fn factorial<T>(n: T) -> T
where
    T: Mul<Output = T> + Add<Output = T> + From<u8> + Copy + PartialOrd,
{
    let mut result = T::from(1);
    let mut i = T::from(1);
    while i <= n {
        result = result * i;
        i = i + T::from(1);
    }
    result
}

#[inline]
pub fn gcd<T>(a: T, b: T) -> T
where
    T: Copy + PartialOrd + From<u8> + Rem<Output = T> + Eq,
{
    if b == T::from(0u8) {
        a
    } else {
        gcd(b, a % b)
    }
}

#[inline]
pub fn lcm<T>(a: T, b: T) -> T
where
    T: Copy + PartialOrd + From<u8> + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Eq,
{
    a * (b / gcd(a, b))
}

#[inline]
pub fn is_prime<T>(n: T) -> bool
where
    T: Copy + PartialOrd + From<u8> + TryInto<u64> + Rem<Output = T>,
{
    if n <= T::from(1u8) {
        return false;
    }

    let n_u64: u64 = match n.try_into() {
        Ok(num) => num,
        Err(_) => return false,
    };

    let upper = (n_u64 as f64).sqrt() as u64 + 1;
    for i in 2..upper {
        if n_u64 % i == 0 {
            return false;
        }
    }
    true
}
#[inline]
pub fn pow<T>(base: T, exp: u32) -> T
where
    T: Mul<Output = T> + Copy + From<u8>,
{
    (0..exp).fold(T::from(1u8), |acc, _| acc * base)
}

#[inline]
pub fn abs<T>(value: T) -> T
where
    T: PartialOrd + Neg<Output = T> + Copy + From<u8>,
{
    if value < T::from(0u8) {
        -value
    } else {
        value
    }
}

#[inline]
pub fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
