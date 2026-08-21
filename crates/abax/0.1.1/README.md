# abax

A lightweight Rust library providing high-precision mathematical constants and special functions.

## Features

- **Mathematical Constants**:
  - Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>) up to <math><mi>n</mi><mo>=</mo><mn>16</mn></math>.
  - Riemann Zeta function values for integers <math><mn>2</mn></math> through <math><mn>16</mn></math>.
  - Euler-Mascheroni constant.
  - Stirling series coefficients for asymptotic expansions.
- **Special Functions**:
  - `gammaln(x)`: Natural logarithm of the Gamma function <math><mi>ln</mi><mo>(</mo><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>)</mo></math> using Stirling's approximation for high precision.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
abax = "0.1.0"
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Repository

Source code is available on GitHub.