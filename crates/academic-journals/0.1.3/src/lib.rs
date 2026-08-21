mod journal;

use crate::journal::Journal;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;

/// Static binary data for journal records, embedded at compile time.
static JOURNAL_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/generated_journals.bin"));

lazy_static! {
        /// A vector of `Arc<Journal>` objects, containing shared references to deserialized journals.
        ///
        /// # Panics
        /// Panics if deserialization of journals fails.
    static ref JOURNALS: Vec<Arc<Journal>> = {
        let journals: Vec<Journal> =
            bincode::deserialize(JOURNAL_DATA).expect("Failed to deserialize journals");
        journals.into_iter().map(Arc::new).collect()
    };
        /// A hash map mapping journal full names to their `Arc<Journal>` objects.
    static ref FULL_NAME_TO_RECORD: HashMap<String, Arc<Journal>> = {
        JOURNALS
            .iter()
            .map(|journal| (journal.name.clone(), Arc::clone(journal)))
            .collect()
    };
        /// A hash map mapping journal abbreviations to their full names.
    static ref ABBREVIATION_TO_FULL_NAME: HashMap<String, String> = {
        JOURNALS
            .iter()
            .flat_map(|journal| {
                let name = &journal.name;
                [
                    journal.abbr_1.as_deref(),
                    journal.abbr_2.as_deref(),
                    journal.abbr_3.as_deref(),
                ]
                .into_iter()
                .flatten()
                .map(move |abbr| (abbr.to_string(), name.clone()))
            })
            .collect()
    };
}

/// Retrieves the first available abbreviation for a given journal full name.
///
/// # Arguments
/// * `full_name` - The full name of the journal.
///
/// # Returns
/// Returns `Some(String)` containing the abbreviation if found, otherwise `None`.
///
/// # Examples
/// ```
/// let abbreviation = academic_journals::get_abbreviation("Journal of Rust Studies").unwrap();
/// println!("Abbreviation: {}", abbreviation);
/// ```
pub fn get_abbreviation(full_name: &str) -> Option<String> {
    FULL_NAME_TO_RECORD.get(full_name).and_then(|journal| {
        journal
            .abbr_1
            .as_ref()
            .or(journal.abbr_2.as_ref())
            .or(journal.abbr_3.as_ref())
            .cloned()
    })
}

/// Retrieves the full name for a given journal abbreviation.
///
/// # Arguments
/// * `abbreviation` - The abbreviation of the journal.
///
/// # Returns
/// Returns `Some(String)` containing the full name if found, otherwise `None`.
///
/// # Examples
/// ```
/// let full_name = academic_journals::get_full_name("JRS").unwrap();
/// println!("Full name: {}", full_name);
/// ```
pub fn get_full_name(abbreviation: &str) -> Option<String> {
    ABBREVIATION_TO_FULL_NAME.get(abbreviation).cloned()
}
