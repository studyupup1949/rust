use crate::*;

#[test]
fn for_ankara_turkey() {
    let coordinates = Coordinates {
        latitude: 39.939382,
        longitude: 32.819713,
    };
    let params = Parameters::new(&prominent_methods::Turkey)
        .with_high_latitude_rule(HighLatitudeRule::MiddleOfTheNight);
    let variance = jiff::SignedDuration::from_mins(2);

    {
        let date = jiff::civil::date(2019, 1, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 33, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(8, 3, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 20, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 40, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 5, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 1, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 33, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(8, 2, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 32, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 54, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 17, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 2, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 25, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 51, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 49, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 14, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 34, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 2, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 11, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 35, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 2, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 30, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 49, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 3, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 53, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 46, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 3, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 54, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 1, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 4, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 1, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 27, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 58, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 39, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 4, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 35, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 5, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 33, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 56, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 5, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 7, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 43, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 49, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(21, 18, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 5, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 44, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 27, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 43, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 3, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(21, 39, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 6, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 24, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 15, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 48, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 17, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(22, 0, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 6, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 17, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 12, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 51, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 26, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(22, 12, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 7, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 21, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 16, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 55, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 28, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(22, 15, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 7, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 35, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 25, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 0, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 56, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 24, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(22, 6, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 8, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(3, 58, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 40, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 0, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 53, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(20, 10, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(21, 45, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 8, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 18, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 53, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 58, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 46, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 54, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(21, 22, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 9, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 40, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 9, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 29, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 52, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 9, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 57, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 22, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 49, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 6, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 10, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 14, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 37, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 44, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 2, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 40, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 58, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 10, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 40, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 46, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 18, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 36, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 11, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 10, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 37, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 28, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 55, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 14, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 11, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 59, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 26, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 38, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 16, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 40, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 2, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 12, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 14, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 43, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 42, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 10, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 31, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 55, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2019, 12, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(6, 25, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 56, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 48, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 10, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 31, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 56, 0, 0).in_tz("Europe/Istanbul").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
}
