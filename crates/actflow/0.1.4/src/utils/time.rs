use chrono::{DateTime, Utc};

#[allow(unused)]
pub fn time_millis() -> i64 {
    let time: DateTime<chrono::Utc> = Utc::now();
    time.timestamp_millis()
}

#[allow(unused)]
pub fn time_micros() -> i64 {
    let time: DateTime<chrono::Utc> = Utc::now();
    time.timestamp_micros()
}
