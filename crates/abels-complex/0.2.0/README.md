# abels-complex
Complex numbers with rectangular and polar representations

### design decisions
ComplexPolar does not implicitly normalize (|z| >= 0 and -PI < Arg(z) >=PI)

no implicit conversions between rectangular and polar forms just to get back the original type

### `no_std` support

`no_std` support can be enabled by compiling with `--no-default-features` to
disable `std` support and `--features libm` for math functions that are only
defined in `std`.

### features

* [`approx`] - traits and macros for approximate float comparisons
* [`libm`] - uses `libm` math functions instead of `std`
* [`glam`] - implements `From<glam::Vec2> for Complex` and `From<Complex> for glam::Vec2`
* [`rand`] - implements `rand::distr::Distribution<Complex> for rand::distr::StandardUniform`
and `rand::distr::Distribution<ComplexPolar> for rand::distr::StandardUniform`
