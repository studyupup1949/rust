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
