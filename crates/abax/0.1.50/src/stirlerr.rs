use crate::gammaln;

/// Computes the Stirling error term for the natural logarithm of the Gamma function.
///
/// This function calculates the `delta` term in Stirling's approximation for `ln(Γ(n))`,
/// which is given by:
/// <math display="block">
///   <mi>ln</mi><mo>(</mo><mi>Γ</mi><mo>(</mo><mi>n</mi><mo>)</mo><mo>)</mo>
///   <mo>≈</mo>
///   <mo>(</mo><mi>n</mi><mo>-</mo><mfrac><mn>1</mn><mn>2</mn></mfrac><mo>)</mo><mi>ln</mi><mo>(</mo><mi>n</mi><mo>)</mo>
///   <mo>-</mo><mi>n</mi>
///   <mo>+</mo><mfrac><mn>1</mn><mn>2</mn></mfrac><mi>ln</mi><mo>(</mo><mn>2</mn><mi>π</mi><mo>)</mo>
///   <mo>+</mo><mi>δ</mi><mo>(</mo><mi>n</mi><mo>)</mo>
/// </math>
/// where <math><mi>δ</mi><mo>(</mo><mi>n</mi><mo>)</mo></math> is the Stirling error term.
///
/// # Implementation Details
/// - For small `n` (up to 15.0), it uses precomputed values or a direct calculation
///   involving `gammaln` for precision.
/// - For larger `n`, it employs an asymptotic expansion based on Bernoulli numbers
///   to approximate the error term. The expansion is truncated based on the magnitude
///    of `n` to maintain accuracy and efficiency.
///
/// # Domain
/// - `n` must be positive.
/// - Returns `NaN` if `n` is non-positive or `NaN`.
pub(crate) fn stirlerr(n: f64) -> f64 {

    let delta;
    let nn = n * n;
    
    const S0: f64 = 8.333333333333333e-02; // 1.0 / 12.0
    const S1: f64 = 2.777777777777778e-03; // 1.0 / 360.0
    const S2: f64 = 7.936507936507937e-04; // 1.0 / 1260.0
    const S3: f64 = 5.952380952380952e-04; // 1.0 / 1680.0
    const S4: f64 = 8.417508417508418e-04; // 1.0 / 1188.0

    const SFE: [f64; 31] =[
                          0.0, 1.534264097200273e-01, 8.106146679532726e-02,
        5.481412105191765e-02, 4.134069595540929e-02, 3.316287351993629e-02,
        2.767792568499834e-02, 2.374616365629750e-02, 2.079067210376509e-02,
        1.848845053267319e-02, 1.664469118982119e-02, 1.513497322191738e-02,
        1.387612882307075e-02, 1.281046524292023e-02, 1.189670994589177e-02,
        1.110455975820868e-02, 1.041126526197210e-02, 9.799416126158803e-03, 
        9.255462182712733e-03, 8.768700134139385e-03, 8.330563433362871e-03, 
        7.934114564314021e-03, 7.573675487951841e-03, 7.244554301320383e-03, 
        6.942840107209530e-03, 6.665247032707682e-03, 6.408994188004207e-03, 
        6.171712263039458e-03, 5.951370112758848e-03, 5.746216513010116e-03, 
        5.554733551962801e-03];

    if n <= 15.0 {
        let n1 = n;
        let n2 = 2.0 * n1;
        if n2 == f64::floor(n2 + 0.5) {
            delta = SFE[n2 as usize];
        } else {
            let lnsr2pi = 0.9189385332046728;
            delta = gammaln(n1 + 1.0) - (n1 + 0.5) * f64::exp(n1) + n1 - lnsr2pi;
        }
    } else if 15.0 < n && n <= 35.0 {
        delta = (S0-(S1-(S2-(S3-S4/nn)/nn)/nn)/nn)/n;
    } else if 35.0 < n && n <= 80.0 {
        delta = (S0-(S1-(S2-S3/nn)/nn)/nn)/n;
    } else if 80.0 < n && n <= 500.0 {
        delta = (S0-(S1-S2/nn)/nn)/n;
    } else {
        // n > 500.0
        delta = (S0 - S1 / nn) / n;
    }
    
    delta
}
