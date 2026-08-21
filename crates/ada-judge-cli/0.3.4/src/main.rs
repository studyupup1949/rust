//! Cli for helping creating problems and contests for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use crate::problems_handlers::{
    handle_archive_problem_cmd, handle_insert_tests_to_problem_cmd, handle_prepare_problem_cmd,
    handle_run_cmd,
};
use anyhow::Ok;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod constants;
mod problems_handlers;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Cli for helping creating problems and contests for ada-judge
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum ProblemCommands {
    /// Create a problem in file system
    Prepare,
    /// Insert given inclusive range of tests to problem
    InsertTests {
        /// Range left boundary
        from: i32,
        /// Range right boundary
        to: i32,
    },
    /// Run solution on given range of tests and write the output
    Run {
        /// Solution's path
        #[arg()]
        solution_path: PathBuf,
        /// Range left boundary
        from: i32,
        /// Range right boundary
        to: i32,
    },
    /// Make a zip archive from the problem
    Zip {},
}

#[derive(Subcommand)]
enum Commands {
    /// Problem related commands
    Problem {
        /// Problem's path
        #[arg()]
        path: PathBuf,

        #[command(subcommand)]
        command: ProblemCommands,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Problem { path, command } => match command {
                ProblemCommands::Prepare => {
                    handle_prepare_problem_cmd(path).await?;
                }
                ProblemCommands::InsertTests { from, to } => {
                    handle_insert_tests_to_problem_cmd(path, from, to).await?;
                }
                ProblemCommands::Run {
                    solution_path,
                    from,
                    to,
                } => {
                    handle_run_cmd(path, solution_path, from, to).await?;
                }
                ProblemCommands::Zip {} => {
                    handle_archive_problem_cmd(path).await?;
                }
            },
        }
    }

    Ok(())
}
