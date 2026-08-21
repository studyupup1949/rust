use crate::Year;

/// Get the time and date when the first puzzle will unlock for the given year.
pub fn get_puzzle_unlock_time(for_year: Year) -> chrono::DateTime<chrono::Utc> {
    use chrono::{NaiveDate, TimeZone};
    use chrono_tz::US::Eastern;

    let unlock_dt = NaiveDate::from_ymd_opt(for_year.into(), 12, 1)
        .unwrap()
        .and_time(Default::default());

    Eastern
        .from_local_datetime(&unlock_dt)
        .unwrap()
        .with_timezone(&chrono::Utc)
}
