use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Failed fetching environment variable ${key}: {reason}")]
    VarError {
        key: String,
        reason: String,
        #[source]
        source: Option<std::env::VarError>,
    },

    #[error("Failed to fetch input for year {year} day {day}: {reason}")]
    FetchFailed {
        year: i32,
        day: u8,
        reason: String,
        #[source]
        source: Option<reqwest::Error>,
    },

    #[error("Failed to read input file {path}: {reason}")]
    FileReadError {
        path: PathBuf,
        reason: String,
        #[source]
        source: Option<std::io::Error>,
    },

    #[error("Failed to save input to {path}: {reason}")]
    FileSaveError {
        path: PathBuf,
        reason: String,
        #[source]
        source: Option<std::io::Error>,
    },

    #[error("Failed to open editor")]
    Editor {
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error(
        "The test input for day {day} is not set. Run 'aou add-test <YEAR> <DAY>' to set the day"
    )]
    NoTestInput { day: u8 },

    #[error("Failed creating a temporary file for setting the test input")]
    TestInputFailed {
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
