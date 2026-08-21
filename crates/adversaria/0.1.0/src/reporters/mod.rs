pub mod json_reporter;
pub mod traits;

pub use json_reporter::JsonReporter;
pub use traits::Reporter;

use crate::core::{Config, Result, TestRun};
use std::path::PathBuf;

pub fn create_reporter(config: &Config) -> Box<dyn Reporter> {
    Box::new(JsonReporter::new(config.reporting.output_directory.clone()))
}

pub fn list_reports(config: &Config) -> Result<Vec<PathBuf>> {
    let report_dir = &config.reporting.output_directory;

    if !report_dir.exists() {
        return Ok(Vec::new());
    }

    let mut reports = Vec::new();

    for entry in std::fs::read_dir(report_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    reports.push(path);
                }
            }
        }
    }

    reports.sort_by(|a, b| {
        let a_meta = std::fs::metadata(a).ok();
        let b_meta = std::fs::metadata(b).ok();

        match (a_meta, b_meta) {
            (Some(a_m), Some(b_m)) => b_m
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(&a_m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)),
            _ => std::cmp::Ordering::Equal,
        }
    });

    Ok(reports)
}

pub fn load_report(path: PathBuf) -> Result<TestRun> {
    let content = std::fs::read_to_string(path)?;
    let test_run = serde_json::from_str(&content)?;
    Ok(test_run)
}
