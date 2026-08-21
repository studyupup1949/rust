//! Cli for helping creating problems and contests for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use crate::{
    config::load_config,
    db::Database,
    problems_handlers::{
        handle_create_problem_cmd, handle_insert_range_of_tests_to_problem_cmd,
        handle_prepare_problem_cmd,
    },
};
use anyhow::Ok;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod config;
mod constants;
mod db;
mod problems_handlers;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Cli for helping creating problems and contests for ada-judge
struct Cli {
    /// Custom config path
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum ProblemCommands {
    /// Create a problem in file system
    Prepare,
    /// Create a problem in database
    Create,
    /// Insert given inclusive range of tests to problem
    InsertRangeOfTests {
        /// Range left boundary
        from: i32,
        /// Range right boundary
        to: i32,
    },
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

    /// Create a contest in database
    CreateContest {
        /// Contest's name
        #[arg()]
        name: String,
        /// Timestamp when contest starts, in UTC chrono format
        #[arg()]
        starts_at: DateTime<Utc>,
        /// Timestamp when contest ends, in UTC chrono format
        #[arg()]
        ends_at: DateTime<Utc>,
        /// Owner's id
        #[arg()]
        owner_id: Option<i64>,
    },
}

async fn handle_create_contest_cmd(
    cli: &Cli,
    owner_id: Option<&i64>,
    name: &str,
    starts_at: &DateTime<Utc>,
    ends_at: &DateTime<Utc>,
) -> anyhow::Result<()> {
    let config = load_config(cli).await?;
    let db = Database::new(&config).await.map_err(|e| {
        println!("{} {e}", "Failed to connect to database:".red().bold());
        e
    })?;
    let contest_id = db
        .insert_contest(owner_id, name, starts_at, ends_at)
        .await
        .map_err(|e| {
            println!(
                "{} {e}",
                "Failed to insert a contest to database:".red().bold()
            );
            e
        })?;

    println!(
        "{} {contest_id}",
        "Contest has inserted to database successfully. Contest's id:"
            .green()
            .italic()
    );

    Ok(())
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
                ProblemCommands::Create => {
                    handle_create_problem_cmd(&cli, path).await?;
                }
                ProblemCommands::InsertRangeOfTests { from, to } => {
                    handle_insert_range_of_tests_to_problem_cmd(path, from, to).await?;
                }
            },
            Commands::CreateContest {
                owner_id,
                name,
                starts_at,
                ends_at,
            } => {
                handle_create_contest_cmd(&cli, owner_id.as_ref(), name, starts_at, ends_at)
                    .await?;
            }
        }
    }

    Ok(())
}
