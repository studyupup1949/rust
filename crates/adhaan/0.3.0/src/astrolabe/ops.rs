use jiff::{Zoned, civil::Date, tz::TimeZone};

use crate::astrolabe::unit::{Angle, Normalize};

/// The geometric mean longitude of the sun.
pub fn mean_solar_longitude(julian_century: f64) -> Angle {
    // Equation from Astronomical Algorithms page 163
    let term1 = 280.4664567;
    let term2 = 36000.76983 * julian_century;
    let term3 = 0.0003032 * julian_century.powf(2.0);
    let degrees = term1 + term2 + term3;

    Angle::from_degrees(degrees).unwound()
}

/// The geometric mean longitude of the moon.
pub fn mean_lunar_longitude(julian_century: f64) -> Angle {
    // Equation from Astronomical Algorithms page 144
    let term1 = 218.3165;
    let term2 = 481267.8813 * julian_century;
    let degrees = term1 + term2;

    Angle::from_degrees(degrees).unwound()
}

pub fn ascending_lunar_node_longitude(julian_century: f64) -> Angle {
    // Equation from Astronomical Algorithms page 144
    let term1 = 125.04452;
    let term2 = 1934.136261 * julian_century;
    let term3 = 0.0020708 * julian_century.powf(2.0);
    let term4 = julian_century.powf(3.0) / 450000.0;
    let degrees = term1 - term2 + term3 + term4;

    Angle::from_degrees(degrees).unwound()
}

/// The mean anomaly of the sun.
pub fn mean_solar_anomaly(julian_century: f64) -> Angle {
    // Equation from Astronomical Algorithms page 163
    let term1 = 357.52911;
    let term2 = 35999.05029 * julian_century;
    let term3 = 0.0001537 * julian_century.powf(2.0);
    let degrees = term1 + term2 - term3;

    Angle::from_degrees(degrees).unwound()
}

/// The Sun's equation of the center.
pub fn solar_equation_of_the_center(julian_century: f64, mean_anomaly: Angle) -> Angle {
    // Equation from Astronomical Algorithms page 164
    let mean_radians = mean_anomaly.radians();
    let term1 = (1.914602 - (0.004817 * julian_century) - (0.000014 * julian_century.powf(2.0)))
        * mean_radians.sin();
    let term2 = (0.019993 - (0.000101 * julian_century)) * (2.0 * mean_radians).sin();
    let term3 = 0.000289 * (3.0 * mean_radians).sin();

    Angle::from_degrees(term1 + term2 + term3)
}

// The apparent longitude of the Sun, referred to the true equinox of the date.
pub fn apparent_solar_longitude(julian_century: f64, mean_longitude: Angle) -> Angle {
    // Equation from Astronomical Algorithms page 164
    let longitude = mean_longitude
        + solar_equation_of_the_center(julian_century, mean_solar_anomaly(julian_century));
    let omega = Angle::from_degrees(125.04 - (1934.136 * julian_century));
    let lambda =
        Angle::from_degrees(longitude.degrees - 0.00569 - (0.00478 * omega.radians().sin()));

    lambda.unwound()
}

/// The mean obliquity of the ecliptic, formula adopted by the International Astronomical Union.
pub fn mean_obliquity_of_the_ecliptic(julian_century: f64) -> Angle {
    // Equation from Astronomical Algorithms page 147
    let term1 = 23.439291;
    let term2 = 0.013004167 * julian_century;
    let term3 = 0.0000001639 * julian_century.powf(2.0);
    let term4 = 0.0000005036 * julian_century.powf(3.0);

    Angle::from_degrees(term1 - term2 - term3 + term4)
}

/// The mean obliquity of the ecliptic, corrected for calculating the apparent position of the sun.
pub fn apparent_obliquity_of_the_ecliptic(
    julian_century: f64,
    mean_obliquity_of_the_ecliptic: Angle,
) -> Angle {
    // Equation from Astronomical Algorithms page 165
    let degrees: f64 = 125.04 - (1934.136 * julian_century);

    Angle::from_degrees(
        mean_obliquity_of_the_ecliptic.degrees
            + (0.00256 * Angle::from_degrees(degrees).radians().cos()),
    )
}

/// Mean sidereal time, the hour angle of the vernal equinox.
pub fn mean_sidereal_time(julian_century: f64) -> Angle {
    // Equation from Astronomical Algorithms page 165
    let julian_day = (julian_century * 36525.0) + 2451545.0;
    let term1 = 280.46061837;
    let term2 = 360.98564736629 * (julian_day - 2451545.0);
    let term3 = 0.000387933 * julian_century.powf(2.0);
    let term4 = julian_century.powf(3.0) / 38710000.0;
    let degrees = term1 + term2 + term3 - term4;

    Angle::from_degrees(degrees).unwound()
}

pub fn nutation_in_longitude(
    solar_longitude: Angle,
    lunar_longitude: Angle,
    ascending_node: Angle,
) -> f64 {
    // Equation from Astronomical Algorithms page 144
    let term1 = (-17.2 / 3600.0) * ascending_node.radians().sin();
    let term2 = (1.32 / 3600.0) * (2.0 * solar_longitude.radians()).sin();
    let term3 = (0.23 / 3600.0) * (2.0 * lunar_longitude.radians()).sin();
    let term4 = (0.21 / 3600.0) * (2.0 * ascending_node.radians()).sin();

    term1 - term2 - term3 + term4
}

pub fn nutation_in_obliquity(
    solar_longitude: Angle,
    lunar_longitude: Angle,
    ascending_node: Angle,
) -> f64 {
    // Equation from Astronomical Algorithms page 144
    let term1 = (9.2 / 3600.0) * ascending_node.radians().cos();
    let term2 = (0.57 / 3600.0) * (2.0 * solar_longitude.radians()).cos();
    let term3 = (0.10 / 3600.0) * (2.0 * lunar_longitude.radians()).cos();
    let term4 = (0.09 / 3600.0) * (2.0 * ascending_node.radians()).cos();

    term1 + term2 + term3 - term4
}

pub fn altitude_of_celestial_body(
    observer_latitude: Angle,
    declination: Angle,
    local_hour_angle: Angle,
) -> Angle {
    // Equation from Astronomical Algorithms page 93
    let term1 = observer_latitude.radians().sin() * declination.radians().sin();
    let term2 = observer_latitude.radians().cos()
        * declination.radians().cos()
        * local_hour_angle.radians().cos();

    Angle::from_radians((term1 + term2).asin())
}

pub fn approximate_transit(longitude: Angle, sidereal_time: Angle, right_ascension: Angle) -> f64 {
    // Equation from page Astronomical Algorithms 102
    let longitude_angle = -longitude;

    ((right_ascension + longitude_angle - sidereal_time) / 360.)
        .degrees
        .normalized_to_scale(1.0)
}

/// Interpolation of a value given equidistant previous and next values and a factor equal to
/// the fraction of the interpolated point's time over the time between values.
pub fn interpolate(value: f64, previous_value: f64, next_value: f64, factor: f64) -> f64 {
    // Equation from Astronomical Algorithms page 24
    let a = value - previous_value;
    let b = next_value - value;
    let c = b - a;

    value + ((factor / 2.0) * (a + b + (factor * c)))
}

/// Interpolation of three angles, accounting for angle unwinding.
pub fn interpolate_angles(
    value: Angle,
    previous_value: Angle,
    next_value: Angle,
    factor: f64,
) -> Angle {
    // Equation from Astronomical Algorithms page 24
    let a = (value - previous_value).unwound();
    let b = (next_value - value).unwound();
    let c = b - a;

    Angle::from_degrees(
        value.degrees + ((factor / 2.0) * (a.degrees + b.degrees + (factor * c.degrees))),
    )
}

/// Gives the civil time for the given hour-angle. Returns `None` if hour-angle is not finite.
pub fn time_for_hour_angle(hour_angle: f64, date: Date) -> Option<Zoned> {
    hour_angle.is_finite().then(|| {
        &date.to_zoned(TimeZone::UTC).unwrap()
            + jiff::SignedDuration::from_secs_f64(hour_angle * 60.0 * 60.0)
    })
}

/// Julian century from date.
pub fn julian_century_from_date(date: Date) -> f64 {
    // Equation from Astronomical Algorithms page 60

    let year = date.year() as i64;
    let month = date.month() as i64;
    let day = date.day() as i64;

    let adjusted_year = if month > 2 { year } else { year - 1 };
    let adjusted_month = if month > 2 { month } else { month + 12 };
    let adjusted_day = day;

    let a = adjusted_year / 100;
    let b = 2 - a + (a / 4);

    let i0 = (adjusted_year + 4716) * 1461 / 4; // truncate((adjusted_year + 4716) * 365.25);
    let i1 = (adjusted_month + 1) * 30_6001 / 1_0000; // truncate((adjusted_month + 1) * 30.6001)

    let julian_day = (i0 + i1 + adjusted_day + b) as f64 - 1524.5;

    // Equation from Astronomical Algorithms page 163
    (julian_day - 2451545.0) / 36525.0
}

#[cfg(test)]
#[allow(clippy::excessive_precision)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use jiff::civil::date;

    #[test]
    fn calculate_mean_solar_longitude() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_solar_longitude = mean_solar_longitude(julian_century);

        assert_abs_diff_eq!(
            mean_solar_longitude.degrees,
            201.80719320670732,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_apparent_solar_longitude() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_solar_longitude = mean_solar_longitude(julian_century);
        let apparent_solar_longitude =
            apparent_solar_longitude(julian_century, mean_solar_longitude).radians();

        assert_abs_diff_eq!(
            apparent_solar_longitude,
            3.4890691820452062,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_mean_obliquity_of_the_ecliptic() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_obliq_of_ecliptic = mean_obliquity_of_the_ecliptic(julian_century);

        assert_abs_diff_eq!(
            mean_obliq_of_ecliptic.degrees,
            23.440229684413012,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_apparent_obliquity_of_the_ecliptic() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_obliq_of_ecliptic = mean_obliquity_of_the_ecliptic(julian_century);
        let apparent_obliq_of_ecliptic =
            apparent_obliquity_of_the_ecliptic(julian_century, mean_obliq_of_ecliptic);

        assert_abs_diff_eq!(
            apparent_obliq_of_ecliptic.degrees,
            23.43999110619955,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_mean_solar_anomaly() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_solar_anomaly = mean_solar_anomaly(julian_century);

        assert_abs_diff_eq!(
            mean_solar_anomaly.degrees,
            278.99396643159753,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_solar_equation_of_the_center() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_solar_anomaly = mean_solar_anomaly(julian_century);
        let solar_equation_of_center =
            solar_equation_of_the_center(julian_century, mean_solar_anomaly);

        assert_abs_diff_eq!(
            solar_equation_of_center.degrees,
            -1.897323843371985,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_mean_lunar_longitude() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_lunar_longitude = mean_lunar_longitude(julian_century);

        assert_abs_diff_eq!(
            mean_lunar_longitude.degrees,
            38.747190008209145,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_ascending_lunar_node_longitude() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let ascending_lunar_node = ascending_lunar_node_longitude(julian_century);

        assert_abs_diff_eq!(
            ascending_lunar_node.degrees,
            264.657131805429,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_mean_sidereal_time() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_sidereal_time = mean_sidereal_time(julian_century);

        assert_abs_diff_eq!(
            mean_sidereal_time.degrees,
            21.801339167752303,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_nutation_longitude() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_solar_longitude = mean_solar_longitude(julian_century);
        let mean_lunar_longitude = mean_lunar_longitude(julian_century);
        let ascending_lunar_node = ascending_lunar_node_longitude(julian_century);
        let nutation_longitude = nutation_in_longitude(
            mean_solar_longitude,
            mean_lunar_longitude,
            ascending_lunar_node,
        );

        assert_abs_diff_eq!(nutation_longitude, 0.0044525358169686564, epsilon = 1e-10);
    }

    #[test]
    fn calculate_nutation_in_obliquity() {
        let julian_century = julian_century_from_date(date(1992, 10, 13));
        let mean_solar_longitude = mean_solar_longitude(julian_century);
        let mean_lunar_longitude = mean_lunar_longitude(julian_century);
        let ascending_lunar_node = ascending_lunar_node_longitude(julian_century);
        let nutation_obliq = nutation_in_obliquity(
            mean_solar_longitude,
            mean_lunar_longitude,
            ascending_lunar_node,
        );

        assert_abs_diff_eq!(nutation_obliq, -0.000092747500292341556, epsilon = 1e-10);
    }

    #[test]
    fn calculate_altitude_of_celestial_body() {
        let observer_latitude = Angle::from_degrees(35.783333333333333);
        let declination_angle = Angle::from_degrees(21.894701414701338);
        let local_hour_angle = Angle::from_degrees(108.09275357838322);
        let altitude_of_celestial_body =
            altitude_of_celestial_body(observer_latitude, declination_angle, local_hour_angle);

        assert_abs_diff_eq!(
            altitude_of_celestial_body.degrees,
            -0.90061562155943208,
            epsilon = 1e-10
        );
    }
}
