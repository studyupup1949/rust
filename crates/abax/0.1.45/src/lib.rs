#![allow(clippy::excessive_precision)]
#![allow(clippy::doc_overindented_list_items)]

//! # abax
//! 
//! A high-performance Rust library for statistical computing and special mathematical functions.
//! `abax` provides numerically stable and high-precision implementations of probability 
//! distributions, error functions, and gamma-related functions.
//! 
//! ## Features
//! 
//! ### Probability Distributions
//! Comprehensive support for Probability Density Functions (PDF), Cumulative Distribution 
//! Functions (CDF), and Quantile Functions (Inverse CDF) for common distributions:
//! - **Continuous**: Normal, Lognormal, Student's T, Noncentral T, Gamma, Beta, Exponential, Chi-squared, Weibull, Extreme Value.
//! - **Discrete**: Binomial, Poisson.
//! 
//! ### Special Functions
//! - **Gamma Family**: Gamma, Log-Gamma, Digamma, Trigamma, Tetragamma, and Polygamma (<math><msup><mi>ψ</mi><mrow><mo>(</mo><mi>n</mi><mo>)</mo></mrow></msup></math>).
//! - **Incomplete Gamma**: Regularized incomplete gamma functions and their inverses.
//! - **Beta Family**: Beta, Log-Beta, and Regularized Incomplete Beta functions.
//! - **Error Functions**: Erf, Erfc, Erfcx, and their high-precision inverses.
//! 
//! ### Constants
//! Provides high-precision mathematical constants including:
//! - Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>)
//! - Riemann Zeta values (<math><mi>ζ</mi><mo>(</mo><mi>s</mi><mo>)</mo></math>)
//! - Stirling series coefficients

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
pub use betapdf::betapdf;
mod betacdf;
pub use betacdf::betacdf;
mod betainv;
pub use betainv::betainv;
mod binopdf;
pub use binopdf::binopdf;
mod binocdf;
pub use binocdf::binocdf;
mod binoinv;
pub use binoinv::binoinv;
mod chi2pdf;
pub use chi2pdf::chi2pdf;
mod chi2cdf;
pub use chi2cdf::chi2cdf;
mod chi2inv;
pub use chi2inv::chi2inv;
mod evpdf;
pub use evpdf::evpdf;
mod evcdf;
pub use evcdf::evcdf;
mod evinv;
pub use evinv::evinv;
mod exppdf;
pub use exppdf::exppdf;
mod expcdf;
pub use expcdf::expcdf;
mod expinv;
pub use expinv::expinv;
mod gampdf;
pub use gampdf::gampdf;
mod gamcdf;
pub use gamcdf::gamcdf;
mod gaminv;
pub use gaminv::gaminv;
mod normpdf;
pub use normpdf::normpdf;
mod normcdf;
pub use normcdf::normcdf;
mod norminv;
pub use norminv::norminv;
mod lognpdf;
pub use lognpdf::lognpdf;
mod logncdf;
pub use logncdf::logncdf;
mod logninv;
pub use logninv::logninv;
mod poisspdf;
pub use poisspdf::poisspdf;
mod poisscdf;
pub use poisscdf::poisscdf;
mod poissinv;
pub use poissinv::poissinv;
mod tpdf;
pub use tpdf::tpdf;
mod tcdf;
pub use tcdf::tcdf;
mod tinv;
pub use tinv::tinv;
mod nctpdf;
pub use nctpdf::nctpdf;
mod nctcdf;
pub use nctcdf::nctcdf;
mod nctinv;
pub use nctinv::nctinv;
mod wblpdf;
pub use wblpdf::wblpdf;
mod wblcdf;
pub use wblcdf::wblcdf;
mod wblinv;
pub use wblinv::wblinv;
mod stirlerr;
mod binodeviance;
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
