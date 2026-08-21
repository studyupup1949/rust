//! # Purpose
//! This module gathers all elements defined for accessing and editing history content.
//! # Examples
//! Here is a non-exhaustive list of elements:
//! - [`HistoryRecord`] => model of an history item, which is a de-serializable struct
//! - [`add_to_tmp`] => add to a temporary file a new record
//! - [`save_history`] => save the commited records contained in the temp. file into the history log file

use std::{error::Error, fmt::Display, fs::OpenOptions, io::Write};

use serde::{Deserialize, Serialize};
use serde_json::{de::from_reader, to_writer_pretty};

use crate::{config::Config, utils::get_current_date, errors::SubjectNotFound};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    #[serde(rename = "start-date")]
    pub start_date: String,
    #[serde(rename = "end-date")]
    pub end_date: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>
}

impl HistoryRecord {
    fn update_end_date(self, end_date: String) -> Self {
        Self {
            start_date: self.start_date,
            end_date,
            title: self.title,
            section: self.section,
        }
    }
}

impl Display for HistoryRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.section {
            Some(sect) => write!(f, "{} | {} | {} | {}", self.start_date, self.end_date, self.title, sect),
            None => write!(f, "{} | {} | {} |", self.start_date, self.end_date, self.title),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    #[serde(rename = "record")]
    pub records: Vec<HistoryRecord>
}

impl History {
    pub fn new(records: Vec<HistoryRecord>) -> Self {
        Self { records }
    }
}

pub fn add_to_tmp(title: &str, section: &Option<String>, config: &Config) -> Result<(), Box<dyn Error>> {
    if let Some(sect) =  &section {
        if !config.subjects.all_subjects.contains(sect) {
            return Err(Box::new(SubjectNotFound(sect.clone())))
        }
    }
    let d = get_current_date(true, "-", ":");
    let reader = OpenOptions::new()
    .read(true)
    .open(&config.history.tmp)?;
    let mut tmp_records: Vec<HistoryRecord> = from_reader(&reader)?;
    let most_recent_record = tmp_records.pop();
    match most_recent_record {
        Some(rec) => {
            tmp_records.push(rec.update_end_date(d.clone()));
            tmp_records.push(HistoryRecord {
                start_date: d.clone(),
                end_date: d.clone(),
                title: title.to_owned(),
                section: section.clone()
            });
        },
        None => {
            tmp_records.push(HistoryRecord {
                start_date: d.clone(),
                end_date: d.clone(),
                title: title.to_owned(),
                section: section.clone()
            });
        },
    }
    let writer = OpenOptions::new()
    .write(true)
    .open(&config.history.tmp)?;
    to_writer_pretty::<_, Vec<HistoryRecord>>(&writer, &tmp_records)?;
    Ok(())
}

fn clear_tmp(config: &Config) -> Result<(), Box<dyn Error>> {
    let writer = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(&config.history.tmp)?;
    to_writer_pretty::<_, Vec<HistoryRecord>>(writer, &Vec::new())?;
    Ok(())
}

pub fn save_history(config: &Config) -> Result<(), Box<dyn Error>> {
    let d = get_current_date(false, "-", "_");
    let out_filepath = format!("{}/{}.log", config.history.path, d);
    let tmp_reader = OpenOptions::new()
    .read(true)
    .open(&config.history.tmp)?;
    let mut history_writer = OpenOptions::new()
    .create(true)
    .append(true)
    .open(out_filepath)?;
    let mut tmp_records: Vec<HistoryRecord> = from_reader(&tmp_reader)?;
    let latest_record = tmp_records.pop();
    if let Some(rec) = latest_record {
        let d2 = get_current_date(true, "-", ":");
        tmp_records.push(rec.update_end_date(d2));
    }
    let records_fmt = tmp_records.iter()
    .fold(String::new(), |acc, v| format!("{}\n{}", acc, v));
    history_writer.write_all(records_fmt.as_bytes())?;
    clear_tmp(config)?;
    Ok(())
}