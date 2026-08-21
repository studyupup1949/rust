use crate::core::{AttackSuite, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct SuiteLoader;

impl SuiteLoader {
    pub fn load_suite<P: AsRef<Path>>(path: P) -> Result<AttackSuite> {
        let content = fs::read_to_string(path.as_ref())?;

        let suite = if path.as_ref().extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };

        Ok(suite)
    }

    pub fn load_suites_from_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<AttackSuite>> {
        let mut suites = Vec::new();

        if !dir.as_ref().exists() {
            return Ok(suites);
        }

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" || ext == "json" {
                        match Self::load_suite(path) {
                            Ok(suite) => suites.push(suite),
                            Err(e) => {
                                tracing::warn!("Failed to load suite from {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(suites)
    }

    pub fn filter_enabled(suites: Vec<AttackSuite>, enabled_names: &[String]) -> Vec<AttackSuite> {
        suites
            .into_iter()
            .filter(|s| s.enabled && enabled_names.contains(&s.id))
            .collect()
    }
}
