//! # abax
//!
//! A lightweight Rust library providing high-precision mathematical constants and special functions.
//! This library includes Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>), Riemann Zeta values (<math><mi>ζ</mi><mo>(</mo><mi>s</mi><mo>)</mo></math>), and Stirling series coefficients.

mod consts;
mod digamma;
mod gamma;
mod gammainc;
mod gammaincinv;
mod gammaln;
mod psi;
mod tetragamma;
mod trigamma;

pub use digamma::digamma;
pub use gamma::gamma;
pub use gammainc::gammainc;
pub use gammaincinv::gammaincinv;
pub use gammaln::gammaln;
pub use psi::psi;
pub use tetragamma::tetragamma;
pub use trigamma::trigamma;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gammaln_export() {
        assert_eq!(gammaln(1.0), 0.0);
    }

    #[test]
    fn test_gamma_export() {
        assert_eq!(gamma(5.0), 24.0);
    }

    #[test]
    fn test_gammainc_export() {
        let p = gammainc(1.0, 1.0, true, false);
        assert!((p - (1.0 - (-1.0f64).exp())).abs() < 1e-14);
    }

    #[test]
    fn test_gammaincinv_export() {
        let x = gammaincinv(1.0 - (-1.0f64).exp(), 1.0, true);
        assert!((x - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_digamma_export() {
        assert!((digamma(1.0) + 0.5772156649015329).abs() < 1e-14);
    }

    #[test]
    fn test_trigamma_export() {
        assert!((trigamma(1.0) - 1.6449340668482264).abs() < 1e-14);
    }

    #[test]
    fn test_tetragamma_export() {
        assert!((tetragamma(1.0) + 2.4041138063191885).abs() < 1e-13);
    }
}
