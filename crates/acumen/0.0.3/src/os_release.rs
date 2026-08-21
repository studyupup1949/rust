use std::{borrow::Borrow, collections::BTreeMap, fs, io};

use crate::impl_getters;

#[derive(Debug)]
pub struct OsRelease(BTreeMap<String, String>);

impl OsRelease {
    pub fn new() -> Self {
        Self(Self::new_inner().unwrap_or(BTreeMap::new()))
    }

    pub fn try_new() -> Result<Self, io::Error> {
        Self::new_inner().map(Self)
    }

    fn new_inner() -> Result<BTreeMap<String, String>, io::Error> {
        let os_release_file = fs::read_to_string("/etc/os-release")?;

        let mut entries = BTreeMap::new();

        for line in os_release_file
            .lines()
            .filter(|line| !(line.starts_with('#') || line.is_empty()))
        {
            if let Some((name, content)) = line.split_once('=') {
                entries.insert(
                    name.trim().to_string(),
                    content.trim_matches('"').to_string(),
                );
            }
        }

        Ok(entries)
    }

    pub fn id_like(&self) -> Option<Vec<&str>> {
        self.0
            .get("ID_LIKE")
            .map(|s| s.split_whitespace().collect())
    }

    impl_getters! {
        name: "NAME"
        id: "ID"
        pretty_name: "PRETTY_NAME"
        cpe_name: "CPE_NAME"
        variant: "VARIANT"
        variant_id: "VARIANT_ID"
        version: "VERSION"
    }
}
