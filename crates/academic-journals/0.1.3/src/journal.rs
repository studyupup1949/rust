use serde::Deserialize;

/// Represents a journal with its name and possible abbreviations.
///
/// This struct is designed to store data about academic journals. It includes the journal's full name
/// and up to three optional abbreviations.
///
/// # Fields
/// - `name`: The full name of the journal.
/// - `abbr_1`, `abbr_2`, `abbr_3`: Optional fields for different abbreviations of the journal's name.
#[derive(Debug, Deserialize, Clone)]
pub struct Journal {
    pub name: String,
    pub abbr_1: Option<String>,
    pub abbr_2: Option<String>,
    pub abbr_3: Option<String>,
}
