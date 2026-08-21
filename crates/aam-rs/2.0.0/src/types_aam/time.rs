use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::ExecutionContext;
use crate::types_aam::TypeAAM;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeTypes {
    DateTime,
    Duration,
    Year,
    Day,
    Hour,
    Minute,
}

/// Returns `true` when `date` is a structurally valid `YYYY-MM-DD` string.
fn validate_date_part(date: &str) -> bool {
    let parts: Vec<&str> = date.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts[0].parse::<u32>().is_ok()
        && parts[1].parse::<u32>().is_ok()
        && parts[2].parse::<u32>().is_ok()
}

/// Validates an ISO 8601 date (`YYYY-MM-DD`) or datetime (`YYYY-MM-DDTHH:MM:SS`) string.
fn validate_datetime(value: &str) -> Result<(), AamlError> {
    if value.len() < 10 || !validate_date_part(&value[..10]) {
        return Err(AamlError::InvalidValue {
            details: format!("'{}' is not a valid ISO 8601 date/time", value),
            expected: "YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS format".to_string(),
            diagnostics: Some(ErrorDiagnostics::new(
                "Invalid datetime format",
                format!("DateTime '{}' does not match ISO 8601 format", value),
                "Use format: 2024-03-24 or 2024-03-24T14:30:00",
            )),
        });
    }
    Ok(())
}

/// Validates that `value` parses as a finite `f64` number.
fn validate_numeric(value: &str, label: &str) -> Result<(), AamlError> {
    value
        .parse::<f64>()
        .map(|_| ())
        .map_err(|_| AamlError::InvalidValue {
            details: format!("'{}' is not a valid number", value),
            expected: "floating-point number".to_string(),
            diagnostics: Some(ErrorDiagnostics::new(
                format!("Invalid {}", label),
                format!("Value '{}' cannot be parsed as a number", value),
                "Use numeric notation: 10.5, 3, -2.5, etc.",
            )),
        })
}

impl TypeAAM for TimeTypes {
    fn from_name(name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        match name {
            "datetime" => Ok(TimeTypes::DateTime),
            "duration" => Ok(TimeTypes::Duration),
            "year" => Ok(TimeTypes::Year),
            "day" => Ok(TimeTypes::Day),
            "hour" => Ok(TimeTypes::Hour),
            "minute" => Ok(TimeTypes::Minute),
            _ => Err(AamlError::NotFound {
                key: name.to_string(),
                context: "time types".to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Unknown time type",
                    format!("Time type '{}' is not recognized", name),
                    "Valid types: datetime, duration, year, day, hour, minute",
                )),
            }),
        }
    }

    fn base_type(&self) -> crate::types_aam::primitive_type::PrimitiveType {
        crate::types_aam::primitive_type::PrimitiveType::F64
    }

    fn validate(&self, value: &str, _context: &ExecutionContext) -> Result<(), AamlError> {
        match self {
            TimeTypes::DateTime => validate_datetime(value),
            TimeTypes::Duration => {
                // ISO 8601 duration (PnYnMnDTnHnMnS) or plain seconds as f64.
                if value.starts_with('P') {
                    Ok(())
                } else {
                    validate_numeric(value, "Duration")
                }
            }
            TimeTypes::Year => validate_numeric(value, "Year"),
            TimeTypes::Day => validate_numeric(value, "Day"),
            TimeTypes::Hour => validate_numeric(value, "Hour"),
            TimeTypes::Minute => validate_numeric(value, "Minute"),
        }
    }
}
