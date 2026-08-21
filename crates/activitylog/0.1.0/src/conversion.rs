//! # Purpose
//! This module gathers all functions needed to convert history content into standard formats, such as :
//! - CSV
//! - JSON
//! - XML

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{create_dir, OpenOptions};
use std::io::Write;

use serde::{Deserialize, Serialize};
use crate::utils::{get_current_date, DirContent};
use crate::history::{History, HistoryRecord};
use crate::Format;

fn convert_file_to_records(filename: &str, content: String) -> (Vec<HistoryRecord>, Vec<ConversionError>) {
    let mut reader = csv::ReaderBuilder::new()
    .has_headers(false)
    .trim(csv::Trim::All)
    .delimiter(b'|')
    .from_reader(content.as_bytes());
    let mut records: Vec<HistoryRecord> = Vec::new();
    let mut errors: Vec<ConversionError> = Vec::new();
    for (line, result) in reader.deserialize::<HistoryRecord>().enumerate() {
        match result {
            Ok(rec) => records.push(rec),
            Err(e) => {errors.push(ConversionError::new(filename.to_owned(), line, Box::new(e)));},
        }
    }
    (records, errors)
}

fn convert_to_records(content: DirContent) -> (BTreeMap<String, Vec<HistoryRecord>>, Vec<ConversionError>) {
    let mut records: BTreeMap<String, Vec<HistoryRecord>> = BTreeMap::new();
    let mut errors: Vec<ConversionError> = Vec::new();
    match content {
        DirContent::SingleFile(fname, fcontent) => {
            let (rec, mut err) = convert_file_to_records(&fname, fcontent);
            records.insert(fname.clone(), rec);
            errors.append(&mut err);
        },
        DirContent::DirectoryFiles(files) => {
            for (fname, fcontent) in files {
                let (rec, mut err) = convert_file_to_records(&fname, fcontent);
                records.insert(fname.clone(), rec);
                errors.append(&mut err);
            }
        },
    }
    (records, errors)
}

fn save_errors(dir_path: &String, errors: Vec<ConversionError>) -> Result<(), Box<dyn Error>>{
    let d = get_current_date(true, "_", "_");
    let p = format!("{dir_path}/{d}.json");
    let filepath = p.as_str();
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(filepath)?;
    let errors_as_json = serde_json::to_string_pretty::<Vec<ConversionError>>(&errors)?;
    file.write_all(errors_as_json.as_bytes())?;
    Ok(())
}

fn merge_records(records: BTreeMap<String, Vec<HistoryRecord>>) -> Vec<HistoryRecord> {
    let mut all_records: Vec<HistoryRecord> = Vec::new();
    for (_, mut frecords) in records {
        all_records.append(&mut frecords);
    }
    all_records
}

fn save_to_csv(records: BTreeMap<String, Vec<HistoryRecord>>, out_path: &String, merge: &bool) -> Result<(), Box<dyn Error>> {
    let d = get_current_date(true, "_", "_");
    if *merge {    
        let merged_records = merge_records(records);
        let mut writer = csv::Writer::from_path(format!("{out_path}/csv_{d}.csv"))?;
        for rec in merged_records {
            writer.serialize(rec)?;
        }
        writer.flush()?;
        
    } else {
        create_dir(format!("{out_path}/csv_{d}"))?;
        for (fname, frecords) in records {
            let mut writer = csv::Writer::from_path(format!("{out_path}/csv_{d}/{fname}.csv"))?;
            for rec in frecords {
                writer.serialize(rec)?;
            }
            writer.flush()?;
        }
    }
    Ok(())
}

fn save_to_json(records: BTreeMap<String, Vec<HistoryRecord>>, out_path: &String, merge: &bool) -> Result<(), Box<dyn Error>> {
    let d = get_current_date(true, "_", "_");
    if *merge {
        let merged_records = merge_records(records);
        let writer = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{out_path}/json_{d}.json"))?;
    serde_json::to_writer_pretty(&writer, &merged_records)?;
} else {
        create_dir(format!("{out_path}/json_{d}"))?;
        for (fname, frecords) in records {
            let writer = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{out_path}/json_{d}/{fname}.json"))?;
        for rec in frecords {
            serde_json::to_writer_pretty(&writer, &rec)?;
        }
    }
}

Ok(())
}

fn save_to_xml(records: BTreeMap<String, Vec<HistoryRecord>>, out_path: &String, merge: &bool) -> Result<(), Box<dyn Error>> {
    let d = get_current_date(true, "_", "_");
    if *merge {
        let history = History::new(merge_records(records));
        let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{out_path}/xml_{d}.xml"))?;
    let se_res = quick_xml::se::to_string(&history)?;
    file.write_all(se_res.as_bytes())?;
} else {
    create_dir(format!("{out_path}/xml_{d}"))?;
    for (fname, frecords) in records {
            let history = History::new(frecords);
            let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{out_path}/xml_{d}/{fname}.xml"))?;
            let se_res = quick_xml::se::to_string(&history)?;
            file.write_all(se_res.as_bytes())?;
        }
    }
    Ok(())
}

pub fn convert_to(out_path: &String, error_path: &String, content: DirContent, format: &Format, merge: &bool) -> Result<(), Box<dyn Error>> {
    let (records, errors) = convert_to_records(content);
    save_errors(error_path, errors)?;
    match format {
        Format::Csv => save_to_csv(records, out_path, merge),
        Format::Json => save_to_json(records, out_path, merge),
        Format::Xml => save_to_xml(records, out_path, merge),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ConversionError {
    source: String,
    line_number: usize,
    error_message: String
}

impl ConversionError {
    pub fn new(source: String, line_number: usize, error: Box<dyn Error>) -> Self {
        Self { source, line_number, error_message: error.to_string() }
    }
}