use jiff::{ToSpan, Zoned, civil::Date};

use crate::{Coordinates, Parameters, PolarCircleResolutionError, Prayer, TimeOutsideOfDate};

/// A data structure that holds the timings of prayers.
#[derive(PartialEq, Debug, Clone)]
pub struct PrayerTimes {
    qiyam_yesterday: Zoned,
    fajr: Zoned,
    sunrise: Zoned,
    dhuhr: Zoned,
    asr_awwal: Zoned,
    asr_thaani: Zoned,
    maghrib: Zoned,
    isha: Zoned,
    middle_of_night: Zoned,
    qiyam: Zoned,
    fajr_tomorrow: Zoned,

    coordinates: Coordinates,
    date: Date,
    parameters: Parameters,
}

impl PrayerTimes {
    /// Calculates prayer times.
    ///
    /// # Errors
    ///
    /// This function may error if polar circle resolution fails.
    pub fn calculate(
        date: Date,
        coordinates: Coordinates,
        parameters: Parameters,
    ) -> Result<PrayerTimes, PolarCircleResolutionError> {
        let crate::models::polar_circle_resolution::PolarCircleResolution {
            date: _resolved_date,
            coordinates,
            today,
            yesterday,
            tomorrow,
        } = parameters
            .polar_circle_resolver
            .resolve(date, coordinates)?;

        let today_sunrise = today.sunrise();
        let today_sunset = today.sunset();
        let yesterday_sunset = yesterday.sunset();

        let tonight = today_sunrise.duration_since(&yesterday_sunset);
        let next_night = tomorrow.sunrise().duration_since(&today_sunset);

        let fajr =
            &parameters
                .method
                .calculate_fajr(&parameters, &today, tonight, coordinates, date)
                + parameters.time_adjustments(Prayer::Fajr).minutes();

        let sunrise = &today_sunrise + parameters.time_adjustments(Prayer::Sunrise).minutes();

        let dhuhr = &today.transit() + parameters.time_adjustments(Prayer::Dhuhr).minutes();

        let asr_awwal =
            &today.afternoon(1.0) + parameters.time_adjustments(Prayer::AsrAwwal).minutes();

        let asr_thaani =
            &today.afternoon(2.0) + parameters.time_adjustments(Prayer::AsrThaani).minutes();

        let maghrib = parameters
            .maghrib_angle
            .is_finite()
            .then(|| {
                today.time_for_solar_angle(
                    crate::astrolabe::unit::Angle::from_degrees(-parameters.maghrib_angle),
                    true,
                )
            })
            .flatten()
            .unwrap_or(today_sunset)
            + parameters.time_adjustments(Prayer::Maghrib).minutes();

        let isha =
            &parameters
                .method
                .calculate_isha(&parameters, &today, next_night, coordinates, date)
                + parameters.time_adjustments(Prayer::Isha).minutes();

        let fajr_tomorrow = &parameters.method.calculate_fajr(
            &parameters,
            &tomorrow,
            next_night,
            coordinates,
            date.tomorrow().expect("sensible date"),
        ) + parameters.time_adjustments(Prayer::Fajr).minutes();

        let next_night_short = fajr_tomorrow.duration_since(&maghrib);
        let middle_of_night = &maghrib + next_night_short / 2;
        let last_third_of_night = &maghrib + next_night_short * 2 / 3;

        let maghrib_yesterday =
            &yesterday_sunset + parameters.time_adjustments(Prayer::Maghrib).minutes();

        let last_night_short = fajr.duration_since(&maghrib_yesterday);
        let last_third_of_last_night = &maghrib_yesterday + last_night_short * 2 / 3;

        Ok(PrayerTimes {
            qiyam_yesterday: last_third_of_last_night,
            fajr,
            sunrise,
            dhuhr,
            asr_awwal,
            asr_thaani,
            maghrib,
            isha,
            middle_of_night,
            qiyam: last_third_of_night,
            fajr_tomorrow,
            coordinates,
            date,
            parameters,
        })
    }

    /// Returns time of the prayer.
    pub fn time_of(&self, prayer: Prayer) -> Zoned {
        use Prayer::*;
        use jiff::{RoundMode, Unit, ZonedRound};
        match prayer {
            QiyamYesterday => self
                .qiyam_yesterday
                .round(ZonedRound::from(Unit::Minute).mode(RoundMode::Trunc))
                .unwrap(),
            Fajr => self
                .fajr
                .round(ZonedRound::from(Unit::Minute).mode(RoundMode::Trunc))
                .unwrap(),
            Sunrise => self
                .sunrise
                .round(ZonedRound::from(Unit::Minute).mode(RoundMode::Trunc))
                .unwrap(),
            Dhuhr => self.dhuhr.round(Unit::Minute).unwrap(),
            AsrAwwal => self.asr_awwal.round(Unit::Minute).unwrap(),
            AsrThaani => self.asr_thaani.round(Unit::Minute).unwrap(),
            Maghrib => self.maghrib.round(Unit::Minute).unwrap(),
            Isha => self.isha.round(Unit::Minute).unwrap(),
            Qiyam => self
                .qiyam
                .round(ZonedRound::from(Unit::Minute).mode(RoundMode::Trunc))
                .unwrap(),
        }
    }

    /// Returns the current prayer at given time in the date.
    ///
    /// # Errors
    ///
    /// This function errors if the queried time is outside of the day for which
    /// this [`PrayerTimes`] has been calculated.
    pub fn prayer_at(&self, time: &Zoned) -> Result<Prayer, TimeOutsideOfDate> {
        if time >= self.fajr_tomorrow {
            Err(TimeOutsideOfDate::Tomorrow)
        } else if time >= self.qiyam {
            Ok(Prayer::Qiyam)
        } else if time >= self.isha {
            Ok(Prayer::Isha)
        } else if time >= self.maghrib {
            Ok(Prayer::Maghrib)
        } else if time >= self.asr_thaani {
            Ok(Prayer::AsrThaani)
        } else if time >= self.asr_awwal {
            Ok(Prayer::AsrAwwal)
        } else if time >= self.dhuhr {
            Ok(Prayer::Dhuhr)
        } else if time >= self.sunrise {
            Ok(Prayer::Sunrise)
        } else if time >= self.fajr {
            Ok(Prayer::Fajr)
        } else if time >= self.qiyam_yesterday {
            Ok(Prayer::QiyamYesterday)
        } else {
            Err(TimeOutsideOfDate::Yesterday)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prominent_methods;

    use jiff::{
        civil::{date, datetime},
        tz::TimeZone,
    };

    #[test]
    fn prayer_at_returns_correct_prayer() {
        // Given the above DateTime, the Fajr prayer is at 2015-07-12T08:42:00Z
        let local_date = date(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica);
        let coordinates = Coordinates {
            latitude: 35.7750,
            longitude: -78.6336,
        };
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();

        for (prayer, hour, minute) in [
            (Prayer::QiyamYesterday, 6, 00),
            (Prayer::Fajr, 9, 0),
            (Prayer::Sunrise, 11, 0),
            (Prayer::Dhuhr, 19, 0),
            (Prayer::AsrAwwal, 21, 10),
            (Prayer::AsrThaani, 22, 25),
            (Prayer::Maghrib, 24, 35),
            (Prayer::Isha, 26, 0),
            (Prayer::Qiyam, 30, 0),
        ] {
            let current_prayer_time =
                &local_date.to_zoned(TimeZone::UTC).unwrap() + hour.hours().minutes(minute);

            assert_eq!(times.prayer_at(&current_prayer_time), Ok(prayer));
        }
    }

    #[test]
    fn calculate_times_for_moonsighting_method() {
        let date = date(2016, 1, 31);
        let params = Parameters::new(&prominent_methods::MOONSIGHTING_COMMITTEE);
        let coordinates = {
            let longitude = -78.6336;
            Coordinates {
                latitude: 35.7750,
                longitude,
            }
        };
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // fajr    = 2016-01-31 10:48:00 UTC
        // sunrise = 2016-01-31 12:16:00 UTC
        // dhuhr   = 2016-01-31 17:33:00 UTC
        // asr     = 2016-01-31 20:20:00 UTC
        // maghrib = 2016-01-31 22:43:00 UTC
        // isha    = 2016-02-01 00:05:00 UTC
        assert_eq!(
            schedule.time_of(Prayer::Fajr),
            datetime(2016, 1, 31, 10, 48, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise),
            datetime(2016, 1, 31, 12, 15, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr),
            datetime(2016, 1, 31, 17, 33, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrAwwal),
            datetime(2016, 1, 31, 20, 20, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib),
            datetime(2016, 1, 31, 22, 43, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Isha),
            datetime(2016, 2, 1, 0, 5, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
    }

    #[test]
    fn calculate_times_for_moonsighting_method_with_high_latitude() {
        let date = date(2016, 1, 1);
        let params = Parameters::new(&prominent_methods::MOONSIGHTING_COMMITTEE);
        let coordinates = Coordinates {
            latitude: 59.9094,
            longitude: 10.7349,
        };
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // fajr    = 2016-01-01 06:34:00 UTC
        // sunrise = 2016-01-01 08:19:00 UTC
        // dhuhr   = 2016-01-01 11:25:00 UTC
        // asr     = 2016-01-01 12:36:00 UTC
        // maghrib = 2016-01-01 14:25:00 UTC
        // isha    = 2016-01-01 16:02:00 UTC
        assert_eq!(
            schedule.time_of(Prayer::Fajr),
            datetime(2016, 1, 1, 6, 33, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise),
            datetime(2016, 1, 1, 8, 18, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr),
            datetime(2016, 1, 1, 11, 25, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrThaani),
            datetime(2016, 1, 1, 12, 36, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib),
            datetime(2016, 1, 1, 14, 25, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
        assert_eq!(
            schedule.time_of(Prayer::Isha),
            datetime(2016, 1, 1, 16, 2, 0, 0)
                .to_zoned(TimeZone::UTC)
                .unwrap(),
        );
    }
}
