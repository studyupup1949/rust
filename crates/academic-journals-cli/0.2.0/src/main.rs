//! Command-line interface for academic journal abbreviation lookups.

use std::fs;

use academic_journals::{get_abbreviation, get_full_name};
use clap::Parser;

use crate::command::{Cli, InputType};

mod command;

// TODO: Add possibility to convert a list of journal names to abbreviations or
// vice versa. In cases of ambiguity, offer a list of possible matches with the
// matching score.

/// The entry point of the application.
///
/// This function parses command-line arguments using the `Cli` struct from the
/// `cli` module and interacts with the `academic_journals` module to resolve
/// journal abbreviations or full names.
///
/// # Examples
/// ```bash
/// # To get the abbreviation of a journal:
/// academic-journals --abbreviation "Journal of Rust Studies"
///
/// # To get the full name from an abbreviation:
/// academic-journals "JRS"
/// ```
fn main() {
    let cli: Cli = Cli::parse();

    // Process input based on the specified input type
    match cli.input_type {
        InputType::String => process_input(&cli.input, cli.abbreviation),
        InputType::File => {
            if let Ok(contents) = fs::read_to_string(&cli.input) {
                for line in contents.lines() {
                    process_input(line, cli.abbreviation);
                }
            } else {
                eprintln!("Failed to read file: {}", &cli.input);
            }
        }
    }
}

fn process_input(input: &str, abbreviation_mode: bool) {
    if abbreviation_mode {
        let result =
            get_abbreviation(input).unwrap_or_else(|| format!("No abbreviation found for {input}"));
        println!("{result}");
    } else {
        let result =
            get_full_name(input).unwrap_or_else(|| format!("No full name found for {input}"));
        println!("{result}");
    }
}
