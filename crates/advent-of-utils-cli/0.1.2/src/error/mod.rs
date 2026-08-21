mod database;
mod input;
mod loading;
mod solution;

pub use database::DatabaseError;
pub use input::InputError;
pub use loading::LoadingError;
pub use solution::SolutionError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AocError {
    #[error("Input error: {0}")]
    Input(#[from] InputError),

    #[error("Solution error: {0}")]
    Solution(#[from] SolutionError),

    #[error("Loading error: {0}")]
    Loading(#[from] LoadingError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Invalid part number: {0}")]
    InvalidPart(u8),

    #[error("Invalid year {year}: {reason}")]
    InvalidYear { year: i32, reason: String },

    #[error("Invalid day {day} for year {year}: {reason}")]
    InvalidDay { year: i32, day: u8, reason: String },
}
