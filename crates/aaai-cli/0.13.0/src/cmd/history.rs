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
    /// Show trend analytics (pass rate, averages).
    #[arg(long)]
    pub stats: bool,
    /// Prune history to at most N entries (newest kept).
    #[arg(long, value_name = "N")]
    pub prune: Option<usize>,
}

pub fn run(args: HistoryArgs) -> anyhow::Result<()> {
    let records = history_store::load_recent(args.count)?;

    if args.json_output {
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(());
    }

    if let Some(max) = args.prune {
        let removed = aaai_core::history::store::prune(max)?;
        println!("{} History pruned: kept up to {max} entries, removed {removed}.", "✓".green());
        return Ok(());
    }

    println!("{}", "aaai history".bold());

    if args.stats {
        return show_stats();
    }

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

fn show_stats() -> anyhow::Result<()> {
    use colored::Colorize;
    let all = aaai_core::history::store::load_all()?;
    if all.is_empty() {
        println!("No history recorded yet.");
        return Ok(());
    }

    let total = all.len();
    let passed = all.iter().filter(|r| r.result == "PASSED").count();
    let pass_rate = (passed as f64 / total as f64) * 100.0;

    let avg_ok: f64 = all.iter().map(|r| r.ok as f64).sum::<f64>() / total as f64;
    let avg_pending: f64 = all.iter().map(|r| r.pending as f64).sum::<f64>() / total as f64;
    let avg_failed: f64 = all.iter().map(|r| r.failed as f64).sum::<f64>() / total as f64;

    println!("{}", "Audit History Analytics".bold());
    println!();
    println!("  Total runs   : {total}");
    println!("  Pass rate    : {:.1}%  ({passed}/{total})",
        pass_rate);
    println!("  Avg OK/run   : {avg_ok:.1}");
    println!("  Avg Pending  : {avg_pending:.1}");
    println!("  Avg Failed   : {avg_failed:.1}");

    // Recent trend (last 5 vs previous 5)
    if total >= 10 {
        let recent_pass = all[..5].iter().filter(|r| r.result == "PASSED").count();
        let older_pass  = all[5..10].iter().filter(|r| r.result == "PASSED").count();
        let trend = if recent_pass > older_pass { "↑ Improving".green() }
                    else if recent_pass < older_pass { "↓ Declining".red() }
                    else { "→ Stable".yellow() };
        println!("  Recent trend : {trend}  (last 5 vs prior 5)");
    }

    println!();
    println!("  Most recent runs:");
    for r in all.iter().take(5) {
        let res = if r.result == "PASSED" { r.result.green() } else { r.result.red() };
        let ts  = r.run_at.format("%Y-%m-%d %H:%M").to_string();
        println!("    [{ts}] {res}  OK:{} Fail:{}", r.ok, r.failed);
    }

    Ok(())
}
