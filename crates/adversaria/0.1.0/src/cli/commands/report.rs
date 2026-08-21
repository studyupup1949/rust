use crate::core::Config;
use crate::reporters::{create_reporter, list_reports, load_report};
use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use std::path::PathBuf;

#[derive(Args)]
pub struct ReportCommand {
    #[arg(help = "Report ID or path to display")]
    report_id: Option<String>,

    #[arg(
        short,
        long,
        help = "Path to config file",
        default_value = "adversaria.config.yaml"
    )]
    config: PathBuf,

    #[arg(short, long, help = "List all available reports")]
    list: bool,

    #[arg(short, long, help = "Show detailed results")]
    verbose: bool,
}

impl ReportCommand {
    pub async fn execute(self) -> Result<()> {
        let config = if self.config.exists() {
            Config::load(&self.config).context("Failed to load config file")?
        } else {
            Config::default()
        };

        if self.list {
            return self.list_reports(&config).await;
        }

        if let Some(ref report_id) = self.report_id {
            return self.show_report(&config, report_id).await;
        }

        self.list_reports(&config).await
    }

    async fn list_reports(&self, config: &Config) -> Result<()> {
        println!("{}", "📊 Available Reports".bold().bright_cyan());
        println!();

        let reports = list_reports(config)?;

        if reports.is_empty() {
            println!(
                "{} No reports found in {}",
                "⚠️".yellow(),
                config.reporting.output_directory.display()
            );
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Timestamp").fg(Color::Cyan),
            Cell::new("Model").fg(Color::Cyan),
            Cell::new("Provider").fg(Color::Cyan),
            Cell::new("Risk Score").fg(Color::Cyan),
            Cell::new("Attacks").fg(Color::Cyan),
            Cell::new("File").fg(Color::Cyan),
        ]);

        for report_path in &reports {
            if let Ok(test_run) = load_report(report_path.clone()) {
                let risk_color = match test_run.overall_risk_score {
                    0..=25 => Color::Green,
                    26..=50 => Color::Yellow,
                    51..=75 => Color::Red,
                    _ => Color::DarkRed,
                };

                table.add_row(vec![
                    Cell::new(test_run.timestamp.format("%Y-%m-%d %H:%M:%S")),
                    Cell::new(&test_run.model),
                    Cell::new(&test_run.provider),
                    Cell::new(format!("{}/100", test_run.overall_risk_score)).fg(risk_color),
                    Cell::new(format!(
                        "{}/{}",
                        test_run.successful_attacks, test_run.total_attacks
                    )),
                    Cell::new(report_path.file_name().unwrap().to_string_lossy()),
                ]);
            }
        }

        println!("{}", table);
        println!("\n{}: {} report(s) found", "Total".bold(), reports.len());
        println!(
            "\n{} Use {} to view a specific report",
            "💡".bright_yellow(),
            "adversaria report <filename>".bold()
        );

        Ok(())
    }

    async fn show_report(&self, config: &Config, report_id: &str) -> Result<()> {
        let report_path = if PathBuf::from(report_id).exists() {
            PathBuf::from(report_id)
        } else {
            config.reporting.output_directory.join(report_id)
        };

        if !report_path.exists() {
            anyhow::bail!("Report not found: {}", report_path.display());
        }

        let test_run = load_report(report_path)?;

        let reporter = create_reporter(config);
        let summary = reporter.format_summary(&test_run);
        println!("{}", summary);

        if self.verbose {
            println!("\n{}", "DETAILED RESULTS".bold().underline());

            let successful_attacks: Vec<_> =
                test_run.results.iter().filter(|r| r.success).collect();

            if !successful_attacks.is_empty() {
                println!("\n{} Successful Attacks:", "⚠️".red().bold());
                for (i, result) in successful_attacks.iter().enumerate() {
                    println!(
                        "\n  {}. {} ({})",
                        i + 1,
                        result.payload_name.bold(),
                        format!("{:?}", result.severity).red()
                    );
                    println!("     Category: {}", result.category.to_string());
                    println!("     Risk Score: {}/100", result.risk_score);
                    println!("     Execution Time: {}ms", result.execution_time_ms);
                    println!(
                        "     Prompt: {}",
                        result.prompt.chars().take(100).collect::<String>()
                    );
                    if result.prompt.len() > 100 {
                        println!("            ...");
                    }
                    println!(
                        "     Response: {}",
                        result.response.chars().take(150).collect::<String>()
                    );
                    if result.response.len() > 150 {
                        println!("              ...");
                    }
                }
            }

            let failed_attacks: Vec<_> = test_run.results.iter().filter(|r| !r.success).collect();

            if !failed_attacks.is_empty() && failed_attacks.len() <= 10 {
                println!("\n{} Failed Attacks (Model Defended):", "✓".green().bold());
                for (i, result) in failed_attacks.iter().enumerate() {
                    println!(
                        "  {}. {} - {}",
                        i + 1,
                        result.payload_name,
                        result
                            .detection_reason
                            .as_ref()
                            .unwrap_or(&"Unknown".to_string())
                    );
                }
            }
        }

        Ok(())
    }
}
