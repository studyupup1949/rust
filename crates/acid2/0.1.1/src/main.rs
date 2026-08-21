// use std::f64::consts::TAU;

// use acid2::F64;
// use rand::{thread_rng, Rng};
// use rand_distr::Standard;

// fn fourier(rng: &mut impl Rng, iterations: usize, x: F64, f: impl Fn(F64) -> f64) -> f64 {
//     rng.sample_iter(Standard)
//         .take(iterations)
//         .map(|y| f(y) * ((x * y).fract() * TAU).cos())
//         .sum::<f64>()
//         / (iterations as f64)
// }

fn main() {
    // println!(
    //     "{}",
    //     fourier(
    //         &mut thread_rng(),
    //         100_000_000,
    //         F64::ONE / F64::from(2),
    //         |x| (x.exponent() == 1) as u8 as f64 * x.abs()
    //     )
    // );
}
