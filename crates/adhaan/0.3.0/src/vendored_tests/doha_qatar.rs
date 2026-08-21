use crate::*;

#[test]
fn for_doha_qatar() {
    let coordinates = Coordinates {
        latitude: 25.283897,
        longitude: 51.52877,
    };
    let params = Parameters::new(&prominent_methods::Qatar)
        .with_high_latitude_rule(HighLatitudeRule::MiddleOfTheNight);
    let variance = jiff::SignedDuration::from_mins(2);

    {
        let date = jiff::civil::date(2016, 1, 1);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        let fajr = date.at(4, 58, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 19, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 37, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 35, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(16, 55, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 25, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(4, 59, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 17, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 47, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 56, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 18, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 48, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(4, 41, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 57, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 46, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 7, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 36, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 6, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(4, 8, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 25, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 38, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 7, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 51, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 21, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(3, 36, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(4, 58, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 31, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 0, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 5, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 35, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(3, 15, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(4, 43, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 32, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 56, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 20, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 50, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(3, 18, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(4, 47, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 38, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 1, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 28, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 58, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(3, 37, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 1, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 40, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 7, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(18, 19, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 49, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(3, 56, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 15, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 34, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(15, 3, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 53, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(19, 23, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(4, 10, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 26, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 23, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 47, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(17, 20, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 50, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(4, 24, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(5, 42, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 17, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 29, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(16, 53, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 23, 0, 0).in_tz("Asia/Riyadh").unwrap();
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

        let fajr = date.at(4, 42, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_fajr = schedule.time_of(Prayer::Fajr);
        assert!(
            (&fajr - variance..=&fajr + variance).contains(&scheduled_fajr),
            "expected = {}\nfound = {}",
            fajr,
            scheduled_fajr,
        );

        let sunrise = date.at(6, 3, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_sunrise = schedule.time_of(Prayer::Sunrise);
        assert!(
            (&sunrise - variance..=&sunrise + variance).contains(&scheduled_sunrise),
            "expected = {}\nfound = {}",
            sunrise,
            scheduled_sunrise,
        );

        let dhuhr = date.at(11, 23, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_dhuhr = schedule.time_of(Prayer::Dhuhr);
        assert!(
            (&dhuhr - variance..=&dhuhr + variance).contains(&scheduled_dhuhr),
            "expected = {}\nfound = {}",
            dhuhr,
            scheduled_dhuhr,
        );

        let asr = date.at(14, 23, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_asr = schedule.time_of(Prayer::AsrAwwal);
        assert!(
            (&asr - variance..=&asr + variance).contains(&scheduled_asr),
            "expected = {}\nfound = {}",
            asr,
            scheduled_asr,
        );

        let maghrib = date.at(16, 43, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_maghrib = schedule.time_of(Prayer::Maghrib);
        assert!(
            (&maghrib - variance..=&maghrib + variance).contains(&scheduled_maghrib),
            "expected = {}\nfound = {}",
            maghrib,
            scheduled_maghrib,
        );

        let isha = date.at(18, 13, 0, 0).in_tz("Asia/Riyadh").unwrap();
        let scheduled_isha = schedule.time_of(Prayer::Isha);
        assert!(
            (&isha - variance..=&isha + variance).contains(&scheduled_isha),
            "expected = {}\nfound = {}",
            isha,
            scheduled_isha,
        );
    }
}
