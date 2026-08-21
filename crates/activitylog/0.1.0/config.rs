use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;

use regex::Regex;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Config {
    pub history: History,
    pub subjects: Subjects,
    pub conversion: Conversion,
    pub samples: Samples,
}

#[derive(Serialize, Debug)]
pub struct History {
    pub path: String,
    pub tmp: String,
}

#[derive(Serialize, Debug)]
pub struct Subjects {
    pub all_subjects: Vec<String>,
    pub current_subject: String,
}

#[derive(Serialize, Debug)]
pub struct Conversion {
    pub directory_path: String,
    pub error_path: String,
}

#[derive(Serialize, Debug)]
pub struct Samples {
    pub sample_directory: String,
    pub sample_file_path: String,
    pub sample_output_path: String,
    pub start_day: String,
    pub end_day: String,
    pub start_day_time: String,
    pub end_day_time: String,
    pub minimum_task_amount_per_day: usize,
}

impl Config {
    pub fn new() -> Self {
        let home_var = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            String::from("HOME")
        } else {
            String::from("HOMEPATH")

        };
        Self {
            history: History::new(&home_var),
            subjects: Subjects::new(),
            conversion: Conversion::new(&home_var),
            samples: Samples::new(&home_var),
        }
    }

    fn process_path(path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut res = path.to_string();
        if path.contains("$") {
            let re = Regex::new(r"\$([a-zA-Z_]+)")?;
            for caps in re.captures_iter(path) {
                let var_value = std::env::var(&caps[1])?;
                res = res.replace(&caps[0], &var_value)
            }
            Ok(res)
        } else {
            Ok(path.to_string())
        }
    }

    pub fn to_toml(&self) -> Result<String, Box<dyn Error>> {
        let res = toml::ser::to_string_pretty(&self)?;
        Ok(res)
    }

    pub fn create_elements(&self) -> Result<(), Box<dyn Error>> {
        let p1 = Self::process_path(&self.history.path)?;
        let p2 = Self::process_path(&self.conversion.directory_path)?;
        let p3 = Self::process_path(&self.conversion.error_path)?;
        let p4 = Self::process_path(&self.samples.sample_directory)?;
        let p5 = Self::process_path(&self.samples.sample_file_path)?;
        let p6 = Self::process_path(&self.samples.sample_output_path)?;
        let p7 = Self::process_path(&self.history.tmp)?;
        create_dir_all(p1)?;
        create_dir_all(p2)?;
        create_dir_all(p3)?;
        create_dir_all(p4)?;
        File::create(p5)?;
        File::create(p6)?;
        let mut f7 = File::create(p7)?;
        f7.write_all("[]".as_bytes())?;
        Ok(())
    }
}

impl History {
    fn new(home_var: &String) -> Self {
        Self {
            path: format!("${home_var}/.activitylog/history"),
            tmp: format!("${home_var}/.activitylog/tmp.json"),
        }
    }
}

impl Subjects {
    fn new() -> Self {
        Self { all_subjects: Vec::new(), current_subject: String::new() }
    }
}
impl Conversion {
    fn new(home_var: &String) -> Self {
        Self {
            directory_path: format!("${home_var}/.activitylog/conversion/out"),
            error_path: format!("${home_var}/.activitylog/conversion/errors"),
        }
    }
}

impl Samples {
    pub fn new(home_var: &String) -> Self {
        Self {
            sample_directory: format!("${home_var}/.activitylog/samples"),
            sample_file_path: format!("${home_var}/.activitylog/samples/example.csv"),
            sample_output_path: format!("${home_var}/.activitylog/samples/example_out.csv"),
            start_day: String::from("2024-01-01"),
            end_day: String::from("2024-12-31"),
            start_day_time: String::from("09:00"),
            end_day_time: String::from("17:00"),
            minimum_task_amount_per_day: 5,
            
        }
    }
}