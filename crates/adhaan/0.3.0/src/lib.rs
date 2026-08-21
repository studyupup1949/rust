//! An Islamic prayer time calculator based on the [Adhan](https://github.com/batoulapps/Adhan)
//! library by Batoul Apps.
//!
//! It is an attempt to make an ergonomic Rust port of the library from Batoul Apps, and attempts
//! to be as unopinionated and flexible as possible.
//!
//! ## Example
//!
//! ```
//! use adhaan::*;
//!
//! let new_york_city = Coordinates { latitude: 40.7128, longitude: -74.0059 };
//! let date = jiff::civil::date(2019, 1, 25);
//! let params = Parameters::new(&prominent_methods::NorthAmerica);
//! let prayers = PrayerTimes::calculate(date, new_york_city, params).unwrap();
//! ```

#![forbid(unsafe_code, future_incompatible, deprecated_in_future)]
#![deny(unused, missing_docs)]

mod astrolabe;
mod models;
mod schedule;

pub use astrolabe::qiblah::qiblah_direction;
pub use models::{
    Coordinates, HighLatitudeRule, Method, Parameters, PolarCircleResolutionError,
    PolarCircleResolver, Prayer, TimeAdjustment, TimeOutsideOfDate, prominent_methods,
};
pub use schedule::PrayerTimes;

#[cfg(test)]
mod tests {
    use jiff::{
        civil::{date, datetime},
        tz::TimeZone,
    };

    use super::*;

    #[test]
    fn calculate_prayer_times_raleigh_usa() {
        let local_date = date(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica);
        let coordinates = {
            let longitude = -78.6336;
            Coordinates {
                latitude: 35.7750,
                longitude,
            }
        };
        let schedule = PrayerTimes::calculate(local_date, coordinates, params).unwrap();

        assert_eq!(
            schedule.time_of(Prayer::Fajr),
            datetime(2015, 7, 12, 8, 42, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise),
            datetime(2015, 7, 12, 10, 7, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr),
            datetime(2015, 7, 12, 17, 21, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrThaani),
            datetime(2015, 7, 12, 22, 22, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib),
            datetime(2015, 7, 13, 0, 32, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Isha),
            datetime(2015, 7, 13, 1, 57, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
    }

    #[test]
    fn calculate_prayer_times_rajshahi_bd() {
        let local_date = date(2021, 5, 24);
        let params = Parameters::new(&prominent_methods::Karachi);
        let coordinates = Coordinates {
            latitude: 24.383144,
            longitude: 88.583183,
        };
        let schedule = PrayerTimes::calculate(local_date, coordinates, params).unwrap();

        assert_eq!(
            schedule.time_of(Prayer::Fajr),
            datetime(2021, 5, 23, 21, 53, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise),
            datetime(2021, 5, 23, 23, 18, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr),
            datetime(2021, 5, 24, 6, 4, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrThaani),
            datetime(2021, 5, 24, 10, 43, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib),
            datetime(2021, 5, 24, 12, 46, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        // assert_eq!(
        //     schedule.time_of(Prayer::Isha),
        //     datetime(2021, 5, 24, 14, 12, 0, 0)
        //         .to_zoned(TimeZone::UTC)
        //         .unwrap(),
        // );
    }

    #[test]
    fn calculate_qiyam_times() {
        let date = date(2015, 7, 12);
        // let qiyam_date  = date(2015, 7, 13);
        let params = Parameters::new(&prominent_methods::NorthAmerica);
        let coordinates = {
            let longitude = -78.6336;
            Coordinates {
                latitude: 35.7750,
                longitude,
            }
        };
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // Today's Maghrib: 2015-07-13T00:32:00Z
        // Tomorrow's Fajr: 2015-07-13T08:43:00Z
        // Middle of Night: 2015-07-13T04:38:00Z
        // Last Third     : 2015-07-13T05:59:00Z
        assert_eq!(
            schedule.time_of(Prayer::Qiyam),
            datetime(2015, 7, 13, 5, 59, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
    }
}

#[cfg(test)]
mod vendored_tests;
