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
mod betapdf;
mod betacdf;
mod betainv;
mod norminv;
mod normcdf;
mod normpdf;
mod lognpdf;
mod logncdf;
mod logninv;
mod tpdf;
mod tcdf;
mod tinv;
mod nctpdf;
mod nctcdf;
mod nctinv;
mod stirlerr;
mod binodeviance;
mod gampdf;
mod gamcdf;
mod gaminv;
mod dgammainc;

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
pub use betapdf::betapdf;
pub use betacdf::betacdf;
pub use betainv::betainv;
pub use norminv::norminv;
pub use normcdf::normcdf;
pub use normpdf::normpdf;
pub use lognpdf::lognpdf;
pub use logncdf::logncdf;
pub use logninv::logninv;
pub use tpdf::tpdf;
pub use tcdf::tcdf;
pub use tinv::tinv;
pub use nctpdf::nctpdf;
pub use nctcdf::nctcdf;
pub use nctinv::nctinv;
pub use gampdf::gampdf;
pub use gamcdf::gamcdf;
pub use gaminv::gaminv;


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

    #[test]
    fn test_logninv_export() {
        // Median of standard lognormal is 1.0
        assert!((logninv(0.5, 0.0, 1.0) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_tpdf_export() {
        assert!((tpdf(0.0, 1.0) - 1.0 / std::f64::consts::PI).abs() < 1e-15);
    }
}
