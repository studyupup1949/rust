use crate::core::Config;
use crate::suites::SuiteLoader;
use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use std::path::PathBuf;

#[derive(Args)]
pub struct ListCommand {
    #[arg(
        short,
        long,
        help = "Path to config file",
        default_value = "adversaria.config.yaml"
    )]
    config: PathBuf,

    #[arg(short, long, help = "Show detailed information")]
    verbose: bool,
}

impl ListCommand {
    pub async fn execute(self) -> Result<()> {
        println!("{}", "📋 Available Attack Suites".bold().bright_cyan());
        println!();

        let config = if self.config.exists() {
            Config::load(&self.config).context("Failed to load config file")?
        } else {
            Config::default()
        };

        let suites = SuiteLoader::load_suites_from_directory(&config.suites.directory)
            .context("Failed to load suites")?;

        if suites.is_empty() {
            println!(
                "{} No suites found in {}",
                "⚠️".yellow(),
                config.suites.directory.display()
            );
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            Cell::new("ID").fg(Color::Cyan),
            Cell::new("Name").fg(Color::Cyan),
            Cell::new("Category").fg(Color::Cyan),
            Cell::new("Payloads").fg(Color::Cyan),
            Cell::new("Enabled").fg(Color::Cyan),
        ]);

        for suite in &suites {
            let enabled_str = if suite.enabled { "✓" } else { "✗" };
            let enabled_color = if suite.enabled {
                Color::Green
            } else {
                Color::Red
            };

            table.add_row(vec![
                Cell::new(&suite.id),
                Cell::new(&suite.name),
                Cell::new(suite.category.to_string()),
                Cell::new(suite.payloads.len()),
                Cell::new(enabled_str).fg(enabled_color),
            ]);
        }

        println!("{}", table);
        println!("\n{}: {} suite(s) found", "Total".bold(), suites.len());

        if self.verbose {
            println!("\n{}", "Detailed Information:".bold().underline());
            for suite in &suites {
                println!("\n{} {}", "●".bright_cyan(), suite.name.bold());
                println!("  ID: {}", suite.id);
                println!("  Description: {}", suite.description);
                println!("  Category: {}", suite.category.to_string());
                println!("  Payloads: {}", suite.payloads.len());

                if !suite.payloads.is_empty() {
                    println!("  Sample payloads:");
                    for (i, payload) in suite.payloads.iter().take(3).enumerate() {
                        println!(
                            "    {}. {} ({})",
                            i + 1,
                            payload.name,
                            format!("{:?}", payload.severity).to_lowercase()
                        );
                    }
                    if suite.payloads.len() > 3 {
                        println!("    ... and {} more", suite.payloads.len() - 3);
                    }
                }
            }
        }

        Ok(())
    }
}
