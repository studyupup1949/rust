use crate::core::{Result, TestRun};
use crate::reporters::Reporter;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub struct JsonReporter {
    output_dir: PathBuf,
}

impl JsonReporter {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    fn ensure_output_dir(&self) -> Result<()> {
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)?;
        }
        Ok(())
    }
}

impl Reporter for JsonReporter {
    fn save_report(&self, test_run: &TestRun) -> Result<String> {
        self.ensure_output_dir()?;

        let filename = format!(
            "adversaria_report_{}_{}.json",
            test_run.timestamp.format("%Y%m%d_%H%M%S"),
            test_run.id
        );

        let filepath = self.output_dir.join(&filename);
        let json = serde_json::to_string_pretty(test_run)?;
        fs::write(&filepath, json)?;

        Ok(filepath.to_string_lossy().to_string())
    }

    fn format_summary(&self, test_run: &TestRun) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n{}\n", "=".repeat(80).bright_cyan()));
        output.push_str(&format!(
            "{}\n",
            "ADVERSARIA TEST REPORT".bold().bright_cyan()
        ));
        output.push_str(&format!("{}\n\n", "=".repeat(80).bright_cyan()));

        output.push_str(&format!("{}: {}\n", "Run ID".bold(), test_run.id));
        output.push_str(&format!("{}: {}\n", "Model".bold(), test_run.model));
        output.push_str(&format!("{}: {}\n", "Provider".bold(), test_run.provider));
        output.push_str(&format!(
            "{}: {}\n",
            "Timestamp".bold(),
            test_run.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!(
            "{}: {:.2}s\n\n",
            "Duration".bold(),
            test_run.duration_ms as f64 / 1000.0
        ));

        let risk_color = match test_run.overall_risk_score {
            0..=25 => "green",
            26..=50 => "yellow",
            51..=75 => "bright_red",
            _ => "red",
        };

        output.push_str(&format!("{}\n", "OVERALL RESULTS".bold().underline()));
        output.push_str(&format!(
            "{}: {}\n",
            "Total Attacks".bold(),
            test_run.total_attacks
        ));
        output.push_str(&format!(
            "{}: {} {}\n",
            "Successful".bold(),
            test_run.successful_attacks,
            if test_run.successful_attacks > 0 {
                "⚠️".red()
            } else {
                "✓".green()
            }
        ));
        output.push_str(&format!(
            "{}: {}\n",
            "Failed".bold(),
            test_run.failed_attacks
        ));

        let risk_display = match risk_color {
            "green" => format!("{}", test_run.overall_risk_score).green(),
            "yellow" => format!("{}", test_run.overall_risk_score).yellow(),
            "bright_red" => format!("{}", test_run.overall_risk_score).bright_red(),
            _ => format!("{}", test_run.overall_risk_score).red(),
        };

        output.push_str(&format!(
            "{}: {}/100 {}\n\n",
            "Overall Risk Score".bold(),
            risk_display,
            Self::risk_level_emoji(test_run.overall_risk_score)
        ));

        output.push_str(&format!("{}\n", "CATEGORY BREAKDOWN".bold().underline()));

        let mut categories: Vec<_> = test_run.category_summary.values().collect();
        categories.sort_by(|a, b| {
            b.average_risk_score
                .partial_cmp(&a.average_risk_score)
                .unwrap()
        });

        for summary in categories {
            let success_rate = if summary.total > 0 {
                (summary.successful as f64 / summary.total as f64) * 100.0
            } else {
                0.0
            };

            output.push_str(&format!(
                "\n  {} {}\n",
                "●".bright_cyan(),
                summary.category.to_string().bold()
            ));
            output.push_str(&format!(
                "    Total: {} | Successful: {} ({:.1}%)\n",
                summary.total, summary.successful, success_rate
            ));
            output.push_str(&format!(
                "    Avg Risk: {:.1} | Max Severity: {:?}\n",
                summary.average_risk_score, summary.max_severity
            ));
        }

        output.push_str(&format!("\n{}\n", "=".repeat(80).bright_cyan()));

        output
    }
}

impl JsonReporter {
    fn risk_level_emoji(score: u8) -> &'static str {
        match score {
            0..=25 => "✅",
            26..=50 => "⚠️",
            51..=75 => "🔴",
            _ => "🚨",
        }
    }
}
