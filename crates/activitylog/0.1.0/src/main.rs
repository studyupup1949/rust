mod commands;
mod utils;
mod config;
mod args;
mod history;
mod conversion;
mod errors;
mod misc;

use args::*;
use clap::Parser;
use commands::*;

fn main() {
    let args = Args::parse();
    let config = if cfg!(target_os = "linux") ||  cfg!(target_os = "macos") {
        read_config("$HOME/.activitylog/config.toml")
    } else {
        read_config("$HOMEPATH/.activitylog/config.toml")
    };
    match config {
        Ok(mut cfg) => handle_command(&args, &mut cfg),
        Err(e) => println!("Could not get the config file information :\n{}", e),
    }
}