use advent_of_utils_cli::{error::AocError, types::AocDatabase, types::AocTime, Parts};
use std::fs::{read_dir, DirEntry};
use std::time::Duration;
use std::{error::Error, path::PathBuf};

use crate::Cli;

pub enum Config {
    Run(RunConfig),
    AddTest(AddTestConfig),
}

pub struct RunConfig {
    pub year: i32,
    pub day: Option<u8>,
    pub part: Option<Parts>,
    pub test: bool,
    pub database: AocDatabase,
    pub workspace_dir: PathBuf,
    pub benchmark: bool,
}

pub struct AddTestConfig {
    pub year: i32,
    pub day: u8,
    pub database: AocDatabase,
}

pub struct AddDayConfig {
    pub year: i32,
    pub day: u8,
    pub workspace_dir: PathBuf,
}

impl RunConfig {
    pub fn loader_paths(&self) -> Result<Vec<DirEntry>, Box<dyn Error>> {
        let mut matching_files = Vec::new();

        let dir = read_dir(&self.workspace_dir)?;
        for entry in dir {
            let entry = entry?;
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.contains(&self.year.to_string()) {
                    matching_files.push(entry);
                }
            }
        }

        Ok(matching_files)
    }

    pub fn get_run_counts(&self, first_run_duration: Duration) -> (u32, u32) {
        // (warmup, measurement)
        match first_run_duration {
            _ if !self.benchmark => (1, 1),
            d if d < Duration::from_millis(10) => (5, 10), // Very fast: More runs needed
            d if d < Duration::from_millis(100) => (3, 5), // Fast: Moderate runs
            d if d < Duration::from_secs(1) => (2, 3),     // Medium: Fewer runs
            _ => (1, 1),                                   // Slow: Single run sufficient
        }
    }
}

impl Config {
    pub fn from_cli(cli: Cli) -> Result<Self, AocError> {
        match cli {
            Cli::Run(args) => {
                match args.day {
                    Some(day) => AocTime::now().validate_date(args.year, day)?,
                    None => AocTime::now().validate_year(args.year)?,
                }
                Ok(Self::Run(RunConfig {
                    year: args.year,
                    day: args.day,
                    part: match args.part {
                        Some(num) => Some(Parts::new(num)?),
                        None => None,
                    },
                    test: false,
                    workspace_dir: (args.workspace_dir + "/target/release").into(),
                    database: AocDatabase::new()?,
                    benchmark: args.benchmark,
                }))
            }
            Cli::Test(args) => {
                match args.day {
                    Some(day) => AocTime::now().validate_date(args.year, day)?,
                    None => AocTime::now().validate_year(args.year)?,
                }
                Ok(Self::Run(RunConfig {
                    year: args.year,
                    day: args.day,
                    part: match args.part {
                        Some(num) => Some(Parts::new(num)?),
                        None => None,
                    },
                    test: true,
                    workspace_dir: (args.workspace_dir + "/target/release").into(),
                    database: AocDatabase::new()?,
                    benchmark: args.benchmark,
                }))
            }
            Cli::AddTest(args) => {
                AocTime::now().validate_date(args.year, args.day)?;
                Ok(Self::AddTest(AddTestConfig {
                    year: args.year,
                    day: args.day,
                    database: AocDatabase::new()?,
                }))
            }
        }
    }
}
