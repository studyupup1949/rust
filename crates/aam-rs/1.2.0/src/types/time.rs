use crate::error::AamlError;
use crate::types::Type;

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
        return Err(AamlError::InvalidValue(format!(
            "Invalid DateTime '{}': expected ISO 8601 format (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)",
            value
        )));
    }
    Ok(())
}

/// Validates that `value` parses as a finite `f64` number.
fn validate_numeric(value: &str, label: &str) -> Result<(), AamlError> {
    value.parse::<f64>().map(|_| ()).map_err(|_| {
        AamlError::InvalidValue(format!("Invalid {} '{}': expected a number", label, value))
    })
}

impl Type for TimeTypes {
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
            _ => Err(AamlError::NotFound(name.to_string())),
        }
    }

    fn base_type(&self) -> crate::types::primitive_type::PrimitiveType {
        crate::types::primitive_type::PrimitiveType::F64
    }

    fn validate(&self, value: &str) -> Result<(), AamlError> {
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
