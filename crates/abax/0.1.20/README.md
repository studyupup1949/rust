# abax

A lightweight Rust library providing high-precision mathematical constants and special functions.

## Features

- **Mathematical Constants**:
  - Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>) up to <math><mi>n</mi><mo>=</mo><mn>16</mn></math>.
  - Riemann Zeta function values for integers <math><mn>2</mn></math> through <math><mn>16</mn></math>.
  - Euler-Mascheroni constant.
  - Stirling series coefficients for asymptotic expansions.
- **Special Functions**:
  - `erf(x)`: Error function <math><mrow><mi>erf</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>=</mo><mfrac><mn>2</mn><msqrt><mi>π</mi></msqrt></mfrac><msubsup><mo>∫</mo><mn>0</mn><mi>x</mi></msubsup><msup><mi>e</mi><mrow><mo>-</mo><msup><mi>t</mi><mn>2</mn></msup></mrow></msup><mi>d</mi><mi>t</mi></mrow></math>, implemented with piecewise 53-bit rational approximations for accurate `f64` results.
  - `erfc(x)`: Complementary error function <math><mrow><mi>erfc</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>=</mo><mn>1</mn><mo>-</mo><mi>erf</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>=</mo><mfrac><mn>2</mn><msqrt><mi>π</mi></msqrt></mfrac><msubsup><mo>∫</mo><mi>x</mi><mi>∞</mi></msubsup><msup><mi>e</mi><mrow><mo>-</mo><msup><mi>t</mi><mn>2</mn></msup></mrow></msup><mi>d</mi><mi>t</mi></mrow></math>, evaluated directly in tail regions to avoid cancellation.
  - `erfcx(x)`: Scaled complementary error function <math><mrow><mi>erfcx</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>=</mo><msup><mi>e</mi><msup><mi>x</mi><mn>2</mn></msup></msup><mi>erfc</mi><mo>(</mo><mi>x</mi><mo>)</mo></mrow></math>, implemented with Laplace's continued fraction and reflection formula for numerical stability across all <math><mi>x</mi></math>.
  - `erfinv(x)`: Inverse error function implemented with piecewise rational approximations for `f64` inputs in `[-1, 1]`.
  - `erfcinv(x)`: Inverse complementary error function implemented with piecewise rational approximations for `f64` inputs in `[0, 2]`.
  - `gamma(x)`: Gamma function <math><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math> computed with the Lanczos approximation and reflection formula.
  - `beta(z, w)`: Beta function <math><mrow><mi>B</mi><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo><mo>=</mo><mfrac><mrow><mi>Γ</mi><mo>(</mo><mi>z</mi><mo>)</mo><mi>Γ</mi><mo>(</mo><mi>w</mi><mo>)</mo></mrow><mrow><mi>Γ</mi><mo>(</mo><mi>z</mi><mo>+</mo><mi>w</mi><mo>)</mo></mrow></mfrac></mrow></math>, implemented using the logarithmic Gamma function for numerical stability.
  - `betainc(x, z, w, lower)`: Regularized incomplete beta function <math><mrow><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo><mo>=</mo><mfrac><mn>1</mn><mrow><mi>B</mi><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo></mrow></mfrac><msubsup><mo>∫</mo><mn>0</mn><mi>x</mi></msubsup><msup><mi>t</mi><mrow><mi>z</mi><mo>-</mo><mn>1</mn></mrow></msup><msup><mrow><mo>(</mo><mn>1</mn><mo>-</mo><mi>t</mi><mo>)</mo></mrow><mrow><mi>w</mi><mo>-</mo><mn>1</mn></mrow></msup><mi>d</mi><mi>t</mi></mrow></math>; returns upper tail when `lower=false`.
  - `betaincinv(y, z, w, lower)`: Inverse of the regularized incomplete beta function; returns `x` such that `betainc(x, z, w, lower) = y`.
  - `betaln(z, w)`: Natural logarithm of the Beta function <math><mi>ln</mi><mo>(</mo><mi>B</mi><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo><mo>)</mo></math>, computed using logarithmic Gamma functions.
  - `gammaln(x)`: Natural logarithm of the Gamma function <math><mi>ln</mi><mo>(</mo><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>)</mo></math> using Stirling's approximation for high precision.
  - `digamma(x)`: Digamma function <math><mi>ψ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math>, the logarithmic derivative of <math><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math>, implemented with reflection, recurrence shifting, and asymptotic expansion for numerical stability.
  - `trigamma(x)`: Trigamma function <math><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math>, implemented with reflection, recurrence shifting, and asymptotic expansion for numerical stability.
  - `tetragamma(x)`: Tetragamma function <math><msup><mi>ψ</mi><mo>(</mo><mn>2</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math>, implemented with reflection, recurrence shifting, and asymptotic expansion for numerical stability.
  - `gammainc(x, a, lower, scaled)`: Regularized lower/upper incomplete gamma function with explicit lower/upper tail selection; when `scaled=true`, returns tail value multiplied by <math><mi>Γ</mi><mo>(</mo><mi>a</mi><mo>+</mo><mn>1</mn><mo>)</mo><msup><mi>e</mi><mi>x</mi></msup><msup><mi>x</mi><mrow><mo>-</mo><mi>a</mi></mrow></msup></math> for improved conditioning at large <math><mi>x</mi></math>.
  - `normcdf(x, mu, sigma)`: Normal cumulative distribution function.
  - `normpdf(x, mu, sigma)`: Normal probability density function.
  - `norminv(p, mu, sigma)`: Inverse of the normal cumulative distribution function (quantile function).
  - `gammaincinv(y, a, lower)`: Numerically robust inverse of the regularized incomplete gamma function utilizing a combined Halley-Newton-Bisection method for enhanced convergence speed and stability in tail regions.
  - `psi(k, x)`: Polygamma interface returning <math><msup><mi>ψ</mi><mo>(</mo><mi>k</mi><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math> with stable handling of poles and infinities.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
abax = "0.1.20"
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Repository

Source code is available on GitHub.
