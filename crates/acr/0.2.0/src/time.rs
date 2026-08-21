pub mod msdos;

use chrono::{DateTime, Local, Utc};

pub type LocalTime = DateTime<Local>;
pub type UtcTime = DateTime<Utc>;
