//! Provides structs that neatly fit together into a "blocks"-style status
//! string. For example:
//!
//! `>>> VOLUME | BATTERY | CURRENT_TIME <<<`
//!
//! where each element has its own update cycle, etc.

mod statusbar;
mod statusblock;

pub use statusbar::StatusBar;
pub use statusblock::StatusBlock;
