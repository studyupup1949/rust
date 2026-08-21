use address_book::parse_phone_numbers; 
use anyhow::Result; 
use clap::{Parser, Subcommand}; 
use std::fs; 

#[derive(Parser)]
#[command(
    author = "Khrystyna Kosiv",
    version = "1.0",
    about = "Parses phone numbers from input"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ParseFile { file_path: String },
    Credits,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ParseFile { file_path } => {
            let content = fs::read_to_string(file_path)?;
            match parse_phone_numbers(&content) {
                Ok(_) => println!("Parsing successful"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Credits => {
            println!("Phone Number Parser CLI\nCreated by Khrystyna Kosiv\nVersion 1.0");
        }
    }

    Ok(())
}
