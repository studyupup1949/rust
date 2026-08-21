use crate::converter::Converter;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub struct ConvertOptions {
    pub suffix: String,
    pub pretty: bool,
    pub pattern: Option<String>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            suffix: ".readable".to_string(),
            pretty: true,
            pattern: None,
        }
    }
}

pub struct ConvertResult {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub success: bool,
    pub error: Option<String>,
    pub item_count: Option<usize>,
}

pub fn convert_file(
    input_path: &Path,
    output_path: Option<&Path>,
    options: &ConvertOptions,
) -> ConvertResult {
    let content = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => {
            return ConvertResult {
                input_path: input_path.to_path_buf(),
                output_path: output_path.map(|p| p.to_path_buf()),
                success: false,
                error: Some(format!("Failed to read file: {e}")),
                item_count: None,
            };
        }
    };

    let abi_items = match Converter::parse_abi_content(&content) {
        Ok(items) => items,
        Err(e) => {
            return ConvertResult {
                input_path: input_path.to_path_buf(),
                output_path: output_path.map(|p| p.to_path_buf()),
                success: false,
                error: Some(format!("Failed to parse ABI: {e}")),
                item_count: None,
            };
        }
    };

    if abi_items.is_empty() {
        return ConvertResult {
            input_path: input_path.to_path_buf(),
            output_path: output_path.map(|p| p.to_path_buf()),
            success: false,
            error: Some("No valid ABI items found".to_string()),
            item_count: Some(0),
        };
    }

    let human_readable = Converter::convert_to_human_readable(&abi_items);
    let formatted = Converter::format_as_json_array(&human_readable, options.pretty);

    let final_output_path = if let Some(path) = output_path {
        path.to_path_buf()
    } else {
        let mut path = input_path.to_path_buf();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let new_name = format!("{}{}.json", stem, options.suffix);
        path.set_file_name(new_name);
        path
    };

    if let Some(parent) = final_output_path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                return ConvertResult {
                    input_path: input_path.to_path_buf(),
                    output_path: Some(final_output_path),
                    success: false,
                    error: Some(format!("Failed to create directory: {e}")),
                    item_count: Some(human_readable.len()),
                };
            }
        }
    }

    match fs::write(&final_output_path, formatted) {
        Ok(_) => ConvertResult {
            input_path: input_path.to_path_buf(),
            output_path: Some(final_output_path),
            success: true,
            error: None,
            item_count: Some(human_readable.len()),
        },
        Err(e) => ConvertResult {
            input_path: input_path.to_path_buf(),
            output_path: Some(final_output_path),
            success: false,
            error: Some(format!("Failed to write file: {e}")),
            item_count: Some(human_readable.len()),
        },
    }
}

pub fn convert_directory(
    input_dir: &Path,
    output_dir: &Path,
    options: &ConvertOptions,
) -> Vec<ConvertResult> {
    let mut results = Vec::new();

    let entries = match fs::read_dir(input_dir) {
        Ok(e) => e,
        Err(e) => {
            results.push(ConvertResult {
                input_path: input_dir.to_path_buf(),
                output_path: None,
                success: false,
                error: Some(format!("Failed to read directory: {e}")),
                item_count: None,
            });
            return results;
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    if let Some(pattern) = &options.pattern {
                        if let Some(file_name) = path.file_name() {
                            if let Some(name_str) = file_name.to_str() {
                                if !matches_pattern(name_str, pattern) {
                                    continue;
                                }
                            }
                        }
                    }

                    let relative = path.strip_prefix(input_dir).unwrap_or(&path);
                    let output_path = output_dir.join(relative);

                    results.push(convert_file(&path, Some(&output_path), options));
                }
            }
        }
    }

    results
}

pub fn convert_stdin_to_stdout(options: &ConvertOptions) -> io::Result<()> {
    let mut content = String::new();
    io::stdin().read_to_string(&mut content)?;

    let abi_items = Converter::parse_abi_content(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if abi_items.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No valid ABI items found",
        ));
    }

    let human_readable = Converter::convert_to_human_readable(&abi_items);
    let formatted = Converter::format_as_json_array(&human_readable, options.pretty);

    io::stdout().write_all(formatted.as_bytes())?;
    io::stdout().write_all(b"\n")?;

    Ok(())
}

fn matches_pattern(filename: &str, pattern: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('*').collect();

    if pattern_parts.is_empty() {
        return true;
    }

    let mut filename_pos = 0;

    for (i, part) in pattern_parts.iter().enumerate() {
        if part.is_empty() && (i == 0 || i == pattern_parts.len() - 1) {
            continue;
        }

        if let Some(pos) = filename[filename_pos..].find(part) {
            filename_pos += pos + part.len();
        } else {
            return false;
        }
    }

    if !pattern.ends_with('*') && filename_pos != filename.len() {
        return false;
    }

    true
}
