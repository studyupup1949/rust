use crate::core::Config;
use crate::providers;
use crate::reporters::create_reporter;
use crate::suites::{SuiteLoader, SuiteRunner};
use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Args)]
pub struct RunCommand {
    #[arg(short, long, help = "Provider to use (openai, anthropic, ollama)")]
    provider: Option<String>,

    #[arg(short, long, help = "Model to test")]
    model: Option<String>,

    #[arg(
        short,
        long,
        help = "Path to config file",
        default_value = "adversaria.config.yaml"
    )]
    config: PathBuf,

    #[arg(short, long, help = "Specific suite IDs to run (comma-separated)")]
    suites: Option<String>,

    #[arg(long, help = "Skip saving report")]
    no_save: bool,
}

impl RunCommand {
    pub async fn execute(self) -> Result<()> {
        println!(
            "{}",
            "🔍 Adversaria - LLM Security Testing".bold().bright_cyan()
        );
        println!();

        let config = if self.config.exists() {
            Config::load(&self.config).context("Failed to load config file")?
        } else {
            println!(
                "{} Config file not found, creating default at {}",
                "⚠️".yellow(),
                self.config.display()
            );
            Config::create_default(&self.config)?;
            Config::default()
        };

        let provider_name = self.provider.as_ref().unwrap_or(&config.default_provider);

        println!(
            "{} Loading provider: {}",
            "→".bright_cyan(),
            provider_name.bold()
        );

        let provider = providers::create_provider(provider_name, &config)
            .context("Failed to create provider")?;

        println!("{} Checking provider health...", "→".bright_cyan());
        match provider.health_check().await {
            Ok(true) => println!("{} Provider is healthy", "✓".green()),
            Ok(false) => println!("{} Provider health check failed", "✗".red()),
            Err(e) => println!("{} Provider health check error: {}", "⚠️".yellow(), e),
        }

        println!("{} Loading attack suites...", "→".bright_cyan());

        let mut suites = SuiteLoader::load_suites_from_directory(&config.suites.directory)
            .context("Failed to load suites")?;

        if suites.is_empty() {
            println!(
                "{} No suites found in {}",
                "⚠️".yellow(),
                config.suites.directory.display()
            );
            return Ok(());
        }

        if let Some(suite_filter) = self.suites {
            let filter_ids: Vec<String> = suite_filter
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            suites.retain(|s| filter_ids.contains(&s.id));
        } else {
            suites = SuiteLoader::filter_enabled(suites, &config.suites.enabled_suites);
        }

        println!("{} Loaded {} suite(s)", "✓".green(), suites.len());
        for suite in &suites {
            println!(
                "  {} {} ({} payloads)",
                "●".bright_cyan(),
                suite.name.bold(),
                suite.payloads.len()
            );
        }
        println!();

        println!("{}", "Starting attack simulation...".bold());
        println!();

        let runner = SuiteRunner::new(provider);
        let test_run = runner.run_suites(suites).await?;

        println!();

        let reporter = create_reporter(&config);
        let summary = reporter.format_summary(&test_run);
        println!("{}", summary);

        if !self.no_save {
            let report_path = reporter.save_report(&test_run)?;
            println!("{} Report saved to: {}", "✓".green(), report_path.bold());
        }

        if test_run.overall_risk_score > 50 {
            println!(
                "\n{} High risk score detected! Review the report for details.",
                "⚠️".red().bold()
            );
        }

        Ok(())
    }
}
