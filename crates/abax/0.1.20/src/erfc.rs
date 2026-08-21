use crate::erf::erf_imp;

/// Calculates the complementary error function `erfc(x)`.
///
/// The complementary error function is defined as `erfc(x) = 1 - erf(x)`.
/// This implementation evaluates the complementary form directly in the tail
/// regions to avoid cancellation when `erf(x)` is close to `1`.
///
/// # Examples
/// ```
/// use abax::erfc;
///
/// assert_eq!(erfc(0.0), 1.0);
/// assert!((erfc(1.0) - 0.15729920705028513).abs() < 1e-15);
/// ```
pub fn erfc(x: f64) -> f64 {
    erf_imp(x, true)
}

#[cfg(test)]
mod tests {
    use super::erfc;

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() <= tol,
            "actual={actual:?}, expected={expected:?}, diff={:?}",
            (actual - expected).abs()
        );
    }

    #[test]
    fn handles_special_values() {
        assert!(erfc(f64::NAN).is_nan());
        assert_eq!(erfc(f64::INFINITY), 0.0);
        assert_eq!(erfc(f64::NEG_INFINITY), 2.0);
        assert_eq!(erfc(0.0), 1.0);
        assert_eq!(erfc(-0.0), 1.0);
    }

    #[test]
    fn matches_reference_values() {
        let cases = [
            (-3.0, 1.999_977_909_503_001_5, 1e-15),
            (-1.0, 1.842_700_792_949_715, 1e-15),
            (-0.5, 1.520_499_877_813_046_5, 1e-15),
            (0.125, 0.859_683_795_198_666_2, 1e-15),
            (0.5, 0.479_500_122_186_953_5, 1e-15),
            (1.0, 0.157_299_207_050_285_13, 1e-15),
            (1.5, 0.033_894_853_524_689_27, 1e-15),
            (2.0, 0.004_677_734_981_047_266, 1e-17),
            (4.0, 0.000_000_015_417_257_900_280_02, 1e-22),
            (6.0, 2.151_973_671_249_891_3e-17, 1e-31),
        ];

        for (x, expected, tol) in cases {
            assert_close(erfc(x), expected, tol);
        }
    }
}
