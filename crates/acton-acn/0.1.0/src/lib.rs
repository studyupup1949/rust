mod crystal;
mod utils;

pub use crystal::{Crystal, CrystalConfig};
pub use utils::generate_session_id;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActonError {
    #[error("Crystal error: {0}")]
    Crystal(String),
}

pub type Result<T> = std::result::Result<T, ActonError>;

pub fn version() -> &'static str {
    "0.1.0"
}