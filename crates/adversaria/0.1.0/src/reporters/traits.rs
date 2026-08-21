use crate::core::{Result, TestRun};

pub trait Reporter: Send + Sync {
    fn save_report(&self, test_run: &TestRun) -> Result<String>;
    fn format_summary(&self, test_run: &TestRun) -> String;
}
