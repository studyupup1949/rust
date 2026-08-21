
//! # Purpose
//! This module gathers the most general-purpose functions used for handling all the commands provided by the user.
//! 
//! Any other more detailed functions implementing the business-logic of the commands are placed in their one modules.
//! The main one is the [`handle_command`] used in the main module.
//! 
//! # More information
//! This module has to be refactored any time too much detail has to be handled during business-logic implementation.
//! In the other hand, if any tiny miscellaneous element has to created, it can be placed right here.

use std::collections::BTreeMap;
use std::fs::read_dir;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::error::Error;
use std::path::PathBuf;
use regex::Regex;

use crate::conversion::convert_to;
use crate::history::add_to_tmp;
use crate::history::save_history;
use crate::args::{Args, Format};
use crate::config::Config;
use crate::misc::sample_generation::create_samples;
use crate::utils::config_init;
use crate::utils::process_path;
use crate::utils::DirContent;
use crate::Command;

/// Read a configuration file and returns it as a [`Config`] object
pub fn read_config(config_file_path: &str) -> Result<Config, Box<dyn Error>> {
    let config_file_path_processed = process_path(config_file_path)?;
    let mut file = OpenOptions::new()
        .read(true)
        .open(config_file_path_processed)?;
    let mut content: String = String::from("");
    let _ = file.read_to_string(&mut content)?;
    let cfg = toml::from_str::<Config>(content.as_str())?;
    Ok(cfg)
}

/// Save a [`Config`] object into a configuration file
fn save_to_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = process_path("$HOME/.activitylog/config.toml")?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_path)?;
    match toml::to_string_pretty(config) {
        Ok(v) => {
            file.write_all(v.as_bytes())?;
            Ok(())
        },
        Err(e) => {
            println!("Could not serialize the new configuration values:\n{}", e);
            Ok(())
        },
    }
}
/// Switch the current subject with an other one, by updating the sate of a [`Config`] object
fn switch_subject(config: &mut Config, subject: &String) {
    for sub in config.subjects.all_subjects.iter() {
        if sub == subject {
            println!("Switching subject: {} -------> {}", config.subjects.current_subject, subject);
            config.subjects.current_subject = subject.clone();
            return;
        }
    }
    println!("Could not find the subject in the config file...")
}

/// List all the subjects defined in a [`Config`] object
fn list_subjects(config: &Config, filter: &Option<String>) {
    if config.subjects.all_subjects.is_empty() {
        println!("No subject yet, add a new one with the following command:");
        println!("activitylog subject add <NAME>");
        return;
    }
    let flt_pat = filter.clone().map(
        |v| Regex::new(&format!("^{v}$"))
    );
    println!("Subjects:");
    match flt_pat {
        Some(Ok(re)) => for sub in config.subjects.all_subjects.iter() {
            if re.is_match(sub) {
                println!("- {sub}")
            }
        },
        None => for sub in config.subjects.all_subjects.iter() {
            println!("- {sub}")
        },
        Some(Err(e)) => println!("{e}")
    }
}

/// Add a subject into a [`Config`] object
fn add_subject(config: &mut Config, name: &str) {
    config.subjects.all_subjects.push(name.to_owned());
}

/// Remove a subject from a [`Config`] object
fn remove_subject(config: &mut Config, name: &String) {
    config.subjects.all_subjects.retain(|sub| sub != name);
}

/// Updating the name of a subject already defined in a [`Config`] object
fn update_subject(config: &mut Config, name: &String, new: &str) {
    for x in &mut config.subjects.all_subjects {
        if x == name {
            *x = new.to_owned();
        }
    }
}

fn get_file_content(path: PathBuf) -> Result<String, Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)?;
    let mut content = String::new();
    let _ = file.read_to_string(&mut content)?;
    Ok(content)
}

/// Read the content of the history
/// - `config`: [`Config`] object contained the path of the history files
/// - `all`: specifies if all the history is returned,
///     or just the most recent file (based on the filename) if set to `false`
fn read_history(config: &Config, all: &bool) -> Result<DirContent, Box<dyn Error>> {
    let dir_content = read_dir(config.history.path.clone())?;
    if *all {
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        for (index, entry) in dir_content.enumerate() {
            match entry {
                Ok(e) => {
                    let filename = e.file_name();
                    let content = get_file_content(e.path());
                    match (filename.to_str(), content) {
                        (Some(fname), Ok(ctnt)) => {files.insert(fname.to_string(), ctnt);},
                        (None, Ok(ctnt)) => {files.insert(format!("__file_nº{index}__"), ctnt);},
                        (_, Err(err)) => {return Err(err);},
                    }
                },
                Err(err) => return Err(Box::new(err)),
            }
        }
        Ok(DirContent::DirectoryFiles(files))
    } else {
        let entry = dir_content.into_iter()
        .max_by(|e1, e2| match (e1, e2) {
            (Ok(p1  ), Ok(p2)) => p1.file_name().cmp(&p2.file_name()),
            (Err(_), Err(_)) => std::cmp::Ordering::Equal,
            (Ok(_), Err(_)) => std::cmp::Ordering::Greater,
            (Err(_), Ok(_)) => std::cmp::Ordering::Less,
        });
        match entry {
            Some(Ok(e)) => match (e.file_name().to_str(), get_file_content(e.path())) {
                (Some(filename), Ok(file_content)) => Ok(DirContent::SingleFile(filename.to_string(), file_content)),
                (None, Ok(file_content)) => Ok(DirContent::SingleFile(String::from("__most_recent_file__"), file_content)),
                (_, Err(e)) => Err(e),
            },
            Some(Err(e)) => Err(Box::new(e)),
            None => Ok(DirContent::SingleFile(String::from("No file"), String::from(""))),
            
        }
    }
}

/// Convert the history content into a specific format
/// - `config`: [`Config`] object containing information about conversion
/// - `format`: specifies the format (listed in the [`Format`] enum) in which the content as to be transformed 
fn convert_history(config: &Config, format: &Format, history_content: DirContent, merge: &bool) {
    if let Err(e) = convert_to(
        &config.conversion.directory_path,
        &config.conversion.error_path, history_content,
        format,
        merge
    ) {
        println!("Conversion into {:?} - error :\n{}", format, e);
    }
}

/// Handle all the commands specified in a CLI args object (c.f. [`Args`])
pub fn handle_command(args: &Args, config: &mut Config) {
    let _ = config_init(config);
    match &args.commands {
        Command::Commit {
            title,
            section
        } => if let Err(tmp_e) = add_to_tmp(title, section, config) {
                println!("{tmp_e}");
            },
        Command::Save => if let Err(save_e) = save_history(config) {
            println!("{save_e}");
        },
        Command::Switch { subject } => {
            switch_subject(config, subject);
            if let Err(e) = save_to_config(config) {
                println!("{e}")
            }
        }
        Command::Subject { command } => match command {
            crate::Subject::List { filter } => list_subjects(config, filter),
            crate::Subject::Add { name } => {
                add_subject(config, name);
                if let Err(e) = save_to_config(config) {
                    println!("{e}")
                }
            },
            crate::Subject::Remove { name } => {
                remove_subject(config, name);
                if let Err(e) = save_to_config(config) {
                    println!("{e}")
                }
            },
            crate::Subject::Update { name, new } => {
                update_subject(config, name, new);
                if let Err(e) = save_to_config(config) {
                    println!("{e}")
                }
            }
        },
        crate::Command::Convert { format, all , merge} => {
            match read_history(config, all) {
                Ok(hist) => convert_history(config, format, hist, merge),
                Err(e) => println!("{e}"),
            };
        },
        crate::Command::Generate => if let Err(e) = create_samples(config) {
            println!("{e}")
        },
    }
}