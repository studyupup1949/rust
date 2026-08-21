# abax

A lightweight Rust library providing high-precision mathematical constants and special functions.

## Features

- **Mathematical Constants**:
  - Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>) up to <math><mi>n</mi><mo>=</mo><mn>16</mn></math>.
  - Riemann Zeta function values for integers <math><mn>2</mn></math> through <math><mn>16</mn></math>.
  - Euler-Mascheroni constant.
  - Stirling series coefficients for asymptotic expansions.
- **Special Functions**:
  - `gamma(x)`: Gamma function <math><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math> computed with the Lanczos approximation and reflection formula.
  - `gammaln(x)`: Natural logarithm of the Gamma function <math><mi>ln</mi><mo>(</mo><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>)</mo></math> using Stirling's approximation for high precision.
  - `digamma(x)`: Digamma function <math><mi>ψ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math>, the logarithmic derivative of <math><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math>, implemented with reflection, recurrence shifting, and asymptotic expansion for numerical stability.
  - `trigamma(x)`: Trigamma function <math><msup><mi>ψ</mi><mo>(</mo><mn>1</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math>, implemented with reflection, recurrence shifting, and asymptotic expansion for numerical stability.
  - `tetragamma(x)`: Tetragamma function <math><msup><mi>ψ</mi><mo>(</mo><mn>2</mn><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math>, implemented with reflection, recurrence shifting, and asymptotic expansion for numerical stability.
  - `gammainc(x, a, lower, scaled)`: Regularized lower/upper incomplete gamma function with explicit lower/upper tail selection; when `scaled=true`, returns tail value multiplied by <math><mi>Γ</mi><mo>(</mo><mi>a</mi><mo>+</mo><mn>1</mn><mo>)</mo><msup><mi>e</mi><mi>x</mi></msup><msup><mi>x</mi><mrow><mo>-</mo><mi>a</mi></mrow></msup></math> for improved conditioning at large <math><mi>x</mi></math>.
  - `gammaincinv(y, a, lower)`: Numerically robust inverse of the regularized incomplete gamma function utilizing a combined Halley-Newton-Bisection method for enhanced convergence speed and stability in tail regions.
  - `psi(k, x)`: Polygamma interface returning <math><msup><mi>ψ</mi><mo>(</mo><mi>k</mi><mo>)</mo></msup><mo>(</mo><mi>x</mi><mo>)</mo></math> with stable handling of poles and infinities.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
abax = "0.1.8"
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Repository

Source code is available on GitHub.
