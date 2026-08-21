pub mod json;
pub mod toml_config;
pub mod yaml;

use crate::error::Result;
use crate::types::ConfigFormat;
use std::path::Path;

/// Read a config file into a `serde_json::Value`, normalizing all formats to JSON Value.
pub fn read_config(path: &Path, format: ConfigFormat) -> Result<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }

    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }

    match format {
        ConfigFormat::Json => json::read(&content),
        ConfigFormat::Yaml => yaml::read(&content),
        ConfigFormat::Toml => toml_config::read(&content),
    }
}

/// Write a `serde_json::Value` back to disk in the appropriate format.
pub fn write_config(path: &Path, format: ConfigFormat, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = match format {
        ConfigFormat::Json => json::write(value)?,
        ConfigFormat::Yaml => yaml::write(value)?,
        ConfigFormat::Toml => toml_config::write(value)?,
    };

    std::fs::write(path, content)?;
    Ok(())
}

/// Merge an MCP server entry into a config object under the given top-level key.
/// Returns whether the server already existed (was overwritten).
pub fn merge_server(
    config: &mut serde_json::Value,
    section_key: &str,
    server_name: &str,
    server_value: serde_json::Value,
) -> bool {
    let obj = config.as_object_mut().expect("config must be an object");

    // Ensure the section exists
    if !obj.contains_key(section_key) {
        obj.insert(
            section_key.to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );
    }

    let section = obj
        .get_mut(section_key)
        .unwrap()
        .as_object_mut()
        .expect("section must be an object");

    let existed = section.contains_key(server_name);
    section.insert(server_name.to_string(), server_value);
    existed
}
