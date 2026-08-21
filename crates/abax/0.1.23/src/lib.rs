#![allow(clippy::excessive_precision)]
#![allow(clippy::doc_overindented_list_items)]

//! # abax
//!
//! A lightweight Rust library providing high-precision mathematical constants and special functions.
//! This library includes Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>), Riemann Zeta values (<math><mi>ζ</mi><mo>(</mo><mi>s</mi><mo>)</mo></math>), and Stirling series coefficients.

mod consts;
mod digamma;
mod erf;
mod erfc;
mod erfcinv;
mod erfinv;
mod gamma;
mod gammainc;
mod gammaincinv;
mod gammaln;
mod psi;
mod tetragamma;
mod trigamma;
mod beta;
mod betaln;
mod betainc;
mod betaincinv;
mod norminv;
mod normcdf;
mod normpdf;
mod lognpdf;
mod logncdf;

pub use beta::beta;
pub use betaln::betaln;
pub use betainc::betainc;
pub use betaincinv::betaincinv;
pub use digamma::digamma;
pub use erf::erf;
pub use erfc::erfc;
pub use erfcinv::erfcinv;
pub use erfinv::erfinv;
pub use gamma::gamma;
pub use gammainc::gammainc;
pub use gammaincinv::gammaincinv;
pub use gammaln::gammaln;
pub use psi::psi;
pub use tetragamma::tetragamma;
pub use trigamma::trigamma;
pub use norminv::norminv;
pub use normcdf::normcdf;
pub use normpdf::normpdf;
pub use lognpdf::lognpdf;
pub use logncdf::logncdf;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gammaln_export() {
        assert_eq!(gammaln(1.0), 0.0);
    }

    #[test]
    fn test_erf_export() {
        assert!((erf(1.0) - 0.8427007929497149).abs() < 1e-15);
    }

    #[test]
    fn test_erfc_export() {
        assert!((erfc(1.0) - 0.15729920705028513).abs() < 1e-15);
    }

    #[test]
    fn test_erfinv_export() {
        assert!((erfinv(0.5) - 0.4769362762044699).abs() < 1e-15);
    }

    #[test]
    fn test_erfcinv_export() {
        assert!((erfcinv(0.5) - 0.4769362762044699).abs() < 1e-15);
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
