use std::num::ParseIntError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Invalid value: {0}")]
    InvalidValue(#[from] ParseIntError),
}
