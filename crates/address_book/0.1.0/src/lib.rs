use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Invalid input format")]
    InvalidFormat,
}

#[derive(Parser)]
#[grammar = "./grammar.pest"]
pub struct Grammar;

pub fn parse_phone_numbers(input: &str) -> Result<(), ParseError> {
    let pairs =
        Grammar::parse(Rule::fields, input).map_err(|e| ParseError::ParsingError(e.to_string()))?;
    for pair in pairs {
        println!("Parsed: {:?}", pair);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_single_phone_number() -> Result<()> {
        let input = "123-456-7890";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for a single phone number"
        );
        Ok(())
    }

    #[test]
    fn test_parse_multiple_phone_numbers() -> Result<()> {
        let input = "123-456-7890, 555-123-4567";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for multiple phone numbers"
        );
        Ok(())
    }

    #[test]
    fn test_invalid_phone_number_format() -> Result<()> {
        let input = "123-45-67890";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_err(),
            "Expected parsing error for invalid phone number format"
        );
        Ok(())
    }
}
