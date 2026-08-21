mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{list::ListCommand, report::ReportCommand, run::RunCommand};

#[derive(Parser)]
#[command(name = "adversaria")]
#[command(author = "Adversaria Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Adversarial Testing Harness for Large Language Models", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run attack suites against a model")]
    Run(RunCommand),

    #[command(about = "List available attack suites")]
    List(ListCommand),

    #[command(about = "Generate or view reports from previous runs")]
    Report(ReportCommand),
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        match self.command {
            Commands::Run(cmd) => cmd.execute().await,
            Commands::List(cmd) => cmd.execute().await,
            Commands::Report(cmd) => cmd.execute().await,
        }
    }
}
