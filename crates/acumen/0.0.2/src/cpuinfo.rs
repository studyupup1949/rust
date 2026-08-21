use crate::impl_getters;
use std::{borrow::Borrow, collections::BTreeMap, fs, io};

#[derive(Debug)]
pub struct Cpuinfo(Vec<Cpu>);

#[derive(Debug)]
pub struct Cpu(BTreeMap<String, String>);

impl Cpuinfo {
    pub fn new() -> Self {
        Self(Self::new_inner().unwrap_or(Vec::new()))
    }

    pub fn try_new() -> Result<Self, io::Error> {
        Self::new_inner().map(Self)
    }

    fn new_inner() -> Result<Vec<Cpu>, io::Error> {
        let cpuinfo_file = fs::read_to_string("/proc/cpuinfo")?;

        let mut cpus = Vec::new();

        for info in cpuinfo_file.split("\n\n").filter(|s| !s.is_empty()) {
            let mut entries = BTreeMap::new();

            for line in info.lines().filter(|s| !s.is_empty()) {
                if let Some((name, content)) = line.split_once(':') {
                    entries.insert(name.trim().to_string(), content.trim().to_string());
                }
            }

            cpus.push(Cpu(entries))
        }

        Ok(cpus)
    }

    pub fn cpus(&self) -> &[Cpu] {
        &self.0
    }
}

impl Cpu {
    impl_getters! {
        model_name: "model name"
    }
}
