use crate::erfinv::erfinv_imp;

/// Calculates the inverse complementary error function `erfc⁻¹(x)`.
///
/// The inverse complementary error function returns the value `y` such that
/// `erfc(y) = x`. Valid finite inputs are in the closed interval `[0, 2]`;
/// values outside that interval return `NaN`, while `0` returns positive
/// infinity and `2` returns negative infinity.
///
/// This implementation uses 64-bit piecewise rational approximations
/// for inverse `erf`/`erfc`.
///
/// # Examples
/// ```
/// use abax::{erfc, erfcinv};
///
/// assert_eq!(erfcinv(1.0), 0.0);
/// assert!((erfc(erfcinv(0.5)) - 0.5).abs() < 1e-15);
/// ```
pub fn erfcinv(x: f64) -> f64 {
    if x.is_nan() || !(0.0..=2.0).contains(&x) {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::INFINITY;
    }
    if x == 2.0 {
        return f64::NEG_INFINITY;
    }
    if x == 1.0 {
        return 0.0;
    }

    let (p, q, sign) = if x < 1.0 {
        (1.0 - x, x, 1.0)
    } else {
        (x - 1.0, 2.0 - x, -1.0)
    };

    sign * erfinv_imp(p, q)
}

#[cfg(test)]
mod tests {
    use super::erfcinv;
    use crate::erfc;

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() <= tol,
            "actual={actual:?}, expected={expected:?}, diff={:?}",
            (actual - expected).abs()
        );
    }

    #[test]
    fn handles_special_values() {
        assert!(erfcinv(f64::NAN).is_nan());
        assert!(erfcinv(-0.000_000_000_000_000_1).is_nan());
        assert!(erfcinv(2.000_000_000_000_000_4).is_nan());
        assert_eq!(erfcinv(0.0), f64::INFINITY);
        assert_eq!(erfcinv(-0.0), f64::INFINITY);
        assert_eq!(erfcinv(2.0), f64::NEG_INFINITY);
        assert_eq!(erfcinv(1.0), 0.0);
    }

    #[test]
    fn matches_reference_values() {
        let cases = [
            (0.000_001, 3.458_910_737_275_498, 5e-12),
            (0.001, 2.326_753_765_513_524_6, 4e-15),
            (0.01, 1.821_386_367_718_449_6, 3e-15),
            (0.1, 1.163_087_153_676_674_3, 2e-15),
            (0.5, 0.476_936_276_204_469_9, 1e-15),
            (1.0, 0.0, 0.0),
            (1.5, -0.476_936_276_204_469_9, 1e-15),
            (1.9, -1.163_087_153_676_674_3, 2e-15),
            (1.99, -1.821_386_367_718_449_6, 3e-15),
            (1.999, -2.326_753_765_513_524_6, 3e-14),
            (1.999_999, -3.458_910_737_275_498, 2e-11),
        ];

        for (x, expected, tol) in cases {
            assert_close(erfcinv(x), expected, tol);
        }
    }

    #[test]
    fn round_trips_through_erfc() {
        let cases = [
            0.000_001, 0.001, 0.01, 0.1, 0.5, 0.9, 1.1, 1.5, 1.9, 1.99, 1.999_999,
        ];

        for x in cases {
            assert_close(erfc(erfcinv(x)), x, 5e-15);
        }
    }
}
