//! This module defines the configuration of the CLI app.
//! It has 3 sections :
//! - history: represents the history containing all tracking information, plus a temporary file for short-term saves
//! - subjects: lists all the subjects that can be present in the section field of the logs
//!     --> it help restraining the user ability to modify the format of the logs,
//!     which is convenient for external uses of those information
//! - conversion: represents the conversion between the original format of the logs into serializable format, 
//!     such as CSV, JSON and XML.

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::utils::serde::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub history: History,
    pub subjects: Subjects,
    pub conversion: Conversion,
    pub samples: Samples,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct History {
    pub path: String,
    pub tmp: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Subjects {
    pub all_subjects: Vec<String>,
    pub current_subject: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Conversion {
    pub directory_path: String,
    pub error_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Samples {
    pub sample_directory: String,
    pub sample_file_path: String,
    pub sample_output_path: String,
    #[serde(deserialize_with = "parse_date", serialize_with = "serialize_date")]
    pub start_day: NaiveDate,
    #[serde(deserialize_with = "parse_date", serialize_with = "serialize_date")]
    pub end_day: NaiveDate,
    #[serde(deserialize_with = "parse_time", serialize_with = "serialize_time")]
    pub start_day_time: NaiveTime,
    #[serde(deserialize_with = "parse_time", serialize_with = "serialize_time")]
    pub end_day_time: NaiveTime,
    pub minimum_task_amount_per_day: usize,
}