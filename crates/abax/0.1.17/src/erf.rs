#[inline]
fn evaluate_polynomial(coeffs: &[f64], x: f64) -> f64 {
    let mut result = *coeffs.last().unwrap();
    for &c in coeffs[..coeffs.len() - 1].iter().rev() {
        result = result * x + c;
    }
    result
}

#[inline]
fn truncated_26_bit_head(x: f64) -> f64 {
    if !x.is_normal() {
        return x;
    }

    // Match a floor(ldexp(frexp(x), 26)) / 2^26 split for positive
    // normal f64 values: keep 26 significant bits, including the hidden bit.
    const LOWER_27_FRACTION_BITS: u64 = (1_u64 << 27) - 1;
    f64::from_bits(x.to_bits() & !LOWER_27_FRACTION_BITS)
}

#[inline]
fn exp_neg_z2_over_z(z: f64) -> f64 {
    let hi = truncated_26_bit_head(z);
    let lo = z - hi;
    let sq = z * z;
    let err_sqr = ((hi * hi - sq) + 2.0 * hi * lo) + lo * lo;

    (-sq).exp() * (-err_sqr).exp() / z
}

#[inline]
pub(crate) fn erf_imp(mut z: f64, mut invert: bool) -> f64 {
    if z.is_nan() {
        return f64::NAN;
    }

    let mut prefix_multiplier = 1.0;
    let mut prefix_adder = 0.0;

    if z < 0.0 {
        z = -z;
        if !invert {
            prefix_multiplier = -1.0;
        } else if z > 0.5 {
            prefix_adder = 2.0;
            prefix_multiplier = -1.0;
        } else {
            invert = false;
            prefix_adder = 1.0;
        }
    }

    let result;

    if z < 0.5 {
        if z < 1.0e-10 {
            result = if z == 0.0 {
                0.0
            } else {
                z * 1.125 + z * 0.003_379_167_095_512_574
            };
        } else {
            const Y: f64 = 1.044_948_577_880_859_4;
            const P: [f64; 5] = [
                0.083_430_589_214_653_18,
                -0.338_165_134_459_360_93,
                -0.050_999_073_514_677_745,
                -0.007_727_583_458_021_333,
                -0.000_322_780_120_964_605_7,
            ];
            const Q: [f64; 5] = [
                1.0,
                0.455_004_033_050_794,
                0.087_522_260_014_225_25,
                0.008_585_719_250_744_063,
                0.000_370_900_071_787_748,
            ];
            let zz = z * z;
            result = z * (Y + evaluate_polynomial(&P, zz) / evaluate_polynomial(&Q, zz));
        }
    } else if if invert { z < 28.0 } else { z < 5.93 } {
        invert = !invert;

        result = if z < 1.5 {
            const Y: f64 = 0.405_935_764_312_744_14;
            const P: [f64; 6] = [
                -0.098_090_592_216_281_24,
                0.178_114_665_841_120_34,
                0.191_003_695_796_775_43,
                0.088_890_036_896_788_45,
                0.019_504_900_125_121_88,
                0.001_804_245_382_970_142_2,
            ];
            const Q: [f64; 7] = [
                1.0,
                1.847_590_709_830_022,
                1.426_280_048_455_113_2,
                0.578_052_804_889_902_4,
                0.123_850_974_679_008_64,
                0.011_338_523_357_700_141,
                0.337_511_472_483_094_7e-5,
            ];
            let zm = z - 0.5;
            (Y + evaluate_polynomial(&P, zm) / evaluate_polynomial(&Q, zm)) * (-z * z).exp() / z
        } else if z < 2.5 {
            const Y: f64 = 0.506_728_172_302_246_1;
            const P: [f64; 6] = [
                -0.024_350_047_620_769_844,
                0.038_654_037_503_570_72,
                0.043_948_189_642_095_16,
                0.017_567_943_631_180_21,
                0.003_239_624_062_908_421_4,
                0.000_235_839_115_596_880_72,
            ];
            const Q: [f64; 6] = [
                1.0,
                1.539_914_949_485_524_5,
                0.982_403_709_157_920_2,
                0.325_732_924_782_444_45,
                0.056_392_183_742_047_816,
                0.004_103_697_239_789_046,
            ];
            let zm = z - 1.5;
            (Y + evaluate_polynomial(&P, zm) / evaluate_polynomial(&Q, zm)) * exp_neg_z2_over_z(z)
        } else if z < 4.5 {
            const Y: f64 = 0.540_575_027_465_820_3;
            const P: [f64; 6] = [
                0.002_952_767_165_309_716_6,
                0.013_738_442_589_635_533,
                0.008_408_076_155_555_854,
                0.002_128_256_209_146_186_4,
                0.000_250_269_961_544_794_6,
                0.113_212_406_648_847_56e-4,
            ];
            const Q: [f64; 6] = [
                1.0,
                1.042_178_141_669_384_2,
                0.442_597_659_481_563_14,
                0.095_849_272_630_106_14,
                0.010_598_290_648_487_653,
                0.000_479_411_269_521_714_5,
            ];
            let zm = z - 3.5;
            (Y + evaluate_polynomial(&P, zm) / evaluate_polynomial(&Q, zm)) * exp_neg_z2_over_z(z)
        } else {
            const Y: f64 = 0.557_909_011_840_820_3;
            const P: [f64; 7] = [
                0.006_280_571_706_269_649,
                0.017_538_983_405_249_33,
                -0.212_652_252_872_804_22,
                -0.687_717_681_153_649_9,
                -2.551_855_172_731_152_4,
                -3.227_294_517_641_437,
                -2.817_540_111_451_338,
            ];
            const Q: [f64; 7] = [
                1.0,
                2.792_577_509_805_753,
                11.056_723_792_780_016,
                15.930_646_027_911_794,
                22.936_737_652_288_058,
                13.506_417_019_180_289,
                5.484_091_822_386_417,
            ];
            (Y + evaluate_polynomial(&P, 1.0 / z) / evaluate_polynomial(&Q, 1.0 / z))
                * exp_neg_z2_over_z(z)
        };
    } else {
        result = 0.0;
        invert = !invert;
    }

    if invert {
        prefix_adder += prefix_multiplier;
        prefix_multiplier = -prefix_multiplier;
    }

    prefix_adder + prefix_multiplier * result
}

/// Calculates the error function `erf(x)`.
///
/// The error function is defined by
/// `erf(x) = 2/sqrt(pi) * integral from 0 to x of exp(-t^2) dt`.
///
/// This implementation uses 53-bit (`f64`) rational approximation tables.
/// It uses a direct approximation for small inputs and complementary
/// error-function approximations for larger magnitudes to preserve accuracy in
/// the tails.
///
/// # Examples
/// ```
/// use abax::erf;
///
/// assert_eq!(erf(0.0), 0.0);
/// assert!((erf(1.0) - 0.8427007929497149).abs() < 1e-15);
/// ```
pub fn erf(x: f64) -> f64 {
    erf_imp(x, false)
}

#[cfg(test)]
mod tests {
    use super::erf;

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() <= tol,
            "actual={actual:?}, expected={expected:?}, diff={:?}",
            (actual - expected).abs()
        );
    }

    #[test]
    fn handles_special_values() {
        assert!(erf(f64::NAN).is_nan());
        assert_eq!(erf(f64::INFINITY), 1.0);
        assert_eq!(erf(f64::NEG_INFINITY), -1.0);
        assert_eq!(erf(0.0), 0.0);
        assert_eq!(erf(-0.0), 0.0);
    }

    #[test]
    fn matches_reference_values() {
        let cases = [
            (-3.0, -0.999_977_909_503_001_4),
            (-1.0, -0.842_700_792_949_714_9),
            (-0.5, -0.520_499_877_813_046_5),
            (0.125, 0.140_316_204_801_333_82),
            (0.5, 0.520_499_877_813_046_5),
            (1.0, 0.842_700_792_949_714_9),
            (1.5, 0.966_105_146_475_310_8),
            (2.0, 0.995_322_265_018_952_7),
            (4.0, 0.999_999_984_582_742_1),
            (6.0, 1.0),
        ];

        for (x, expected) in cases {
            assert_close(erf(x), expected, 1e-15);
        }
    }
}
