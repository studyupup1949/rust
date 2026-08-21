use crate::error::Result;

pub fn read(content: &str) -> Result<serde_json::Value> {
    Ok(serde_json::from_str(content)?)
}

pub fn write(value: &serde_json::Value) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)? + "\n")
}
