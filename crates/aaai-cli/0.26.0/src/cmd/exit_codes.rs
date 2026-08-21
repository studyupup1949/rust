//! `aaai exit-codes` — print the exit-code reference table.
//!
//! RFC 024 FR-4 (optional): expose the exit code table to operators without
//! requiring them to grep source or wade through docs. Same values as
//! documented in `audit.rs` module-doc and `docs/src/cli.md`.

use clap::Args;

#[derive(Args)]
#[command(after_help = AFTER_HELP)]
pub struct ExitCodesArgs {}

const AFTER_HELP: &str = "\
The same table is documented in docs/src/cli.md and in `aaai audit --help`.
Codes are stable across the v1.x line per the compatibility contract.\
";

pub fn run(_args: ExitCodesArgs) -> anyhow::Result<()> {
    println!("Exit code reference:");
    println!();
    println!("  0  PASSED        All entries match their expected rules.");
    println!("  1  FAILED        One or more entries do not match expected rules.");
    println!("  2  PENDING       Entries without a filled-in 'reason' field exist.");
    println!("                   (Suppressed by `aaai audit --allow-pending`.)");
    println!("  3  ERROR         File-level read or compare errors occurred.");
    println!("  4  CONFIG_ERROR  Invalid audit definition or other config error.");
    println!();
    Ok(())
}
