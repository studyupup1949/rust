use std::{borrow::Borrow, collections::BTreeMap, fs, io};

use crate::impl_getters;

#[derive(Debug)]
pub struct Meminfo(BTreeMap<String, usize>);

impl Meminfo {
    pub fn new() -> Self {
        Self(Self::new_inner().unwrap_or(BTreeMap::new()))
    }

    pub fn try_new() -> Result<Self, io::Error> {
        Self::new_inner().map(Self)
    }

    fn new_inner() -> Result<BTreeMap<String, usize>, io::Error> {
        let meminfo_file = fs::read_to_string("/proc/meminfo")?;

        let mut entries = BTreeMap::new();

        for line in meminfo_file.lines().filter(|line| !line.is_empty()) {
            if let Some((name, content)) = line.split_once(':') {
                if let Ok(value) = content.trim_end_matches("kB").trim().parse() {
                    entries.insert(name.trim().to_string(), value);
                }
            }
        }

        Ok(entries)
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        String: Borrow<Q>,
        Q: Ord,
    {
        self.0.get(key).copied()
    }

    impl_getters! {
        usize,
        mem_total: "MemTotal"
    }
}
