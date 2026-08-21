use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct LevelType {
    pub max: bool,
    pub min: bool,
    pub nom: bool,
    pub typ: bool,
}
