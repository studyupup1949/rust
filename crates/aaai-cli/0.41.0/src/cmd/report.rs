//! `aaai report` — generate a Markdown or JSON audit report.

use std::path::PathBuf;
use clap::{Args, ValueEnum};
use colored::Colorize;

use aaai::{
    AuditEngine, DiffEngine, Masking, MaskingEngine,
    config::io as config_io,
    project::config::ProjectConfig,
    report::generator::ReportGenerator,
};

const REPORT_AFTER_HELP: &str = "\
Next steps:
  Review the generated report. Re-run `aaai audit` and regenerate the
  report after any rule or reason changes.\
";

#[derive(Args)]
#[command(after_help = REPORT_AFTER_HELP)]
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

    // RFC 103 §5.1a — a report file is written to be reviewed and
    // circulated; it is the paradigm untrusted sink, so masking is always
    // enabled here (owner decision, 2026-07-29). Built the same way
    // `cmd/audit.rs:98` builds its own engine, so project-level custom
    // patterns are honoured.
    let custom_patterns = ProjectConfig::discover(&args.left)
        .unwrap_or(None)
        .map(|(config, _)| config.custom_mask_patterns)
        .unwrap_or_default();
    let masker = MaskingEngine::with_custom(&custom_patterns);
    let masking = Masking::Enabled(&masker);

    match args.format {
        ReportFormat::Sarif => {
            aaai::report::generator::ReportGenerator::write_sarif(
                &result, &args.left, &args.right, &args.out, masking,
            )?;
        }
        ReportFormat::Markdown => {
            if args.include_diff {
                let md = aaai::report::generator::ReportGenerator::build_markdown_string(
                    &result, &args.left, &args.right, Some(&args.config), masking, true,
                );
                std::fs::write(&args.out, md.as_bytes())?;
            } else {
                ReportGenerator::write_markdown(
                    &result, &args.left, &args.right, Some(&args.config), &args.out, masking,
                )?;
            }
        }
        ReportFormat::Html => {
            aaai::report::generator::ReportGenerator::write_html(
                &result, &args.left, &args.right, Some(&args.config), &args.out, masking,
            )?;
        }
        ReportFormat::Json => {
            ReportGenerator::write_json(
                &result, &args.left, &args.right, Some(&args.config), &args.out, masking,
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
