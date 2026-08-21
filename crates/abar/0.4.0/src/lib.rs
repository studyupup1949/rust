//! Provides structs that neatly fit together into a "blocks"-style status
//! string. For example:
//!
//! `>>> VOLUME | BATTERY | CURRENT_TIME <<<`
//!
//! where each element has its own update cycle, etc.
//!
//! ABar has the potential to be fully-asyncronous, which is ideal for people
//! who want their bars to perform slow operations like http requests, etc.

mod statusbar;
mod statusblock;

pub use statusbar::StatusBar;
pub use statusblock::{Command, StatusBlock};
