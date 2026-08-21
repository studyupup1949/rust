use std::ops::{Add, Div, Mul, Neg, Sub};

pub(crate) trait Normalize {
    fn normalized_to_scale(&self, max: f64) -> f64;
}

impl Normalize for f64 {
    fn normalized_to_scale(&self, max: f64) -> f64 {
        self - (max * (self / max).floor())
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Angle {
    pub degrees: f64,
}

impl Angle {
    pub const fn from_degrees(degrees: f64) -> Self {
        Angle { degrees }
    }

    pub const fn from_radians(radians: f64) -> Self {
        Angle {
            degrees: radians.to_degrees(),
        }
    }

    pub const fn radians(self) -> f64 {
        self.degrees.to_radians()
    }

    pub fn unwound(self) -> Angle {
        Angle {
            degrees: self.degrees.normalized_to_scale(360.0),
        }
    }

    pub fn quadrant_shifted(self) -> Angle {
        if self.degrees >= -180.0 && self.degrees <= 180.0 {
            self
        } else {
            Angle::from_degrees(self.degrees - (360.0 * (self.degrees / 360.0).round()))
        }
    }
}

impl Add for Angle {
    type Output = Angle;

    fn add(self, rhs: Angle) -> Angle {
        Angle {
            degrees: self.degrees + rhs.degrees,
        }
    }
}

impl Sub for Angle {
    type Output = Angle;

    fn sub(self, rhs: Angle) -> Angle {
        Angle {
            degrees: self.degrees - rhs.degrees,
        }
    }
}

impl Mul<f64> for Angle {
    type Output = Angle;

    fn mul(self, rhs: f64) -> Angle {
        Angle {
            degrees: self.degrees * rhs,
        }
    }
}

impl Div<f64> for Angle {
    type Output = Angle;

    fn div(self, rhs: f64) -> Angle {
        if rhs == 0.0 && cfg!(debug_assertions) {
            panic!("cannot divide by zero.");
        }

        Angle {
            degrees: self.degrees / rhs,
        }
    }
}

impl Neg for Angle {
    type Output = Angle;

    fn neg(self) -> Angle {
        Angle {
            degrees: -self.degrees,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use std::f64::consts::PI;

    #[test]
    fn angle_conversion_from_radians() {
        assert_abs_diff_eq!(Angle::from_radians(PI).degrees, 180.0, epsilon = 1e-10);
        assert_abs_diff_eq!(Angle::from_radians(PI / 2.0).degrees, 90.0, epsilon = 1e-10);
    }

    #[test]
    fn angle_conversion_degrees_to_radians() {
        assert_abs_diff_eq!(Angle::from_degrees(180.0).radians(), PI, epsilon = 1e-10);
        assert_abs_diff_eq!(
            Angle::from_degrees(90.0).radians(),
            PI / 2.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn normalize_value() {
        assert_abs_diff_eq!(2.0_f64.normalized_to_scale(-5.0), -3.0, epsilon = 1e-10);
        assert_abs_diff_eq!((-4.0_f64).normalized_to_scale(-5.0), -4.0, epsilon = 1e-10);
        assert_abs_diff_eq!((-6.0_f64).normalized_to_scale(-5.0), -1.0, epsilon = 1e-10);

        assert_abs_diff_eq!((-1.0_f64).normalized_to_scale(24.0), 23.0, epsilon = 1e-10);
        assert_abs_diff_eq!(1.0_f64.normalized_to_scale(24.0), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(49.0_f64.normalized_to_scale(24.0), 1.0, epsilon = 1e-10);

        assert_abs_diff_eq!(361.0_f64.normalized_to_scale(360.0), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(360.0_f64.normalized_to_scale(360.0), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(259.0_f64.normalized_to_scale(360.0), 259.0, epsilon = 1e-10);
        assert_abs_diff_eq!(2592.0_f64.normalized_to_scale(360.0), 72.0, epsilon = 1e-10);
    }

    #[test]
    fn angle_unwound() {
        assert_abs_diff_eq!(
            Angle::from_degrees(-45.0).unwound().degrees,
            315.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(361.0).unwound().degrees,
            1.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(360.0).unwound().degrees,
            0.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(259.0).unwound().degrees,
            259.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(2592.0).unwound().degrees,
            72.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn closest_angle() {
        assert_abs_diff_eq!(
            Angle::from_degrees(360.0).quadrant_shifted().degrees,
            0.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(361.0).quadrant_shifted().degrees,
            1.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(1.0).quadrant_shifted().degrees,
            1.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(-1.0).quadrant_shifted().degrees,
            -1.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(-181.0).quadrant_shifted().degrees,
            179.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(180.0).quadrant_shifted().degrees,
            180.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(359.0).quadrant_shifted().degrees,
            -1.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(-359.0).quadrant_shifted().degrees,
            1.0,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(1261.0).quadrant_shifted().degrees,
            -179.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn adding_angles() {
        let angle_a = Angle::from_degrees(45.0);
        let angle_b = Angle::from_degrees(45.0);

        assert_abs_diff_eq!((angle_a + angle_b).degrees, 90.0)
    }
}
