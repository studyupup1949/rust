use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::Hj212Error;

/// Parse HJ212 DataTime (commonly: YYYYMMDDhhmmss) into UTC.
pub fn parse_datatime_to_utc(s: &str) -> Result<DateTime<Utc>, Hj212Error> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y%m%d%H%M%S").map_err(|_| Hj212Error::InvalidDataTime)?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}
