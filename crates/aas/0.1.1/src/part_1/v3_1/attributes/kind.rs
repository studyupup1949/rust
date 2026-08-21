use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct HasKind {
    pub kind: ModellingKind,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum ModellingKind {
    Instance,
    Template,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ModellingKindError {
    #[error("Unknown value")]
    UnknownValue,
}

// easier than EnumString derive from strum.
impl FromStr for ModellingKind {
    type Err = ModellingKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Instance" => ModellingKind::Instance,
            "Template" => ModellingKind::Template,
            _ => return Err(ModellingKindError::UnknownValue),
        })
    }
}
