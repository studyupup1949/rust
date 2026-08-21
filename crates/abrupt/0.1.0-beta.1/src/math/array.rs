use std::{
    cmp::PartialOrd,
    ops::{Add, Mul},
};

#[inline]
pub fn sum<T>(arr: &[T]) -> T
where
    T: Add<Output = T> + Copy + Default,
{
    arr.iter().copied().fold(T::default(), Add::add)
}

#[inline]
pub fn product<T>(arr: &[T]) -> T
where
    T: Mul<Output = T> + Copy + Default + From<u8>,
{
    arr.iter().copied().fold(T::from(1u8), Mul::mul)
}

#[inline]
pub fn average<T>(arr: &[T]) -> f64
where
    T: Add<Output = T> + Copy + Into<f64>,
{
    let sum: f64 = arr.iter().copied().map(Into::into).sum();
    sum / arr.len() as f64
}

#[inline]
pub fn min<T>(arr: &[T]) -> T
where
    T: PartialOrd + Copy,
{
    *arr.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

#[inline]
pub fn max<T>(arr: &[T]) -> T
where
    T: PartialOrd + Copy,
{
    *arr.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}
