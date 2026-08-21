use crate::*;

#[test]
fn for_kuwait_city_kuwait() {
    let coordinates = Coordinates {
        latitude: 29.370865,
        longitude: 47.979139,
    };
    let params = Parameters::new(&prominent_methods::Kuwait)
        .with_high_latitude_rule(HighLatitudeRule::MiddleOfTheNight);
    let variance = jiff::SignedDuration::from_mins(2);

    {
        let date = jiff::civil::date(2016, 1, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(5, 18, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 42, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 51, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 42, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 1, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 23, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(5, 16, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 38, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 2, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 5, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 26, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 45, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(4, 56, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 15, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(12, 1, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 19, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 47, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 4, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(4, 18, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 38, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 52, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 23, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 6, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 24, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(3, 41, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 7, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 45, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 21, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 24, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 47, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(3, 15, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(4, 49, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 46, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 20, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 43, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 13, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(3, 16, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(4, 52, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 52, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 26, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 52, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 23, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(3, 39, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 8, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 54, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 30, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 40, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(20, 6, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(4, 3, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 25, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 48, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 22, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 11, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 30, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(4, 22, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 41, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 38, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 2, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 34, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 51, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(4, 40, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 1, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 32, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 39, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 2, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 20, 0, 0).in_tz("Asia/Kuwait").unwrap();
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

        let fajr = date.at(5, 1, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 24, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 37, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 30, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(16, 49, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 11, 0, 0).in_tz("Asia/Kuwait").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
}
