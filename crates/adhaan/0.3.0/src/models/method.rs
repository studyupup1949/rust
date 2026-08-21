use jiff::{SignedDuration, Zoned, civil::Date};

use crate::{
    Coordinates, Parameters, TimeAdjustment,
    astrolabe::{solar::SolarTime, unit::Angle},
};

fn select_safe(
    normal: Option<Zoned>,
    safe: Zoned,
    select: impl Fn(Zoned, Zoned) -> Zoned,
) -> Zoned {
    if let Some(normal) = normal {
        select(normal, safe)
    } else {
        safe
    }
}

/// Provides calculation parameters.
pub trait Method: std::fmt::Debug {
    /// Method intrinstic time adjustments.
    fn adjustments(&self) -> TimeAdjustment {
        TimeAdjustment {
            dhuhr: 1,
            ..Default::default()
        }
    }

    /// Angle for Fajr calculation.
    fn fajr_angle(&self) -> f64 {
        f64::NAN
    }

    /// Angle for Isha calculation. Can be NaN for interval based calculation.
    fn isha_angle(&self) -> f64 {
        f64::NAN
    }

    /// Angle for Maghrib calculation. Can be NaN for setting it to sunset.
    fn maghrib_angle(&self) -> f64 {
        f64::NAN
    }

    /// Interval between Maghrib and Isha, if applicable.
    ///
    /// `isha_angle()` should return NaN if it returns `None`
    fn isha_interval(&self) -> Option<u8> {
        None
    }

    /// Calculates Fajr time
    #[expect(unused_variables, reason = "used in non-default impls")]
    fn calculate_fajr(
        &self,
        parameters: &Parameters,
        solar_time: &SolarTime,
        night: SignedDuration,
        coordinates: Coordinates,
        date: Date,
    ) -> Zoned {
        select_safe(
            solar_time.time_for_solar_angle(Angle::from_degrees(-parameters.fajr_angle), false),
            solar_time.sunrise() - night.mul_f64(parameters.night_portions().0),
            Ord::max,
        )
    }

    /// Calculates Isha time
    #[expect(unused_variables, reason = "used in non-default impls")]
    fn calculate_isha(
        &self,
        parameters: &Parameters,
        solar_time: &SolarTime,
        night: SignedDuration,
        coordinates: Coordinates,
        date: Date,
    ) -> Zoned {
        select_safe(
            solar_time.time_for_solar_angle(Angle::from_degrees(-parameters.isha_angle), true),
            solar_time.sunset() + night.mul_f64(parameters.night_portions().1),
            Ord::min,
        )
    }
}

mod moonsighting_com;

/// Prominent calculation methods
pub mod prominent_methods {
    use super::*;

    pub use moonsighting_com::{
        MOONSIGHTING_COMMITTEE, MOONSIGHTING_COMMITTEE_RED_ISHA, MOONSIGHTING_COMMITTEE_WHITE_ISHA,
    };

    /// [Muslim World League](https://www.themwl.org/en)
    #[derive(Debug, PartialEq, Eq)]
    pub struct MuslimWorldLeague;

    impl Method for MuslimWorldLeague {
        fn fajr_angle(&self) -> f64 {
            18.0
        }

        fn isha_angle(&self) -> f64 {
            17.0
        }
    }

    /// Egyptian General Authority of Survey
    #[derive(Debug, PartialEq, Eq)]
    pub struct Egyptian;

    impl Method for Egyptian {
        fn fajr_angle(&self) -> f64 {
            19.5
        }

        fn isha_angle(&self) -> f64 {
            17.5
        }
    }

    /// University of Islamic Sciences, Karachi
    #[derive(Debug, PartialEq, Eq)]
    pub struct Karachi;

    impl Method for Karachi {
        fn fajr_angle(&self) -> f64 {
            18.0
        }

        fn isha_angle(&self) -> f64 {
            18.0
        }
    }

    /// Umm al-Qura University, Makkah
    #[derive(Debug, PartialEq, Eq)]
    pub struct UmmAlQura;

    impl Method for UmmAlQura {
        fn adjustments(&self) -> TimeAdjustment {
            Default::default()
        }

        fn fajr_angle(&self) -> f64 {
            18.5
        }

        fn isha_angle(&self) -> f64 {
            f64::NAN
        }

        fn isha_interval(&self) -> Option<u8> {
            Some(90)
        }

        fn calculate_isha(
            &self,
            parameters: &Parameters,
            solar_time: &SolarTime,
            _: SignedDuration,
            _: Coordinates,
            _: Date,
        ) -> Zoned {
            solar_time.sunset()
                + SignedDuration::from_mins(parameters.isha_interval.unwrap_or(90) as _)
        }
    }

    /// The Gulf Region
    #[derive(Debug, PartialEq, Eq)]
    pub struct Dubai;

    impl Method for Dubai {
        fn adjustments(&self) -> TimeAdjustment {
            TimeAdjustment {
                sunrise: -3,
                dhuhr: 3,
                asr: 3,
                maghrib: 3,
                ..Default::default()
            }
        }

        fn fajr_angle(&self) -> f64 {
            18.2
        }

        fn isha_angle(&self) -> f64 {
            18.2
        }
    }

    /// Qatar
    #[derive(Debug, PartialEq, Eq)]
    pub struct Qatar;

    impl Method for Qatar {
        fn adjustments(&self) -> TimeAdjustment {
            Default::default()
        }

        fn fajr_angle(&self) -> f64 {
            18.0
        }

        fn isha_angle(&self) -> f64 {
            f64::NAN
        }

        fn isha_interval(&self) -> Option<u8> {
            Some(90)
        }

        fn calculate_isha(
            &self,
            parameters: &Parameters,
            solar_time: &SolarTime,
            night: SignedDuration,
            coordinates: Coordinates,
            prayer_date: Date,
        ) -> Zoned {
            UmmAlQura.calculate_isha(parameters, solar_time, night, coordinates, prayer_date)
        }
    }

    /// Kuwait
    #[derive(Debug, PartialEq, Eq)]
    pub struct Kuwait;

    impl Method for Kuwait {
        fn adjustments(&self) -> TimeAdjustment {
            Default::default()
        }

        fn fajr_angle(&self) -> f64 {
            18.0
        }

        fn isha_angle(&self) -> f64 {
            17.5
        }
    }

    /// Singapore
    #[derive(Debug, PartialEq, Eq)]
    pub struct Singapore;

    impl Method for Singapore {
        fn fajr_angle(&self) -> f64 {
            20.0
        }

        fn isha_angle(&self) -> f64 {
            18.0
        }
    }

    /// Turkey
    ///
    /// Approximate of Diyanet method
    #[derive(Debug, PartialEq, Eq)]
    pub struct Turkey;

    impl Method for Turkey {
        fn adjustments(&self) -> TimeAdjustment {
            TimeAdjustment {
                sunrise: -7,
                dhuhr: 5,
                asr: 4,
                maghrib: 7,
                ..Default::default()
            }
        }

        fn fajr_angle(&self) -> f64 {
            18.0
        }

        fn isha_angle(&self) -> f64 {
            17.0
        }
    }

    /// Institute of Geophysics, University of Tehran
    #[derive(Debug, PartialEq, Eq)]
    pub struct Tehran;

    impl Method for Tehran {
        fn adjustments(&self) -> TimeAdjustment {
            Default::default()
        }

        fn fajr_angle(&self) -> f64 {
            17.7
        }

        fn isha_angle(&self) -> f64 {
            14.0
        }

        fn maghrib_angle(&self) -> f64 {
            4.5
        }
    }

    /// ISNA
    #[derive(Debug, PartialEq, Eq)]
    pub struct NorthAmerica;

    impl Method for NorthAmerica {
        fn fajr_angle(&self) -> f64 {
            15.0
        }

        fn isha_angle(&self) -> f64 {
            15.0
        }
    }
}
