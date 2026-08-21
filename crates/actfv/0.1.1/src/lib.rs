use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

pub fn show_usage() {
    eprintln!("Usage: actfv <aws_credentials_file> <output_tfvars_file> <profile>");
}

pub fn parse_source(
    source_file: &String,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let file = File::open(source_file)?;
    let reader = BufReader::new(file);

    let mut creds_map: HashMap<String, Vec<String>> = HashMap::new();

    let mut section_name = String::new();
    let mut section_lines: Vec<String> = Vec::new();
    for line in reader.lines().flatten() {
        match line {
            line if line.trim().starts_with('[') && line.trim().ends_with(']') => {
                let found_section_name = line.trim()[1..line.trim().len() - 1].to_string();

                if section_name != found_section_name {
                    if !section_name.is_empty() {
                        creds_map.insert(section_name, section_lines);
                    }

                    section_lines = Vec::new();
                    section_name = found_section_name;
                }
            }
            _ => section_lines.push(line),
        }
    }
    // when we get to the end of the last section and there is no next section to trigger this add
    creds_map.insert(section_name, section_lines);

    Ok(creds_map)
}

pub fn get_entries_for_profile(
    source_map: HashMap<String, Vec<String>>,
    profile: &String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let entries = source_map
        .get(profile)
        .expect("Profile not found in aws credentials");
    Ok(entries.clone())
}

pub fn write_target(
    entries: Vec<String>,
    target_file_path: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(target_file_path)?;
    let mut writer = BufWriter::new(file);

    let mut w = Vec::new();

    for entry in entries {
        if let Some((k, v)) = entry.split_once('=') {
            match k.trim() {
                "region" | "aws_access_key_id" | "aws_secret_access_key" | "aws_session_token" => {
                    writeln!(&mut w, "{} = \"{}\"", k, v.trim())?;
                }
                _ => (),
            }
        }
    }
    writer.write_all(&w[..])?;

    Ok(())
}
