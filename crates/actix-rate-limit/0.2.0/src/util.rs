use std::time::{Duration, SystemTime};

pub type Seconds = u64;

/// Seconds elapsed since UNIX epoch for **Now**
pub fn seconds_elapsed() -> Seconds {
    SystemTime::UNIX_EPOCH
        .elapsed()
        .as_ref()
        .map(Duration::as_secs)
        .unwrap()
}

/// Seconds elapsed since UNIX epoch for the **Next Hour**
pub fn seconds_elapsed_for_next_hour() -> Seconds {
    const ONE_HOUR: Seconds = 3600;
    let seconds = seconds_elapsed();
    seconds - (seconds % ONE_HOUR) + ONE_HOUR
}

/// Get the clock hour, in the range 0..24
pub fn current_hour() -> u8 {
    time::OffsetDateTime::now_utc().hour()
}
