/// Computes the binomial deviance function.
///
/// The deviance is defined as:
/// <math display="block">
///   <mi>D</mi><mo>(</mo><mi>x</mi><mo>,</mo><mi>μ</mi><mo>)</mo>
///   <mo>=</mo>
///   <mi>x</mi><mi>ln</mi><mo>(</mo><mfrac><mi>x</mi><mi>μ</mi></mfrac><mo>)</mo>
///   <mo>+</mo><mi>μ</mi><mo>-</mo><mi>x</mi>
/// </math>
///
/// This is a critical component for Loader's saddle point expansion, used to
/// accurately compute probability density functions for Gamma, Binomial, and
/// Poisson distributions.
///
/// # Implementation Details
/// When <math><mi>x</mi></math> is close to <math><mi>μ</mi></math> (specifically
/// when <math><mo>|</mo><mi>x</mi><mo>-</mo><mi>μ</mi><mo>|</mo><mo>&lt;</mo><mn>0.1</mn><mo>(</mo><mi>x</mi><mo>+</mo><mi>μ</mi><mo>)</mo></math>),
/// the direct logarithmic formula suffers from catastrophic cancellation. In this
/// region, the function switches to a power series expansion to maintain
/// high numerical precision.
pub(crate) fn binodeviance(x: f64, np: f64) -> f64 {
    let bd0: f64 = match f64::abs(x - np) < 0.1 * (x + np) {
        true => {
            let mut s = (x - np) * (x - np) / (x + np);
            let v = (x - np) / (x + np);
            let mut ej = 2.0 * x * v;
            let mut ok = true;
            let mut j = 0;
            while ok {
                ej = ej * v * v;
                j = j + 1;
                let s1 = s + ej / (2.0 * j as f64 + 1.0);
                ok = ok && s1 != s;
                s = s1;
            }
            s
        },
        false => {
            x * f64::ln(x / np) + np - x
        },
    };

    bd0
}
