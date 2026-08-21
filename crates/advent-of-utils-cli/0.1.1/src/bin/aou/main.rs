mod adder;
mod config;
mod loader;
mod runner;

use clap::{Args, Parser};
use config::Config;
use std::process;

#[derive(Parser)]
enum Cli {
    Run(RunArgs),
    Test(RunArgs),
    AddTest(AddArgs),
}

#[derive(Args)]
#[command(author, version, about, long_about = None)]
struct RunArgs {
    #[arg()]
    year: i32,

    #[arg()]
    day: Option<u8>,

    #[arg(short, long)]
    part: Option<u8>,

    #[arg(long, default_value = ".")]
    workspace_dir: String,
}

#[derive(Args)]
#[command(author, version, about, long_about = None)]
struct AddArgs {
    #[arg()]
    year: i32,

    #[arg()]
    day: u8,
}

fn main() {
    let cli = Cli::parse();

    let config = match Config::from_cli(cli) {
        Ok(config) => config,
        Err(error) => {
            println!("{error}");
            process::exit(1);
        }
    };

    if let Err(error) = match config {
        Config::Run(config) => runner::run(&config),
        Config::AddTest(config) => adder::run(&config),
    } {
        println!("{error}");
        process::exit(1);
    };
}
