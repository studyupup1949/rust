use std::fs;
use crate::helpers;

pub fn init() {
    let abbr_dir = helpers::directory::get_abbr_directory();

    // Create the directory if it doesn't exist (which if you are running the command and it doesn't exist what's the point)
    if !abbr_dir.exists() {
        fs::create_dir_all(&abbr_dir).expect("Error: Failed to create the abbreviations directory.");
    }

    let abbr_file = include_str!("../abbreviations.json");

    let abbr_file_dir = abbr_dir.join("abbreviations.json");
    if !abbr_file_dir.exists() {
        let abbreviations = abbr_file;
        fs::write(abbr_file_dir, abbreviations).expect("Error: Failed to create the abbreviations file!");
    }
}