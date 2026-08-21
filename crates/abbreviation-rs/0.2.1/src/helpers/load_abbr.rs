// Loads the abbreviations.json file from the default directory

use crate::structs::Abbreviations;

use super::directory;

use serde_json::Value;
use std::collections::HashMap;
use std::fs;

pub fn load_abbr() -> HashMap<String, Value> {
    let abbr_dir = directory::get_abbr_directory().join("abbreviations.json");
    let data = fs::read_to_string(abbr_dir).expect("Error: Unable to find the abbreviations.json file. Did you run the init command?");
    let abbreviations: Abbreviations = serde_json::from_str(&data)
    .unwrap_or_else(|e| panic!("Error parsing the abbreviations.json file: {}", e));


    abbreviations.abbreviations
}