use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

#[derive(Debug)]
pub struct TomlFile {
    pub authors: Option<Vec<String>>,
    pub year: Option<String>,
    pub license: Option<String>,
    pub file_path: String,
}

impl TomlFile {
    fn new() -> Self {
        Self {
            authors: None,
            year: None,
            license: None,
            file_path: String::new(),
        }
    }

    pub fn read() -> Self {
        let mut toml_file_path = env::current_dir().unwrap();
        toml_file_path.push("Cargo.toml");

        let file = File::open(&toml_file_path).unwrap();
        let reader = BufReader::new(&file);

        let mut toml_file = TomlFile::new();
        toml_file.file_path = toml_file_path.to_str().unwrap().to_string();

        for line in reader.lines() {
            let line = line.unwrap();

            if line.contains("authors") {
                toml_file.read_authors(&line);
            } else if line.contains("year") {
                toml_file.read_year(&line);
            } else if line.contains("license") {
                toml_file.read_license(&line);
            }
        }

        toml_file
    }

    pub fn write(&self) {
        let mut file_content = String::new();
        {
            let file = File::open(&self.file_path).unwrap();
            let reader = BufReader::new(file);
            for line in reader.lines() {
                file_content.push_str(&line.unwrap());
                file_content.push('\n');
            }
        }

        let mut lines: Vec<String> = file_content
            .lines()
            .map(|s| s.to_string())
            .collect();

        let mut package_index = lines
            .iter()
            .position(|x| x.trim() == "[package]")
            .unwrap_or(0) + 1;
        
        let mut authors_found = false;
        let mut year_found = false;
        let mut license_found = false;

        for line in lines.iter_mut() {
            if line.contains("authors") && self.authors.is_some() {
                *line = format!("authors = [{}]", self.format_authors());
                authors_found = true;
            }
            if line.contains("year") && self.year.is_some() {
                *line = format!("year = \"{}\"", self.year.as_ref().unwrap());
                year_found = true;
            }
            if line.contains("license") && self.license.is_some() {
                *line = format!("license = \"{}\"", self.license.as_ref().unwrap());
                license_found = true;
            }
        }

        if !authors_found && self.authors.is_some() {
            lines.insert(
                package_index, 
                format!("authors = [{}]", self.format_authors())
            );
            package_index += 1;
        }
        if !year_found && self.year.is_some() {
            lines.insert(
                package_index, 
                format!("year = \"{}\"", self.year.as_ref().unwrap())
            );
            package_index += 1;
        }
        if !license_found && self.license.is_some() {
            lines.insert(
                package_index, 
                format!("license = \"{}\"", self.license.as_ref().unwrap())
            );
        }

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .unwrap();

        writeln!(file, "{}", lines.join("\n")).unwrap();
    }

    fn format_authors(&self) -> String {
        self.authors.as_ref().unwrap()
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn read_authors(&mut self, line: &str) {
        let authors_str = line
            .split('=')
            .nth(1)
            .unwrap()
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']');

        let authors: Vec<String> = authors_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();

        self.authors = Some(authors);
    }

    fn read_year(&mut self, line: &str) {
        let year_str = line
            .split('=')
            .nth(1)
            .unwrap()
            .trim()
            .trim_matches('"');

        self.year = Some(year_str.to_string());
    }

    fn read_license(&mut self, line: &str) {
        let license_str = line
            .split('=')
            .nth(1)
            .unwrap()
            .trim()
            .trim_matches('"');

        self.license = Some(license_str.to_string());
    }
}
