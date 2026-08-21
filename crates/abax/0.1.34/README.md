# abax

A lightweight Rust library providing high-precision mathematical constants and special functions.

![Crates.io License](https://img.shields.io/crates/l/abax)
![docs.rs](https://img.shields.io/docsrs/abax)
![Crates.io Size](https://img.shields.io/crates/size/abax)
![Crates.io Total Downloads](https://img.shields.io/crates/d/abax)

[crates-badge]: http

## Features

- **Mathematical Constants**:
  - Bernoulli numbers (<math><msub><mi>B</mi><mrow><mn>2</mn><mi>n</mi></mrow></msub></math>) up to <math><mi>n</mi><mo>=</mo><mn>16</mn></math>.
  - Riemann Zeta function values for integers <math><mn>2</mn></math> through <math><mn>16</mn></math>.
  - Euler-Mascheroni constant.
  - Stirling series coefficients for asymptotic expansions.
- **Special Functions**:
  - **Gamma and Polygamma Functions**:
    - `gamma(x)`: Gamma function <math><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo></math> computed with the Lanczos approximation and reflection formula.
    - `gammaln(x)`: Natural logarithm of the Gamma function <math><mi>ln</mi><mo>(</mo><mi>Γ</mi><mo>(</mo><mi>x</mi><mo>)</mo><mo>)</mo></math> using Stirling's approximation.
    - `digamma(x)`, `trigamma(x)`, `tetragamma(x)`: Polygamma functions <math><msup><mi>ψ</mi><mrow><mo>(</mo><mi>n</mi><mo>)</mo></mrow></msup><mo>(</mo><mi>x</mi><mo>)</mo></math> implemented with reflection and recurrence shifting.
    - `psi(k, x)`: Polygamma interface for arbitrary order $k$ with stable handling of poles and infinities.
    - `gammainc(x, a, lower, scaled)`: Regularized incomplete gamma function with scaling for improved conditioning.
    - `gammaincinv(y, a, lower)`: Robust inverse incomplete gamma function using a combined Halley-Newton-Bisection method.
  - **Beta Functions**:
    - `beta(z, w)`, `betaln(z, w)`: Beta function and its natural logarithm.
    - `betainc(x, z, w, lower)`: Regularized incomplete beta function <math><mrow><msub><mi>I</mi><mi>x</mi></msub><mo>(</mo><mi>z</mi><mo>,</mo><mi>w</mi><mo>)</mo></mrow></math>.
    - `betaincinv(y, z, w, lower)`: Inverse of the regularized incomplete beta function.
  - **Error Functions**:
    - `erf(x)`, `erfc(x)`, `erfcx(x)`: Error function, complementary error function, and scaled variant.
    - `erfinv(x)`, `erfcinv(x)`: Inverse error functions implemented with piecewise rational approximations.
- **Probability Distributions**:

  | Family | PDF | CDF | Inverse (Quantile) |
  |:---|:---|:---|:---|
  | **Normal** | `normpdf(x, μ, σ)` | `normcdf(x, μ, σ, upper)` | `norminv(p, μ, σ)` |
  | **Lognormal** | `lognpdf(x, μ, σ)` | `logncdf(x, μ, σ, upper)` | `logninv(p, μ, σ)` |
  | **Student's T** | `tpdf(x, v)` | `tcdf(x, v, upper)` | `tinv(p, v)` |
  | **Noncentral T**| `nctpdf(x, ν, δ)`| `nctcdf(x, ν, δ, upper)` | `nctinv(p, ν, δ)` |
  | **Gamma** | `gampdf(x, a, b)` | `gamcdf(x, a, b, upper)` | `gaminv(p, a, b)` |
  | **Beta** | `betapdf(x, a, b)` | `betacdf(x, a, b, upper)` | `betainv(p, a, b)` |
  
## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
abax = "0.1.34"
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Repository

Source code is available on GitHub.
