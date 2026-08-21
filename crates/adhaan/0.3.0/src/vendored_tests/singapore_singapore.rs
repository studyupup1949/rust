use crate::*;

#[test]
fn for_singapore_singapore() {
    let coordinates = Coordinates {
        latitude: 1.370844612058886,
        longitude: 103.80145644060552,
    };
    let params = Parameters::new(&prominent_methods::Singapore)
        .with_high_latitude_rule(HighLatitudeRule::MiddleOfTheNight);
    let variance = jiff::SignedDuration::from_mins(2);

    {
        let date = jiff::civil::date(2020, 1, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 1, 31);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 2, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 3, 31);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 4, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 5, 31);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 6, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 7, 31);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 46, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 45, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 8, 31);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 9, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 10, 31);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 47, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 48, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 49, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 50, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 51, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 52, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 11, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 53, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 2);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 3);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 11, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 4);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 54, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 5);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 12, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 6);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 55, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 7);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 13, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 8);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 56, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 9);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 14, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 10);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 57, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 11);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 15, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 12);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 58, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 13);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 16, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 14);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 35, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 15);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 59, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 17, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 16);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 36, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 26, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 17);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 0, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 18, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 18);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 37, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 27, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 19);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 1, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 19, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 20);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 38, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 28, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 21);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 2, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 20, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 22);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 39, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 29, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 23);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 3, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 21, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 24);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 40, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 30, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 25);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 4, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 22, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 26);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 41, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 31, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 27);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 5, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 28);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 42, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 8, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 32, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 29);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 6, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 24, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 30);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 43, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 9, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 33, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
    {
        let date = jiff::civil::date(2020, 12, 31);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 44, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(7, 7, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(13, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(16, 34, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(19, 10, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 25, 0, 0).in_tz("Asia/Singapore").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
}
