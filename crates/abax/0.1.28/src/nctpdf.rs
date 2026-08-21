use crate::{normpdf, nctcdf, gammaln};

/// Noncentral T probability density function (PDF).
///
/// Returns the probability density function for the noncentral T distribution
/// with `nu` degrees of freedom and noncentrality parameter `delta` at the value `x`.
///
/// # Mathematical Definition
/// The noncentral T probability density function <math><mi>f</mi><mo>(</mo><mi>x</mi><mo>;</mo><mi>ν</mi><mo>,</mo><mi>δ</mi><mo>)</mo></math>
/// is defined using the noncentral T cumulative distribution function <math><msub><mi>F</mi><mrow><mi>NC</mi><mi>T</mi></mrow></msub></math>:
/// <math display="block">
///   <mi>f</mi>
///   <mo>(</mo>
///   <mi>x</mi>
///   <mo>;</mo>
///   <mi>ν</mi>
///   <mo>,</mo>
///   <mi>δ</mi>
///   <mo>)</mo>
///   <mo>=</mo>
///   <mfrac>
///     <mi>ν</mi>
///     <mi>x</mi>
///   </mfrac>
///   <mfenced open="[" close="]">
///     <mrow>
///       <msub>
///         <mi>F</mi>
///         <mrow>
///           <mi>NC</mi>
///          <mi>T</mi>
///        </mrow>
///      </msub>
///      <mfenced separators="|" open="(" close=")">
///        <mrow>
///          <mi>x</mi>
///          <msqrt>
///            <mfrac>
///              <mrow>
///                <mi>ν</mi>
///                <mo>+</mo>
///                <mn>2</mn>
///              </mrow>
///              <mi>ν</mi>
///            </mfrac>
///          </msqrt>
///          <mo>;</mo>
///          <mi>ν</mi>
///          <mo>+</mo>
///          <mn>2</mn>
///          <mo>,</mo>
///          <mi>δ</mi>
///        </mrow>
///      </mfenced>
///      <mo>-</mo>
///      <msub>
///        <mi>F</mi>
///        <mrow>
///          <mi>NC</mi>
///          <mi>T</mi>
///        </mrow>
///      </msub>
///      <mfenced separators="|" open="(" close=")">
///        <mrow>
///          <mi>x</mi>
///          <mo>;</mo>
///          <mi>ν</mi>
///          <mo>,</mo>
///          <mi>δ</mi>
///        </mrow>
///      </mfenced>
///    </mrow>
///  </mfenced>
/// </math>
/// for <math><mi>x</mi><mo>≠</mo><mn>0</mn></math>. Special handling is applied for <math><mi>x</mi><mo>=</mo><mn>0</mn></math> and very large <math><mi>ν</mi></math>.
///
/// # Domain
/// - <math><mi>ν</mi><mo>&gt;</mo><mn>0</mn></math>
/// - <math><mi>δ</mi></math> must be finite.
/// - Returns `NaN` if any input is `NaN`, <math><mi>ν</mi><mo>≤</mo><mn>0</mn></math>, or <math><mi>δ</mi></math> is infinite.
/// - Returns `0.0` if <math><mi>x</mi></math> is infinite.
///
/// # Examples
/// ```
/// use abax::nctpdf;
///
/// // For delta = 0, it reduces to the central Student's T distribution PDF.
/// // Compare with tpdf(0.0, 5.0)
/// let pdf_val = nctpdf(0.0, 5.0, 0.0);
/// assert!((pdf_val - 3.796066898224945e-01).abs() < 1e-12);
///
/// // Example with non-zero delta
/// let pdf_val_noncentral = nctpdf(1.0, 10.0, 2.0);
/// assert!((pdf_val_noncentral - 2.413718676159760e-01).abs() < 1e-12);
/// ```
pub fn nctpdf(x: f64, nu: f64, delta: f64) -> f64 {
    // Invalid parameters or missing data → NaN
    if x.is_nan() || !(delta.is_finite() && nu > 0.0) {
        return f64::NAN;
    }

    // Normal approximation when degrees of freedom are extremely large.
    if nu > 1e11 {
        let s = 1.0 - 1.0 / (4.0 * nu);
        let d = (1.0 + x * x / (2.0 * nu)).sqrt();
        return normpdf(x * s, delta, d);
    }

    // Zero is a special case – closed form using gamma functions.
    if x == 0.0 {
        // log f(0) = -0.5·δ² - 0.5·ln(πν) + ln Γ((ν+1)/2) - ln Γ(ν/2)
        let log_pdf = -0.5 * delta.powi(2)
            - 0.5 * (std::f64::consts::PI * nu).ln()
            + gammaln(0.5 * (nu + 1.0))
            - gammaln(0.5 * nu);
        return log_pdf.exp();
    }

    // Density at ±∞ is zero.
    if !x.is_finite() {
        return 0.0;
    }

    if x < 0.0 {
        // Negative x: use left tail of CDF to obtain PDF.
        let sqrt_term = x * ((nu + 2.0) / nu).sqrt();
        let cdf1 = nctcdf(sqrt_term, nu + 2.0, delta, false); // lower tail
        let cdf2 = nctcdf(x, nu, delta, false);
        (nu / x) * (cdf1 - cdf2)
    } else {
        // Positive x: reflect about zero and use left tail of reflected CDF.
        let sqrt_term = -x * ((nu + 2.0) / nu).sqrt();
        let cdf1 = nctcdf(sqrt_term, nu + 2.0, -delta, false);
        let cdf2 = nctcdf(-x, nu, -delta, false);
        println!("nctcdf({:.16e}, {:.16e}, {:.16e}, false) = {:.16e}", sqrt_term, nu + 2.0, -delta, cdf1);
        (-nu / x) * (cdf1 - cdf2)
    }
}

#[cfg(test)]
mod tests {
    use super::{nctpdf};
    use crate::tpdf; // Import actual functions for testing

    // ----- Sanity checks for edge cases -----
    #[test]
    fn nan_for_invalid_parameters() {
        assert!(nctpdf(0.0, -1.0, 1.0).is_nan());
        assert!(nctpdf(0.0, 1.0, f64::INFINITY).is_nan());
        assert!(nctpdf(f64::NAN, 5.0, 2.0).is_nan());
        assert!(nctpdf(1.0, 5.0, f64::NAN).is_nan());
    }

    #[test]
    fn zero_at_infinity() {
        assert_eq!(nctpdf(f64::INFINITY, 5.0, 1.0), 0.0);
        assert_eq!(nctpdf(f64::NEG_INFINITY, 5.0, 1.0), 0.0);
        assert_eq!(nctpdf(f64::INFINITY, 5.0, 0.0), 0.0);
    }

    #[test]
    fn normal_approximation_triggers_for_large_nu() {
        // nu > 1e11 uses the normal approximation path.
        // This should return a finite value.
        let y = nctpdf(0.5, 2e11, 1.0);
        assert!(y.is_finite() && y >= 0.0);
    }

    #[test]
    fn zero_special_case_gives_finite_value() {
        let y = nctpdf(0.0, 5.0, 1.0);
        println!("{:16e}", y);
        // Reference value from MATLAB: nctpdf(0, 5, 1) = 2.302430960093666e-01
        assert!((y - 2.302430960093666e-01).abs() < 1e-12);
    }

    #[test]
    fn test_nctpdf_central_t() {
        let x = 0.5;
        let nu = 10.0;
        let p_nct = nctpdf(x, nu, 0.0);
        let p_t = tpdf(x, nu);
        assert!((p_nct - p_t).abs() < 1e-12);
    }

    #[test]
    fn test_nctpdf_known_values() {
        let tol = 1e-14; // Lower tolerance due to iterative nature and potential precision differences
        // Reference values from R's dt function (noncentral t)
        // dt(1, df=5, ncp=1)
        println!("{:16e}", nctpdf(1.0, 5.0, 1.0));
        assert!((nctpdf(1.0, 5.0, 1.0) - 3.623248392337183e-01).abs() < tol);
        // dt(2, df=10, ncp=2)
        assert!((nctpdf(2.0, 10.0, 2.0) - 3.556436303616300e-1).abs() < tol);
    }

    #[test]
    fn test_nctpdf_symmetry() {
        assert!((nctpdf(1.0, 5.0, 1.0) - nctpdf(-1.0, 5.0, -1.0)).abs() < 1e-12);
    }
}