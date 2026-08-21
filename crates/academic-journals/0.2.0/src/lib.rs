//! # Academic Journals
//!
//! A Rust library for fast lookups of academic journal abbreviations and full
//! names.
//!
//! This crate provides a simple API to convert between full journal names and
//! their abbreviations, using data from the `JabRef` project
//! (abbrv.jabref.org).
//!
//! ## Features
//!
//! - **Fast O(1) lookups**: Uses pre-built `HashMaps` for instant lookups
//! - **Large dataset**: Thousands of journals from multiple academic
//!   disciplines
//! - **Multiple formats**: Supports both dotted and dotless abbreviation styles
//! - **Zero runtime cost**: All data is embedded at compile time
//! - **Thread-safe**: Statics are initialized once via `std::sync::LazyLock`
//!
//! ## Quick Start
//!
//! ```no_run
//! use academic_journals::{get_abbreviation, get_full_name};
//!
//! // Get abbreviation from full name (dotless feature)
//! let abbr = get_abbreviation("Critical Care Medicine");
//! assert_eq!(abbr, Some("Crit Care Med".to_string()));
//!
//! // Get full name from abbreviation (dotless feature)
//! let name = get_full_name("Crit Care Med");
//! assert_eq!(name, Some("Critical Care Medicine".to_string()));
//! ```
//!
//! ## Feature Flags
//!
//! - `dotless` (default): Use dotless abbreviations (e.g., "Crit Care Med")
//! - `dot`: Use dotted abbreviations (e.g., "Crit. Care Med.")
//! - `online` (default): Download latest data at build time
//!
//! ## Data Source
//!
//! Journal data is provided by the `JabRef` project and is released under
//! CC0 1.0 Universal (CC0 1.0) Public Domain Dedication.

mod structures;

use crate::structures::{ABBREVIATION_TO_FULL_NAME, FULL_NAME_TO_RECORD};

/// Retrieves the first available abbreviation for a given journal's full name.
///
/// Lookups are exact and case-sensitive. Returns `None` if the journal is not
/// in the dataset or has no abbreviation recorded.
///
/// # Arguments
/// * `full_name` - A string slice representing the full name of the journal.
///
/// # Returns
/// `Some(String)` containing the abbreviation if found, otherwise `None`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "dotless")] {
/// let abbreviation = academic_journals::get_abbreviation("Critical Care Medicine");
/// assert_eq!(abbreviation, Some("Crit Care Med".to_string()));
/// # }
/// # #[cfg(feature = "dot")] {
/// let abbreviation = academic_journals::get_abbreviation("ACS Catalysis");
/// assert_eq!(abbreviation, Some("ACS Catal.".to_string()));
/// # }
/// ```
#[must_use]
pub fn get_abbreviation(full_name: &str) -> Option<String> {
    FULL_NAME_TO_RECORD.get(full_name).and_then(|journal| {
        journal
            .abbr_1
            .as_ref()
            .or_else(|| journal.abbr_2.as_ref())
            .or_else(|| journal.abbr_3.as_ref())
            .cloned()
    })
}

/// Retrieves the full name of a journal given its abbreviation.
///
/// Lookups are exact and case-sensitive. Returns `None` if the abbreviation is
/// not in the dataset.
///
/// # Arguments
/// * `abbreviation` - A string slice representing the abbreviation of the
///   journal.
///
/// # Returns
/// `Some(String)` containing the full name if found, otherwise `None`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "dotless")] {
/// let full_name = academic_journals::get_full_name("Crit Care Med");
/// assert_eq!(full_name, Some("Critical Care Medicine".to_string()));
/// # }
/// # #[cfg(feature = "dot")] {
/// let full_name = academic_journals::get_full_name("ACS Catal.");
/// assert_eq!(full_name, Some("ACS Catalysis".to_string()));
/// # }
/// ```
#[must_use]
pub fn get_full_name(abbreviation: &str) -> Option<String> {
    ABBREVIATION_TO_FULL_NAME
        .get(abbreviation)
        .map(ToString::to_string)
}
