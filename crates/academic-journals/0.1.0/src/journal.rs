use serde::Deserialize;
#[derive(Debug, Deserialize, Clone)]
pub struct Journal {
    pub name: String,
    pub abbr_1: Option<String>,
    pub abbr_2: Option<String>,
    pub abbr_3: Option<String>,
}
