use crate::aaml::AAML;
use crate::error::{AamlError, ErrorDiagnostics};
use crate::types::Type;
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

impl Type for PrimitiveType {
    fn from_name(name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        match name {
            "i32" => Ok(PrimitiveType::I32),
            "f64" => Ok(PrimitiveType::F64),
            "string" => Ok(PrimitiveType::String),
            "bool" => Ok(PrimitiveType::Bool),
            "color" => Ok(PrimitiveType::Color),
            _ => Err(AamlError::NotFound {
                key: name.to_string(),
                context: "primitive types".to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Unknown primitive type",
                    format!("Primitive type '{}' does not exist", name),
                    "Valid types: i32, f64, string, bool, color",
                )),
            }),
        }
    }

    fn base_type(&self) -> PrimitiveType {
        *self
    }

    fn validate(&self, value: &str, _aaml: &AAML) -> Result<(), AamlError> {
        match self {
            PrimitiveType::I32 => {
                value.parse::<i32>().map_err(|_| AamlError::InvalidValue {
                    details: format!("'{}' cannot be parsed as i32", value),
                    expected: "32-bit signed integer".to_string(),
                    diagnostics: Some(ErrorDiagnostics::new(
                        "Invalid integer",
                        format!("Value '{}' is not a valid i32", value),
                        "Use integer format: 42, -100, 0, etc.",
                    )),
                })?;
            }
            PrimitiveType::F64 => {
                value.parse::<f64>().map_err(|_| AamlError::InvalidValue {
                    details: format!("'{}' cannot be parsed as f64", value),
                    expected: "64-bit floating-point number".to_string(),
                    diagnostics: Some(ErrorDiagnostics::new(
                        "Invalid floating-point number",
                        format!("Value '{}' is not a valid f64", value),
                        "Use decimal format: 3.14, 2.0, -1.5, etc.",
                    )),
                })?;
            }
            PrimitiveType::String => {
                // Any string is valid, so no validation needed.
            }
            PrimitiveType::Bool => match value.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" => {}
                _ => {
                    return Err(AamlError::InvalidValue {
                        details: format!("'{}' is not a valid boolean", value),
                        expected: "true, false, 1, or 0".to_string(),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Invalid boolean value",
                            format!("Value '{}' is not a recognized boolean", value),
                            "Use: true, false, 1, or 0",
                        )),
                    });
                }
            },
            PrimitiveType::Color => {
                // Waiting hex #RRGGBB or #RRGGBBAA
                if !value.starts_with('#') || (value.len() != 7 && value.len() != 9) {
                    return Err(AamlError::InvalidValue {
                        details: format!("'{}' is not in valid color format", value),
                        expected: "#RRGGBB or #RRGGBBAA hex format".to_string(),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Invalid color format",
                            format!(
                                "Color '{}' must be #RRGGBB (RGB) or #RRGGBBAA (RGBA)",
                                value
                            ),
                            "Use hex notation: #ff0000 (red), #00ff00 (green), #0000ff (blue), etc.",
                        )),
                    });
                }
                if u64::from_str_radix(&value[1..], 16).is_err() {
                    return Err(AamlError::InvalidValue {
                        details: format!("'{}' contains invalid hexadecimal digits", value),
                        expected: "valid hexadecimal (0-9, a-f)".to_string(),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Invalid hex in color",
                            format!("Color '{}' contains non-hexadecimal characters", value),
                            "Use only 0-9 and a-f in hex colors",
                        )),
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
            PrimitiveType::I32 => "i32",
            PrimitiveType::F64 => "f64",
            PrimitiveType::String => "string",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Color => "color",
        };
        write!(f, "{}", s)
    }
}
