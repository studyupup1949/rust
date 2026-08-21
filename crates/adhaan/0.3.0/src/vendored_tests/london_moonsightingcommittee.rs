use crate::*;

#[test]
fn for_london_moonsightingcommittee() {
    let coordinates = Coordinates {
        latitude: 51.507194,
        longitude: -0.116711,
    };
    let params = Parameters::new(&prominent_methods::MOONSIGHTING_COMMITTEE)
        .with_high_latitude_rule(HighLatitudeRule::MiddleOfTheNight);
    let variance = jiff::SignedDuration::from_mins(2);

    {
        let date = jiff::civil::date(2016, 1, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 25, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(8, 6, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 9, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 15, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(16, 5, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(17, 38, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 2, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 2, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 40, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 19, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 1, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(16, 52, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 18, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 3, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 10, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 45, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 18, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 49, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 44, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 3, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 4, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 59, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 35, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(17, 32, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 37, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 49, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 5, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 47, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 32, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(18, 5, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 27, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(21, 32, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 6, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(2, 55, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(4, 49, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(18, 31, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(21, 12, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(22, 23, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 7, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(2, 51, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(4, 48, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(18, 40, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(21, 24, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(22, 38, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 8, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 37, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 25, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(18, 23, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 51, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(21, 56, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 9, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 35, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 14, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(17, 39, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 49, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 58, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 10, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 55, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 42, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 40, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 57, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 11, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 18, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 55, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 49, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 43, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(16, 36, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 0, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2016, 12, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 4, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 44, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 55, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 7, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrThaani);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(15, 58, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(17, 29, 0, 0).in_tz("Europe/London").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
}
