//! This module provides miscellaneous functions, structs, enums and any other related helper elements
//! not related to the other business-logic elements.
//! 
//! Here is a non-exhaustive list of available elements in the module :
//! - [`get_current_date`]
//! - [`process_path`]
//! - [`DirContent`]
//! - ...

use std::collections::BTreeMap;

use chrono::Local;
use regex::Regex;

use crate::config::Config;

/// # Purpose
/// Returns a formatted YYYY-MM-DD \[HH-MM-SS] date.
/// Based on [`chrono`] crate.
/// # Arguments
/// - `hours_minutes_secs` specifies if hours, minutes and seconds are included
/// - `date_sep`: defines the separator between day elements
/// - `hour_sep`: defines the separator between hours elements
pub fn get_current_date(hours_minutes_secs: bool, date_sep: &str, hour_sep: &str) -> String {
    let fmt = if hours_minutes_secs {
        format!("%Y{}%m{}%d %H{}%M{}%S", date_sep, date_sep, hour_sep, hour_sep)
    } else {
        format!("%Y{}%m{}%d", date_sep, date_sep)
    };
    Local::now()
    .format(&fmt)
    .to_string()
}

/// # Purpose
/// Returns the provided path with the actual value of referencd environment variables.
///
/// Those variables are referenced in a UNIX way (with a `$` prefixing each variable name).
/// # Arguments
/// - `path`: the path containing environment variables references
pub fn process_path(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut res = path.to_string();
    if path.contains("$") {
        let re = Regex::new(r"\$([a-zA-Z_]+)")?;
        for caps in re.captures_iter(path) {
            let var_value = std::env::var(&caps[1])?;
            res = res.replace(&caps[0], &var_value)
        }
        Ok(res)
    } else {
        Ok(path.to_string())
    }
}

/// # Purpose
/// Sets the actual path of the user-defined path of the history logs.
/// # Arguments
/// - `config`: [Config] object to be updated
pub fn config_init(config: &mut Config) -> Result<(), Box<dyn std::error::Error>> {
    config.history.path = process_path(&config.history.path)?;
    config.history.tmp = process_path(&config.history.tmp)?;
    config.conversion.directory_path = process_path(&config.conversion.directory_path)?;
    config.conversion.error_path = process_path(&config.conversion.error_path)?;
    Ok(())
}

pub mod serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use chrono::{Duration, NaiveDate, NaiveTime};

    pub fn parse_date<'de, D>(deserialiser: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>
    {
        let s: String = Deserialize::deserialize(deserialiser)?;
        NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map_err(serde::de::Error::custom)
    }

    pub fn serialize_date<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let fmt_d = date.format("%Y-%m-%d").to_string();
        serializer.serialize_str(&fmt_d)
    }
    
    pub fn parse_time<'de, D>(deserialiser: D) -> Result<NaiveTime, D::Error>
    where
    D: Deserializer<'de>
    {
        let s: String = Deserialize::deserialize(deserialiser)?;
        NaiveTime::parse_from_str(&s, "%H:%M")
        .map_err(serde::de::Error::custom)
    }
    
    pub fn serialize_time<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let fmt_d = time.format("%H:%M").to_string();
        serializer.serialize_str(&fmt_d)
    }
    
    pub fn parse_duration_from_minutes<'de, D>(deserialiser: D) -> Result<Duration, D::Error>
    where
    D: Deserializer<'de>
    {
        let s: usize = Deserialize::deserialize(deserialiser)?;
        Duration::new((s * 60) as i64, 0)
        .ok_or_else(|| serde::de::Error::custom(format!(
            "could not parse the following usize into Duration: {s}"
        )))
    }
    
}

/// # Purpose
/// Represents the content of a directory.
/// 
/// Actually, it is used to represent the part of the history content,
/// wether a single file or all the files are considered for future operations.
pub enum DirContent {
    /// Single file with (respectively) its filename and content.
    SingleFile(String, String),
    /// Directory files represented as a map with keys for filenames and values for contents.
    DirectoryFiles(BTreeMap<String, String>),
}