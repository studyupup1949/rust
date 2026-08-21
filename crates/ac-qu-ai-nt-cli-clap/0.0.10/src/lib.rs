use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
enum Command {}

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

pub fn main(_args: Args) {
    #[cfg(feature = "tracing")]
    tracing::info!("What's up, world?");
}
