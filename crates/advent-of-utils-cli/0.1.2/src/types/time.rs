use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, Utc};
use std::{fmt::Display, sync::OnceLock, time::Duration};

use crate::error::AocError;

const AOC_START_YEAR: i32 = 2015;
/// Midnight EST
const PUZZLE_START_TIME: (u32, u32) = (0, 0);
const MAX_PUZZLE_DAY: u8 = 25;
const EST_OFFSET_SECONDS: i32 = -5 * 3600;

/// Represents the time configuration for Advent of Code puzzles
#[derive(Debug, Clone)]
pub struct AocTime {
    current_time: DateTime<FixedOffset>,
    est_offset: FixedOffset,
}

impl AocTime {
    /// Creates a new AocTime instance using the current time
    pub fn now() -> Self {
        static EST_OFFSET: OnceLock<FixedOffset> = OnceLock::new();
        let est_offset = EST_OFFSET
            .get_or_init(|| FixedOffset::east_opt(EST_OFFSET_SECONDS).expect("Invalid EST offset"));

        Self {
            current_time: Utc::now().with_timezone(est_offset),
            est_offset: *est_offset,
        }
    }

    /// Gets the current available year for Advent of Code
    pub fn current_year(&self) -> i32 {
        if self.current_time.month() == 12 {
            self.current_time.year()
        } else {
            self.current_time.year() - 1
        }
    }

    /// Gets the maximum available day for a given year
    pub fn available_day(&self, year: i32) -> u8 {
        match year {
            y if y > self.current_year() => 0,
            y if y == self.current_time.year() && self.current_time.month() == 12 => {
                self.current_time.day() as u8
            }
            _ => MAX_PUZZLE_DAY,
        }
    }

    /// Validates if a given year is valid for Advent of Code
    pub fn validate_year(&self, year: i32) -> Result<(), AocError> {
        match year {
            y if y < AOC_START_YEAR => Err(AocError::InvalidYear {
                year,
                reason: format!("Advent of Code started in {}", AOC_START_YEAR),
            }),
            y if y > self.current_year() => Err(AocError::InvalidYear {
                year,
                reason: format!("Latest available year is {}", self.current_year()),
            }),
            _ => Ok(()),
        }
    }

    /// Validates if a given date (year and day) is valid for Advent of Code
    pub fn validate_date(&self, year: i32, day: u8) -> Result<(), AocError> {
        self.validate_year(year)?;

        let max_day = self.available_day(year);
        match day {
            d if d == 0 => Err(AocError::InvalidDay {
                year,
                day,
                reason: "Day must be between 1 and 25".to_string(),
            }),
            d if d > max_day => Err(AocError::InvalidDay {
                year,
                day,
                reason: format!("Latest available day for year {} is {}", year, max_day),
            }),
            _ => Ok(()),
        }
    }

    /// Checks if a puzzle is currently available
    pub fn is_puzzle_available(&self, year: i32, day: u8) -> bool {
        self.validate_date(year, day).is_ok()
    }

    /// Gets the release time for a specific puzzle
    pub fn puzzle_release_time(&self, year: i32, day: u8) -> DateTime<FixedOffset> {
        let (hour, minute) = PUZZLE_START_TIME;
        let naive = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(year, 12, day as u32).expect("Invalid date"),
            chrono::NaiveTime::from_hms_opt(hour, minute, 0).expect("Invalid time"),
        );
        DateTime::<FixedOffset>::from_naive_utc_and_offset(naive, self.est_offset)
    }

    /// Gets the time until a puzzle becomes available
    pub fn time_until_release(&self, year: i32, day: u8) -> Option<chrono::Duration> {
        let release = self.puzzle_release_time(year, day);
        if release > self.current_time {
            Some(release.signed_duration_since(self.current_time))
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Default)]
pub struct AocDuration {
    duration: Vec<Duration>,
}

impl AocDuration {
    pub fn new(duration: Vec<Duration>) -> Self {
        Self { duration }
    }
    pub fn get_mut_time(&mut self) -> &mut Vec<Duration> {
        &mut self.duration
    }
    pub fn duration(&self) -> &Vec<Duration> {
        &self.duration
    }
    pub fn avg_time(&self) -> Option<Duration> {
        if self.duration.is_empty() {
            None
        } else {
            Some(self.duration.iter().sum::<Duration>() / self.duration.len() as u32)
        }
    }
    pub fn additional_time(&mut self, time: &mut Vec<Duration>) {
        self.duration.append(time);
    }
    pub fn duration_len(&self) -> usize {
        self.duration.len()
    }
}

impl Display for AocDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(time) = self.avg_time() {
            match time {
                time if time.as_secs() > 0 => write!(f, "{:.2} s", time.as_secs_f32()),
                time if time.as_millis() > 0 => {
                    write!(f, "{:.2} ms", time.as_micros() as f32 / 1000.0)
                }
                time if time.as_micros() > 0 => {
                    write!(f, "{:.2} μs", time.as_nanos() as f32 / 1000.0)
                }
                time => write!(f, "{} ns", time.as_nanos()),
            }
        } else {
            write!(f, "None")
        }
    }
}
