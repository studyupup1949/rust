//! `aaai report` — generate a Markdown or JSON audit report.

use std::path::PathBuf;
use clap::{Args, ValueEnum};
use colored::Colorize;

use aaai_core::{
    AuditEngine, DiffEngine,
    config::io as config_io,
    report::generator::ReportGenerator,
};

#[derive(Args)]
pub struct ReportArgs {
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: PathBuf,
    /// Output file path.
    #[arg(short = 'o', long, value_name = "FILE")]
    pub out: PathBuf,
    /// Report format.
    #[arg(short = 'f', long, default_value = "markdown")]
    pub format: ReportFormat,
    /// Embed actual diff text in the report (Markdown/HTML only).
    #[arg(long)]
    pub include_diff: bool,
}

#[derive(Clone, ValueEnum)]
pub enum ReportFormat {
    Markdown,
    Json,
    Html,
    Sarif,
}

pub fn run(args: ReportArgs) -> anyhow::Result<()> {
    println!("{}", "aaai report".bold());
    let definition = config_io::load(&args.config)?;
    let diffs = DiffEngine::compare(&args.left, &args.right)?;
    let result = AuditEngine::evaluate(&diffs, &definition);

    match args.format {
        ReportFormat::Sarif => {
            aaai_core::report::generator::ReportGenerator::write_sarif(
                &result, &args.left, &args.right, &args.out,
            )?;
        }
        ReportFormat::Markdown => {
            if args.include_diff {
                let md = aaai_core::report::generator::ReportGenerator::build_markdown_string(
                    &result, &args.left, &args.right, Some(&args.config), None, true,
                );
                std::fs::write(&args.out, md.as_bytes())?;
            } else {
                ReportGenerator::write_markdown(
                    &result, &args.left, &args.right, Some(&args.config), &args.out, None,
                )?;
            }
        }
        ReportFormat::Html => {
            aaai_core::report::generator::ReportGenerator::write_html(
                &result, &args.left, &args.right, Some(&args.config), &args.out, None,
            )?;
        }
        ReportFormat::Json => {
            ReportGenerator::write_json(
                &result, &args.left, &args.right, Some(&args.config), &args.out, None,
            )?;
        }
    }

    let s = &result.summary;
    println!("{} report generated: {}", "✓".green(), args.out.display());
    println!(
        "  Summary — OK: {}  Pending: {}  Failed: {}  Error: {}",
        s.ok, s.pending, s.failed, s.error
    );
    Ok(())
}
