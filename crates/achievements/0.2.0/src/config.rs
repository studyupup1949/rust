use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use homedir::get_my_home;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    pub label: String,
    #[serde(with = "time::serde::rfc3339")]
    pub date: OffsetDateTime,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub days: Vec<Day>,
}

impl Config {
    /// Add/update a day to the config
    ///
    /// The day is added if days doesn't contain a day with the given label.
    ///
    /// If a day with the same (trimmed) label is found its date is updated
    /// with the given date
    pub fn set_day(&mut self, label: &str, date: OffsetDateTime) {
        let label = label.trim();
        match self.days.iter_mut().find(|day| day.label == label) {
            Some(d) => {
                d.date = date;
            }
            None => self.days.push(Day {
                label: label.to_string(),
                date,
            }),
        };
    }

    /// Removes a day with the given label from the config
    ///
    /// The label comparison ignores case and leading/trailing whitespace
    pub fn remove_day(&mut self, label: &str) {
        let label = label.trim().to_lowercase();

        let other_days: Vec<Day> = self
            .days
            .iter()
            .filter(|day| day.label.trim().to_lowercase() != label)
            .cloned()
            .collect();

        self.days = other_days;
    }
}

/// Read config file
///
/// File format is something like:
///
/// ```JSON
/// {
///   "days": [
///     {
///       "label": "Moon landing",
///       "date": "1969-07-20T20:17:40+00:00"
///     },
///     {
///       "label": "Berlin Wall Fall",
///       "date": "1989-11-09T18:53:00+01:00"
///     }
///   ]
/// }
/// ```
///
/// Config file is at `~/.config/achievements/config.json`.
/// If the file doesn't exist an empty `Config` with no days is returned.
///
/// # Panics
/// Currently panics if it can't create the config directory (should return
/// a `Result::Err`).
///
/// Currently panics if the config file isn't valid JSON (should return a
/// `Result::Err`)
pub fn read() -> Result<Config, ()> {
    let config_dir = config_dir();
    create_config_dir(&config_dir).expect("Failed to create config directory");

    let config_file = format!("{config_dir}/config.json");
    let config = if let Ok(reader) = File::open(config_file) {
        serde_json::from_reader(reader).expect("Failed to parse config file")
    } else {
        Config::default()
    };

    Ok(config)
}

/// Write the config to `~/.config/achievements/config.json`
///
/// The file is created if it doesn't exist, updated otherwise.
///
/// # Panics
/// Currently panics if it can't create the config directory (should return
/// a `Result::Err`).
///
/// Currently panics if it can't create/open the config file (should return
/// a `Result::Err`).
///
/// Currently panics if it can't write the config file (should return
/// a `Result::Err`).
///
/// Currently panics if it can't flush the config file (should return
/// a `Result::Err`).
pub fn write(config: &Config) -> Result<(), ()> {
    let config_dir = config_dir();
    create_config_dir(&config_dir).expect("Failed to create config directory");

    let config_file = format!("{config_dir}/config.json");
    let file = File::create(config_file).expect("Failed to create config file");

    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &config)
        .expect("Failed to write JSON to config file");
    writer.flush().expect("Failed to flush config file content");

    Ok(())
}

/// Creates the config directory if it doesn't exist
fn create_config_dir(config_dir: &str) -> Result<(), std::io::Error> {
    create_dir_all(config_dir)
}

fn config_dir() -> String {
    let home = get_my_home().expect("Failed to get home directory");
    let home = home.expect("No home directory");
    let config_dir = home.join(".config").join("achievements");

    if let Some(config_dir) = config_dir.to_str() {
        config_dir.to_string()
    } else {
        panic!("Failed to get config dir");
    }
}

#[test]
fn set_day_test() {
    use time::macros::datetime;

    let mut config = Config { days: vec![] };

    // add to empty config
    let first_label = "Festa della liberazione";
    let first_date = datetime!(1944-04-25 12:00 +02:00);
    config.set_day(first_label, first_date);

    let first_day = config.days.first().expect("Should have a day in it");
    assert_eq!(first_day.label, first_label);
    assert_eq!(first_day.date, first_date);

    // adding another day
    let second_label = "A random day";
    let second_date = datetime!(2000-01-31 12:00 +02:00);
    config.set_day(second_label, second_date);

    assert_eq!(config.days.len(), 2);
    // first day still unchanged
    let first_day = config.days.first().expect("Should have a day in it");
    assert_eq!(first_day.label, first_label);
    assert_eq!(first_day.date, first_date);
    let second_day = config.days.last().expect("Should have a day in it");
    assert_eq!(second_day.label, second_label);
    assert_eq!(second_day.date, second_date);

    // updating an existing day
    let second_label = "   A random day  \n";
    let second_date = datetime!(1000-12-12 12:00 +02:00);
    config.set_day(second_label, second_date);

    assert_eq!(config.days.len(), 2);
    // first day still unchanged
    let first_day = config.days.first().expect("Should have a day in it");
    assert_eq!(first_day.label, first_label);
    assert_eq!(first_day.date, first_date);
    // second day updated
    let second_day = config.days.last().expect("Should have a day in it");
    assert_eq!(second_day.label, second_label.trim());
    assert_eq!(second_day.date, second_date);
}

#[test]
fn remove_day_test() {
    use time::macros::datetime;

    let first_label = "Festa della liberazione";
    let first_date = datetime!(1944-04-25 12:00 +02:00);

    let mut config = Config {
        days: vec![
            Day {
                label: first_label.to_string(),
                date: first_date,
            },
            Day {
                label: "something".to_string(),
                date: datetime!(2000-01-31 12:00 +02:00),
            },
        ],
    };

    // Remove day with given label (case-insensitive, trimmed)
    let input_label = " SomeThing  \n";
    config.remove_day(input_label);

    // only 1 day remaining
    assert_eq!(config.days.len(), 1);

    // first day still unchanged
    let first_day = config.days.first().expect("Should have a day in it");
    assert_eq!(first_day.label, first_label);
    assert_eq!(first_day.date, first_date);

    // day with label "something" removed
    let something_is_found = config.days.iter().find(|day| day.label == "something");
    assert!(something_is_found.is_none());
}
