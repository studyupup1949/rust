//! # abax
//!
//! A lightweight Rust library providing high-precision mathematical constants and special functions.
//! This library includes Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>), Riemann Zeta values (<math><mi>ζ</mi><mo>(</mo><mi>s</mi><mo>)</mo></math>), and Stirling series coefficients.

mod consts;
mod gammaln;

pub use gammaln::gammaln;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gammaln_export() {
        assert_eq!(gammaln(1.0), 0.0);
    }
}
