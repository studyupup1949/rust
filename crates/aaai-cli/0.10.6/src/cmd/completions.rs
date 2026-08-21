//! `aaai completions` — generate shell tab-completion scripts.

use std::io;
use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs) -> anyhow::Result<()> {
    // Re-build the CLI definition so clap_complete can inspect it.
    let mut cmd = crate::Cli::command();
    generate(args.shell, &mut cmd, "aaai", &mut io::stdout());
    Ok(())
}
