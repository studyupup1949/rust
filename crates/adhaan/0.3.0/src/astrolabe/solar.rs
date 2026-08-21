use jiff::{Zoned, civil::Date};

use crate::{
    Coordinates,
    astrolabe::{ops, unit::Angle},
};

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct SolarCoordinates {
    /// The declination of the sun, the angle between the rays of the Sun and
    /// the plane of the Earth's equator.
    declination: Angle,

    /// Right ascension of the Sun, the angular distance on the celestial equator
    /// from the vernal equinox to the hour circle.
    right_ascension: Angle,

    /// Apparent sidereal time, the hour angle of the vernal equinox.
    apparent_sidereal_time: Angle,
}

impl SolarCoordinates {
    fn new(date: Date) -> Self {
        let julian_century = ops::julian_century_from_date(date);
        let mean_solar_longitude = ops::mean_solar_longitude(julian_century);
        let mean_lunar_longitude = ops::mean_lunar_longitude(julian_century);
        let ascending_lunar_node = ops::ascending_lunar_node_longitude(julian_century);
        let apparent_solar_longitude =
            ops::apparent_solar_longitude(julian_century, mean_solar_longitude).radians();

        let mean_sidereal_time = ops::mean_sidereal_time(julian_century);
        let nutation_longitude = ops::nutation_in_longitude(
            mean_solar_longitude,
            mean_lunar_longitude,
            ascending_lunar_node,
        );
        let nutation_obliq = ops::nutation_in_obliquity(
            mean_solar_longitude,
            mean_lunar_longitude,
            ascending_lunar_node,
        );

        let mean_obliq_ecliptic = ops::mean_obliquity_of_the_ecliptic(julian_century);
        let apparent_obliq_ecliptic =
            ops::apparent_obliquity_of_the_ecliptic(julian_century, mean_obliq_ecliptic).radians();

        // Equation from Astronomical Algorithms page 165
        let declination = Angle::from_radians(
            (apparent_obliq_ecliptic.sin() * apparent_solar_longitude.sin()).asin(),
        );

        // Equation from Astronomical Algorithms page 165
        let right_ascension = Angle::from_radians(
            (apparent_obliq_ecliptic.cos() * apparent_solar_longitude.sin())
                .atan2(apparent_solar_longitude.cos()),
        )
        .unwound();

        // Equation from Astronomical Algorithms page 88
        let apparent_sidereal_time = Angle::from_degrees(
            mean_sidereal_time.degrees
                + ((nutation_longitude * 3600.0)
                    * Angle::from_degrees(mean_obliq_ecliptic.degrees + nutation_obliq)
                        .radians()
                        .cos())
                    / 3600.0,
        );

        SolarCoordinates {
            declination,
            right_ascension,
            apparent_sidereal_time,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SolarTime {
    date: Date,
    observer: Coordinates,
    today: SolarCoordinates,
    yesterday: SolarCoordinates,
    tomorrow: SolarCoordinates,
    approx_transit: f64,
}

impl SolarTime {
    pub(crate) const HORIZON_ALTITUDE: Angle = Angle::from_degrees(-5. / 6.);

    pub fn new(date: Date, coordinates: Coordinates) -> Result<SolarTime, Error> {
        // All calculation need to occur at 0h0m UTC
        let today = SolarCoordinates::new(date);
        let yesterday = SolarCoordinates::new(date.yesterday().expect("sensible day"));
        let tomorrow = SolarCoordinates::new(date.tomorrow().expect("sensible day"));
        let approx_transit = ops::approximate_transit(
            coordinates.longitude_angle(),
            today.apparent_sidereal_time,
            today.right_ascension,
        );

        let solar_time = SolarTime {
            date,
            observer: coordinates,
            today,
            yesterday,
            tomorrow,
            approx_transit,
        };

        if !solar_time
            .corrected_hour_angle(SolarTime::HORIZON_ALTITUDE, false)
            .is_normal()
        {
            Err(Error::SunWontRise)
        } else if !solar_time
            .corrected_hour_angle(SolarTime::HORIZON_ALTITUDE, true)
            .is_normal()
        {
            Err(Error::SunWontSet)
        } else {
            Ok(solar_time)
        }
    }

    pub fn time_for_solar_angle(&self, angle: Angle, after_transit: bool) -> Option<Zoned> {
        ops::time_for_hour_angle(self.corrected_hour_angle(angle, after_transit), self.date)
    }

    pub fn afternoon(&self, shadow_length: f64) -> Zoned {
        let absolute_degrees = (self.observer.latitude - self.today.declination.degrees).abs();
        let tangent = Angle::from_degrees(absolute_degrees);
        let inverse = shadow_length + tangent.radians().tan();
        let angle = Angle::from_radians((1.0 / inverse).atan());

        self.time_for_solar_angle(angle, true)
            .expect("sun to reach the angle")
    }

    pub fn sunrise(&self) -> Zoned {
        self.time_for_solar_angle(SolarTime::HORIZON_ALTITUDE, false)
            .expect("sun to reach the angle")
    }

    pub fn sunset(&self) -> Zoned {
        self.time_for_solar_angle(SolarTime::HORIZON_ALTITUDE, true)
            .expect("sun to reach the angle")
    }

    pub fn transit(&self) -> Zoned {
        ops::time_for_hour_angle(self.corrected_transit_hour_angle(), self.date)
            .expect("sun to definitely reach the angle")
    }

    /// The time at which the sun is at its highest point in the sky (in universal time)
    fn corrected_transit_hour_angle(&self) -> f64 {
        let &Self {
            observer,
            today,
            yesterday,
            tomorrow,
            approx_transit,
            ..
        } = self;

        // Equation from page Astronomical Algorithms 102
        let longitude_angle = -observer.longitude_angle();
        let plane_angle = Angle::from_degrees(
            today.apparent_sidereal_time.degrees + (360.985647 * approx_transit),
        )
        .unwound();
        let interpolated_angles = ops::interpolate_angles(
            today.right_ascension,
            yesterday.right_ascension,
            tomorrow.right_ascension,
            approx_transit,
        )
        .unwound();
        let angles = (plane_angle - longitude_angle - interpolated_angles).quadrant_shifted();
        let angle_delta = angles / -360.;

        (approx_transit + angle_delta.degrees) * 24.0
    }

    fn corrected_hour_angle(&self, angle: Angle, after_transit: bool) -> f64 {
        let &Self {
            observer,
            today,
            yesterday,
            tomorrow,
            approx_transit,
            ..
        } = self;

        // Equation from page Astronomical Algorithms 102
        let longitude_angle = -observer.longitude_angle();
        let term1 = angle.radians().sin()
            - (observer.latitude_angle().radians().sin() * today.declination.radians().sin());
        let term2 = observer.latitude_angle().radians().cos() * today.declination.radians().cos();
        let term_angle = Angle::from_radians((term1 / term2).acos());

        let adjusted_approx_transit = if after_transit {
            approx_transit + (term_angle.degrees / 360.0)
        } else {
            approx_transit - (term_angle.degrees / 360.0)
        };

        let plane_angle = Angle::from_degrees(
            today.apparent_sidereal_time.degrees + (360.985647 * adjusted_approx_transit),
        )
        .unwound();
        let interpolated_angles = ops::interpolate_angles(
            today.right_ascension,
            yesterday.right_ascension,
            tomorrow.right_ascension,
            adjusted_approx_transit,
        )
        .unwound();
        let declination_angle = Angle::from_degrees(ops::interpolate(
            today.declination.degrees,
            yesterday.declination.degrees,
            tomorrow.declination.degrees,
            adjusted_approx_transit,
        ));
        let adjusted_angles = plane_angle - longitude_angle - interpolated_angles;
        let celestial_body_altitude = ops::altitude_of_celestial_body(
            observer.latitude_angle(),
            declination_angle,
            adjusted_angles,
        );
        let term3 = (celestial_body_altitude - angle).degrees;
        let term4 = 360.0
            * declination_angle.radians().cos()
            * observer.latitude_angle().radians().cos()
            * adjusted_angles.radians().sin();
        let angle_delta = term3 / term4;

        (adjusted_approx_transit + angle_delta) * 24.0
    }
}

#[derive(Debug)]
pub enum Error {
    SunWontRise,
    SunWontSet,
}

#[cfg(test)]
#[allow(clippy::excessive_precision)]
mod tests {
    use super::*;
    use crate::astrolabe::ops;
    use approx::assert_abs_diff_eq;
    use jiff::{
        Unit,
        civil::{date, datetime, time},
        tz::TimeZone,
    };

    #[test]
    fn solar_coordinates() {
        let solar = SolarCoordinates::new(date(1992, 10, 13));

        assert_abs_diff_eq!(
            solar.declination.degrees,
            -7.7850685152648795,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            solar.right_ascension.degrees,
            198.38082214251881,
            epsilon = 1e-10
        );
        assert_abs_diff_eq!(
            solar.right_ascension.unwound().degrees,
            198.38082214251881,
            epsilon = 1e-10
        );
    }

    #[test]
    fn calculate_solar_time() {
        let coordinates = {
            let latitude = 35.0 + 47.0 / 60.0;
            let longitude = -78.0 - 39.0 / 60.0;
            Coordinates {
                latitude,
                longitude,
            }
        };
        let date = date(2015, 7, 12);
        let solar = SolarTime::new(date, coordinates).unwrap();

        let transit_calc = ops::time_for_hour_angle(solar.corrected_transit_hour_angle(), date)
            .unwrap()
            .round(Unit::Second)
            .unwrap();
        let sunrise_calc = solar
            .time_for_solar_angle(SolarTime::HORIZON_ALTITUDE, false)
            .unwrap()
            .round(Unit::Second)
            .unwrap();
        let sunset_calc = solar
            .time_for_solar_angle(SolarTime::HORIZON_ALTITUDE, true)
            .unwrap()
            .round(Unit::Second)
            .unwrap();

        let transit_fact = datetime(2015, 7, 12, 17, 20, 14, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let sunrise_fact = datetime(2015, 7, 12, 10, 7, 54, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();
        let sunset_fact = datetime(2015, 7, 13, 0, 32, 16, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap();

        assert_eq!(transit_calc, transit_fact);
        assert_eq!(sunrise_calc, sunrise_fact);
        assert_eq!(sunset_calc, sunset_fact);
    }

    #[test]
    fn calculate_time_for_solar_angle() {
        let coordinates = {
            let latitude = 35.0 + 47.0 / 60.0;
            let longitude = -78.0 - 39.0 / 60.0;
            Coordinates {
                latitude,
                longitude,
            }
        };
        let date = date(2015, 7, 12);
        let solar = SolarTime::new(date, coordinates).unwrap();
        let angle = Angle::from_degrees(-6.0);
        let twilight_start_calc = solar
            .time_for_solar_angle(angle, false)
            .unwrap()
            .time()
            .round(Unit::Second)
            .unwrap();
        let twilight_end_calc = solar
            .time_for_solar_angle(angle, true)
            .unwrap()
            .time()
            .round(Unit::Second)
            .unwrap();

        let twilight_start_fact = time(9, 38, 21, 0);
        let twilight_end_fact = time(1, 1, 46, 0);

        assert_eq!(twilight_start_calc, twilight_start_fact);
        assert_eq!(twilight_end_calc, twilight_end_fact);
    }
}
