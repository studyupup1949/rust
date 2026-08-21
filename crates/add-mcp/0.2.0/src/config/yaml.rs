use crate::error::Result;

pub fn read(content: &str) -> Result<serde_json::Value> {
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)?;
    let json_str = serde_json::to_string(&yaml_value)?;
    Ok(serde_json::from_str(&json_str)?)
}

pub fn write(value: &serde_json::Value) -> Result<String> {
    Ok(serde_yaml::to_string(value)?)
}
