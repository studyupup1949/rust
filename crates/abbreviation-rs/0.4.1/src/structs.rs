use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Abbreviations {
    pub abbreviations: HashMap<String, Value>
}