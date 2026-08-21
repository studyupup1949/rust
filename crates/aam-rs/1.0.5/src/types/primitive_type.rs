use std::fmt;
use crate::error::AamlError;
use crate::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    I32,
    F64,
    String,
    Bool,
    Color,
}

impl Type for PrimitiveType {
    fn from_name(name: &str) -> Result<Self, crate::error::AamlError>
    where
        Self: Sized
    {
        match name {
            "i32" => Ok(PrimitiveType::I32),
            "f64" => Ok(PrimitiveType::F64),
            "string" => Ok(PrimitiveType::String),
            "bool" => Ok(PrimitiveType::Bool),
            "color" => Ok(PrimitiveType::Color),
            _ => Err(crate::error::AamlError::NotFound(name.to_string())),
        }
    }

    fn base_type(&self) -> PrimitiveType {
        *self
    }

    fn validate(&self, value: &str) -> Result<(), AamlError> {
        match self {
            PrimitiveType::I32 => {
                value.parse::<i32>().map_err(|_| {
                    AamlError::InvalidValue(format!("Expected i32, got '{}'", value))
                })?;
            }
            PrimitiveType::F64 => {
                value.parse::<f64>().map_err(|_| {
                    AamlError::InvalidValue(format!("Expected f64, got '{}'", value))
                })?;
            }
            PrimitiveType::String => {
                // Любая строка валидна
            }
            PrimitiveType::Bool => {
                match value.to_lowercase().as_str() {
                    "true" | "false" | "1" | "0" => {}
                    _ => return Err(AamlError::InvalidValue(format!(
                        "Expected bool (true/false/1/0), got '{}'", value
                    ))),
                }
            }
            PrimitiveType::Color => {
                // Waiting hex #RRGGBB or #RRGGBBAA
                if !value.starts_with('#') || (value.len() != 7 && value.len() != 9) {
                    return Err(AamlError::InvalidValue(format!(
                        "Expected color in #RRGGBB or #RRGGBBAA format, got '{}'", value
                    )));
                }
                if u64::from_str_radix(&value[1..], 16).is_err() {
                    return Err(AamlError::InvalidValue(format!(
                        "Invalid hex color '{}'", value
                    )));
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PrimitiveType::I32 => "i32",
            PrimitiveType::F64 => "f64",
            PrimitiveType::String => "string",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Color => "color",
        };
        write!(f, "{}", s)
    }
}