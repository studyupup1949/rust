use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use std::{
    env,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

#[derive(Debug)]
pub struct Config {
    acronym: String,
    file: String,
    acro_column: usize,
    definition_column: usize,
    color: bool,
    header: bool,
    delimiter: char,
}

pub fn get_args() -> Result<Config, Box<dyn Error>> {
    let matches = Command::new("Acro")
        .author("Nil Ventosa")
        .version("0.2.0")
        .about("Helps query csv files of acronyms")
        .arg(
            Arg::new("acronym")
                .help("the acronym to query")
                .required(true),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("the csv file with the acronyms and definitions, can be set with env variable ACRO_FILE"),
        )
        .arg(
            Arg::new("acro_column")
                .short('a')
                .long("acro")
                .help("the column with the acronyms, can be set with env variable ACRO_COLUMN, defaults to 1")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("definition_column")
                .short('d')
                .long("definition")
                .help("the column with the definitions, can be set with env variable DEFINITION_COLUMN, defaults to 2")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("delimiter")
                .short('D')
                .long("delimiter")
                .help("delimiter character between columns. Defaults to ','")
                .value_parser(clap::value_parser!(char)),
        )
        .arg(
            Arg::new("header")
                .short('H')
                .long("header")
                .action(ArgAction::SetTrue)
                .help("flag if there is a header line"),
            )
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .action(ArgAction::SetTrue)
                .help("disables color output"),
            )
        .get_matches();

    let mut file: String = "".to_string();
    if let Some(f) = matches.get_one::<String>("file") {
        file = f.to_string();
    } else if let Ok(f) = env::var("ACRO_FILE") {
        file = f;
    } else {
        eprintln!("File should be specified in argument -f or in env variable ACRO_FILE");
    }

    let mut acro_column: usize = 0;
    if let Some(a) = matches.get_one::<usize>("acro_column") {
        acro_column = a.to_owned() - 1;
    } else if let Ok(a) = env::var("ACRO_COLUMN") {
        if let Ok(a) = a.parse::<usize>() {
            acro_column = a - 1;
        }
    }

    let mut definition_column: usize = 1;
    if let Some(d) = matches.get_one::<usize>("definition_column") {
        definition_column = d.to_owned() - 1;
    } else if let Ok(d) = env::var("DEFINITION_COLUMN") {
        if let Ok(d) = d.parse::<usize>() {
            definition_column = d - 1;
        }
    }

    let mut delimiter: char = ',';
    if let Some(d) = matches.get_one::<char>("delimiter") {
        delimiter = d.to_owned();
    } else if let Ok(d) = env::var("ACRO_DELIMITER") {
        if let Ok(d) = d.parse::<char>() {
            delimiter = d;
        }
    }

    let header: bool = if matches.get_flag("header") {
        true
    } else {
        env::var("ACRO_HEADER").is_ok()
    };

    let color: bool = if matches.get_flag("color") {
        false
    } else {
        !env::var("ACRO_COLOR").is_ok()
    };

    Ok(Config {
        acronym: matches.get_one::<String>("acronym").unwrap().to_string(),
        file,
        acro_column,
        definition_column,
        color,
        header,
        delimiter,
    })
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let all_entries = get_entries_from_file(&config);
    let matching_entries = find_matching_entries(&all_entries, &config.acronym);

    for entry in matching_entries {
        entry.print(&config.color);
    }

    Ok(())
}

#[derive(Debug)]
struct Entry {
    acronym: String,
    definition: String,
}

impl Entry {
    fn print(&self, color: &bool) {
        if *color {
            println!(" {}: {}", self.acronym.bold().blue(), self.definition);
        } else {
            println!(" {}: {}", self.acronym, self.definition);
        }
    }
}

fn find_matching_entries(entries: &Vec<Entry>, acro: &str) -> Vec<Entry> {
    let matching = find_exact_match(entries, acro);

    if matching.is_empty() {
        return find_partial_match(entries, acro);
    }
    matching
}

fn find_exact_match(entries: &Vec<Entry>, acro: &str) -> Vec<Entry> {
    let mut matching = Vec::new();

    for entry in entries {
        if entry.acronym.to_uppercase() == acro.to_uppercase() {
            matching.push(Entry {
                acronym: entry.acronym.clone(),
                definition: entry.definition.clone(),
            });
        }
    }

    matching
}

fn find_partial_match(entries: &Vec<Entry>, acro: &str) -> Vec<Entry> {
    let mut matching = Vec::new();

    for entry in entries {
        if entry.acronym.to_uppercase().contains(&acro.to_uppercase()) {
            matching.push(Entry {
                acronym: entry.acronym.clone(),
                definition: entry.definition.clone(),
            });
        }
    }
    matching
}

fn get_entries_from_file(config: &Config) -> Vec<Entry> {
    let mut entries = Vec::new();

    match open(&config.file) {
        Err(err) => eprintln!("Failed to open {}: {}", config.file, err),
        Ok(file) => {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(config.header)
                .delimiter(config.delimiter as u8)
                .from_reader(file);

            for record in reader.records().flatten() {
                if let (Some(acronym), Some(definition)) = (
                    record.get(config.acro_column),
                    record.get(config.definition_column),
                ) {
                    entries.push(Entry {
                        acronym: acronym.to_string(),
                        definition: definition.to_string(),
                    });
                }
            }
        }
    }
    entries
}

fn open(filename: &str) -> Result<Box<dyn BufRead>, Box<dyn Error>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_entry_vec() -> Vec<Entry> {
        Vec::from([
            Entry {
                acronym: String::from("NATO"),
                definition: String::from("N A T O"),
            },
            Entry {
                acronym: String::from("USA"),
                definition: String::from("U S A"),
            },
        ])
    }

    #[test]
    fn test_find_exact_match_found() {
        let result = find_exact_match(&get_test_entry_vec(), "NATO");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_find_exact_match_nothing() {
        let result = find_exact_match(&get_test_entry_vec(), "NAT");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_partial_match_nothing() {
        let result = find_partial_match(&get_test_entry_vec(), "NATA");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_find_partial_match_two_results() {
        let result = find_partial_match(&get_test_entry_vec(), "A");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_matching_entries_two_results() {
        let result = find_matching_entries(&get_test_entry_vec(), "A");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_matching_entries_one_result() {
        let result = find_matching_entries(&get_test_entry_vec(), "NATO");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_find_matching_entries_no_results() {
        let result = find_matching_entries(&get_test_entry_vec(), "NATA");
        assert_eq!(result.len(), 0);
    }
}
