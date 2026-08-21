use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Assignment {
    pub subject: String,
    pub title: String,
    pub due_date: String, 
    pub due_time: String, 
    pub completed: bool,
}

impl Assignment {
    /// Load all assignments from JSON
    pub fn load_from_file() -> Vec<Assignment> {
        fs::read_to_string("assignments.json")
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    /// Save all assignments back to JSON
    pub fn save_to_file(assignments: &Vec<Assignment>) {
        if let Ok(json) = serde_json::to_string_pretty(assignments) {
            let _ = fs::write("assignments.json", json);
        }
    }

    ///  Export text summary with current timestamp
    pub fn export_summary_txt(assignments: &[Assignment]) -> std::io::Result<()> {
        let export_dir = std::path::Path::new("export");
        if !export_dir.exists() {
            fs::create_dir_all(export_dir)?;
        }

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut summary = format!(
            "Assignment Summary\n===================\nGenerated on: {}\n\n",
            now
        );

        for a in assignments {
            let status = if a.completed {
                "✅ Done"
            } else {
                "❌ Pending"
            };
            summary.push_str(&format!(
                "{} - {} (Due: {} {}, Status: {})\n",
                a.subject, a.title, a.due_date, a.due_time, status
            ));
        }

        fs::write(export_dir.join("assignments_summary.txt"), summary)?;
        Ok(())
    }
}
