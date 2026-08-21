//! `aaai history` — show recent audit runs.

use clap::Args;
use colored::Colorize;

use aaai_core::history::store as history_store;

#[derive(Args)]
pub struct HistoryArgs {
    /// Number of recent runs to show.
    #[arg(short = 'n', long, default_value = "10")]
    pub count: usize,
    /// Output as JSON.
    #[arg(long = "json-output")]
    pub json_output: bool,
}

pub fn run(args: HistoryArgs) -> anyhow::Result<()> {
    let records = history_store::load_recent(args.count)?;

    if args.json_output {
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(());
    }

    println!("{}", "aaai history".bold());
    println!();

    if records.is_empty() {
        println!("No audit runs recorded yet.");
        return Ok(());
    }

    for r in &records {
        let result_str = if r.result == "PASSED" {
            r.result.green().to_string()
        } else {
            r.result.red().bold().to_string()
        };
        let ts = r.run_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        println!("[{ts}]  {result_str}");
        println!("  Before: {}", r.before);
        println!("  After:  {}", r.after);
        if let Some(def) = &r.definition {
            println!("  Config: {def}");
        }
        println!("  OK: {}  Pending: {}  Failed: {}  Error: {}  Total: {}",
            r.ok, r.pending, r.failed, r.error, r.total);
        println!();
    }
    Ok(())
}
