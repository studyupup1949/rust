//! `aaai completions` — generate shell tab-completion scripts.

use std::io;
use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};

const COMPLETIONS_AFTER_HELP: &str = "\
Next steps:
  Pipe the output to your shell's completion directory. For example:
    bash :  aaai completions bash > ~/.local/share/bash-completion/completions/aaai
    zsh  :  aaai completions zsh  > \"${fpath[1]}\"/_aaai
    fish :  aaai completions fish > ~/.config/fish/completions/aaai.fish\
";

#[derive(Args)]
#[command(after_help = COMPLETIONS_AFTER_HELP)]
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
