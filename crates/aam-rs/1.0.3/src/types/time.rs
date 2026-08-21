use crate::error::AamlError;
use crate::types::primitive_type::PrimitiveType;
use crate::types::Type;

pub enum TimeTypes {
    DateTime,
    Duration,
    Year,
    Day,
    Hour,
    Minute,

}

impl Type for TimeTypes {
    fn from_name(name: &str) -> Result<Self, crate::error::AamlError>
    where
        Self: Sized
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

    fn base_type(&self) -> PrimitiveType {
        PrimitiveType::F64
    }

    fn validate(&self, value: &str) -> Result<(), AamlError> {
        match self {
            TimeTypes::DateTime => {
                // Waiting format ISO 8601: YYYY-MM-DDTHH:MM:SS или YYYY-MM-DD
                if value.len() < 10 {
                    return Err(AamlError::InvalidValue(format!(
                        "Invalid DateTime '{}': expected ISO 8601 format (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)",
                        value
                    )));
                }
                let date_part = &value[..10];
                let parts: Vec<&str> = date_part.split('-').collect();
                if parts.len() != 3
                    || parts[0].len() != 4
                    || parts[1].len() != 2
                    || parts[2].len() != 2
                    || parts[0].parse::<u32>().is_err()
                    || parts[1].parse::<u32>().is_err()
                    || parts[2].parse::<u32>().is_err()
                {
                    return Err(AamlError::InvalidValue(format!(
                        "Invalid DateTime '{}': expected ISO 8601 format",
                        value
                    )));
                }
                Ok(())
            }
            TimeTypes::Duration => {
                //  ISO 8601 duration (PnYnMnDTnHnMnS)
                if value.starts_with('P') {
                    Ok(()) 
                } else {
                    value.parse::<f64>().map_err(|_| {
                        AamlError::InvalidValue(format!(
                            "Invalid Duration '{}': expected number (seconds) or ISO 8601 duration",
                            value
                        ))
                    })?;
                    Ok(())
                }
            }
            TimeTypes::Year | TimeTypes::Day | TimeTypes::Hour | TimeTypes::Minute => {
                value.parse::<f64>().map_err(|_| {
                    AamlError::InvalidValue(format!(
                        "Invalid time value '{}': expected a number",
                        value
                    ))
                })?;
                Ok(())
            }
        }
    }
}