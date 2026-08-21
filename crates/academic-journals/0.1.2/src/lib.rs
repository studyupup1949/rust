mod journal;

use crate::journal::Journal;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;

static JOURNAL_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/generated_journals.bin"));

lazy_static! {
    static ref JOURNALS: Vec<Arc<Journal>> = {
        let journals: Vec<Journal> =
            bincode::deserialize(JOURNAL_DATA).expect("Failed to deserialize journals");
        journals.into_iter().map(Arc::new).collect()
    };
    static ref FULL_NAME_TO_RECORD: HashMap<String, Arc<Journal>> = {
        JOURNALS
            .iter()
            .map(|journal| (journal.name.clone(), Arc::clone(journal)))
            .collect()
    };
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

pub fn get_full_name(abbreviation: &str) -> Option<String> {
    ABBREVIATION_TO_FULL_NAME.get(abbreviation).cloned()
}
