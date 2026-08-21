use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use rkyv::rancor::Error;
use rkyv::util::AlignedVec;
use rkyv::{from_bytes, Archive, Deserialize};

/// Represents an academic journal, including its name and up to three optional
/// abbreviations.
///
/// This struct is primarily used for internal data storage within the crate and
/// should not be used directly by crate users.
/// Access journal data using provided functions such as `get_abbreviation` and
/// `get_full_name`. The data structure is derived from the `JabRef` journal
/// list.
///
/// # Note
/// Fields are decoded positionally by rkyv. The field order here must match the
/// `Record` struct in `build.rs` (`full_name`, `abbreviation_1`,
/// `abbreviation_2`, `abbreviation_3`).
#[derive(Debug, Clone, Archive, Deserialize)]
pub struct Journal {
    pub name: String,
    pub abbr_1: Option<String>,
    pub abbr_2: Option<String>,
    pub abbr_3: Option<String>,
}

/// Embedded binary data for journal records, compiled into the crate.
///
/// This static byte slice contains serialized journal data, which is
/// deserialized at runtime.
static JOURNAL_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/generated_journals.bin"));

/// Collection of journals, each wrapped in an `Arc<Journal>`.
///
/// The `Arc<Journal>` allows for efficient sharing of journal data across the
/// crate. Journals are deserialized from `JOURNAL_DATA` at startup. This crate
/// is thread-safe; initialization is guaranteed to run at most once.
///
/// # Panics
/// Panics if the deserialization of journal data fails. This indicates
/// corrupted binary data at compile time. Rebuild with `cargo clean && cargo
/// build`.
pub static JOURNALS: LazyLock<Vec<Arc<Journal>>> = LazyLock::new(|| {
    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(JOURNAL_DATA);
    from_bytes::<Vec<Journal>, Error>(&aligned)
        .map(|journals| journals.into_iter().map(Arc::new).collect())
        .expect(
            "Failed to deserialize journal data. This indicates corrupted binary data at compile \
             time. Please rebuild the crate with 'cargo clean && cargo build'.",
        )
});

/// Maps journal full names to their corresponding `Arc<Journal>` objects.
///
/// This facilitates quick lookup of journal data using the full name.
pub static FULL_NAME_TO_RECORD: LazyLock<HashMap<String, Arc<Journal>>> = LazyLock::new(|| {
    JOURNALS
        .iter()
        .map(|j| (j.name.clone(), Arc::clone(j)))
        .collect()
});

/// Maps journal abbreviations to their full names.
///
/// Provides a reverse lookup for finding the full name of a journal based on
/// its abbreviation. Uses `Arc<str>` to avoid duplicating the full name string
/// for each abbreviation.
pub static ABBREVIATION_TO_FULL_NAME: LazyLock<HashMap<String, Arc<str>>> = LazyLock::new(|| {
    JOURNALS
        .iter()
        .flat_map(|journal| {
            let name: Arc<str> = Arc::from(journal.name.as_str());
            [
                journal.abbr_1.as_deref(),
                journal.abbr_2.as_deref(),
                journal.abbr_3.as_deref(),
            ]
            .iter()
            .filter_map(move |&abbr| abbr.map(|a| (a.to_string(), Arc::clone(&name))))
            .collect::<Vec<_>>()
        })
        .collect()
});
