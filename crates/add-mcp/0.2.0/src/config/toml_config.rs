use crate::error::Result;

pub fn read(content: &str) -> Result<serde_json::Value> {
    let toml_value: toml::Value = toml::from_str(content)?;
    let json_str = serde_json::to_string(&toml_value)?;
    Ok(serde_json::from_str(&json_str)?)
}

pub fn write(value: &serde_json::Value) -> Result<String> {
    // serde_json::Value → toml::Value → string
    let toml_value: toml::Value = serde_json::from_value(value.clone())?;
    Ok(toml::to_string_pretty(&toml_value)?)
}
