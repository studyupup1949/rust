use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::ExecutionContext;
use crate::types_aam::TypeAAM;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveType {
    I32,
    F64,
    String,
    Bool,
    Color,
}

impl TypeAAM for PrimitiveType {
    fn from_name(name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        match name {
            "i32" => Ok(Self::I32),
            "f64" => Ok(Self::F64),
            "string" => Ok(Self::String),
            "bool" => Ok(Self::Bool),
            "color" => Ok(Self::Color),
            _ => Err(AamlError::NotFound {
                key: name.to_string(),
                context: "primitive types".to_string(),
                diagnostics: Some(Box::new(ErrorDiagnostics::new(
                    "Unknown primitive type",
                    format!("Primitive type '{name}' does not exist"),
                    "Valid types: i32, f64, string, bool, color",
                ))),
            }),
        }
    }

    fn base_type(&self) -> PrimitiveType {
        *self
    }

    fn validate(&self, value: &str, _context: &ExecutionContext) -> Result<(), AamlError> {
        match self {
            Self::I32 => {
                value.parse::<i32>().map_err(|_| AamlError::InvalidValue {
                    details: format!("'{value}' cannot be parsed as i32"),
                    expected: "32-bit signed integer".to_string(),
                    diagnostics: Some(Box::new(ErrorDiagnostics::new(
                        "Invalid integer",
                        format!("Value '{value}' is not a valid i32"),
                        "Use integer format: 42, -100, 0, etc.",
                    ))),
                })?;
            }
            Self::F64 => {
                value.parse::<f64>().map_err(|_| AamlError::InvalidValue {
                    details: format!("'{value}' cannot be parsed as f64"),
                    expected: "64-bit floating-point number".to_string(),
                    diagnostics: Some(Box::new(ErrorDiagnostics::new(
                        "Invalid floating-point number",
                        format!("Value '{value}' is not a valid f64"),
                        "Use decimal format: 3.14, 2.0, -1.5, etc.",
                    ))),
                })?;
            }
            Self::String => {
                // Any string is valid, so no validation needed.
            }
            Self::Bool => match value.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" => {}
                _ => {
                    return Err(AamlError::InvalidValue {
                        details: format!("'{value}' is not a valid boolean"),
                        expected: "true, false, 1, or 0".to_string(),
                        diagnostics: Some(Box::new(ErrorDiagnostics::new(
                            "Invalid boolean value",
                            format!("Value '{value}' is not a recognized boolean"),
                            "Use: true, false, 1, or 0",
                        ))),
                    });
                }
            },
            Self::Color => {
                // Waiting hex #RRGGBB or #RRGGBBAA
                if !value.starts_with('#') || (value.len() != 7 && value.len() != 9) {
                    return Err(AamlError::InvalidValue {
                        details: format!("'{value}' is not in valid color format"),
                        expected: "#RRGGBB or #RRGGBBAA hex format".to_string(),
                        diagnostics: Some(Box::new(ErrorDiagnostics::new(
                            "Invalid color format",
                            format!("Color '{value}' must be #RRGGBB (RGB) or #RRGGBBAA (RGBA)"),
                            "Use hex notation: #ff0000 (red), #00ff00 (green), #0000ff (blue), etc.",
                        ))),
                    });
                }
                if u64::from_str_radix(&value[1..], 16).is_err() {
                    return Err(AamlError::InvalidValue {
                        details: format!("'{value}' contains invalid hexadecimal digits"),
                        expected: "valid hexadecimal (0-9, a-f)".to_string(),
                        diagnostics: Some(Box::new(ErrorDiagnostics::new(
                            "Invalid hex in color",
                            format!("Color '{value}' contains non-hexadecimal characters"),
                            "Use only 0-9 and a-f in hex colors",
                        ))),
                    });
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::I32 => "i32",
            Self::F64 => "f64",
            Self::String => "string",
            Self::Bool => "bool",
            Self::Color => "color",
        };
        write!(f, "{s}")
    }
}
