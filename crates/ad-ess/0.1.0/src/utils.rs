use std::ops::{Add, Sub};

pub fn kl_divergence(p_1: &[f32], p_2: &Vec<f32>) -> f32 {
    p_1.iter().zip(p_2).fold(0.0, |total, (pi_1, pi_2)| {
        total + pi_1 * (pi_1 / pi_2).log2()
    })
}

pub fn entropy(p: &[f32]) -> f32 {
    p.iter().map(|pi| -pi * pi.log2()).sum()
}

pub fn information(p: &[f32]) -> Vec<f32> {
    p.iter().map(|pi| -pi.log2()).collect()
}

pub fn cumsum<T>(list: &[T]) -> Vec<T>
where
    T: Clone,
    T: From<u8>,
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    list.iter().fold(vec![T::from(0u8)], |mut acc, val| {
        acc.push(acc.last().unwrap() + val);
        acc
    })
}

pub fn differeniate<T>(list: &[T]) -> Vec<T>
where
    T: Copy,
    T: Sub<Output = T>,
{
    let mut result = Vec::with_capacity(list.len() - 1);
    for idx in 1..list.len() {
        result.push(list[idx] - list[idx - 1]);
    }
    result
}

pub fn distribution_from_weights(weights: &[usize], res_factor: f32) -> Vec<f32> {
    let exps: Vec<f32> = weights
        .iter()
        .map(|weight| (*weight as f32 / -res_factor).exp2())
        .collect();
    let exps_sum = exps.iter().sum::<f32>();
    let p_goal: Vec<f32> = exps.iter().map(|exp| exp / exps_sum).collect();

    p_goal
}
