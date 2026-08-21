use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
pub struct AdEventLogParser;

#[derive(Debug)]
pub struct AdEventLog {
    pub date: String,
    pub time: String,
    pub uuid: String,
    pub ip: String,
    pub price: f32,
}

#[derive(Debug, Error)]
pub enum AdEventLogError {
    #[error("Error occurred in parsing ad event log: {0}")]
    ParseError(String),

    #[error("Invalid date format in ad event log")]
    InvalidDate,

    #[error("Invalid time format in ad event log")]
    InvalidTime,

    #[error("Invalid UUID format in ad event log")]
    InvalidUuid,

    #[error("Invalid IP address format in ad event log")]
    InvalidIp,

    #[error("Invalid price format in ad event log")]
    InvalidPrice,
}

pub fn parse_date(pair: pest::iterators::Pair<Rule>) -> Result<String, AdEventLogError> {
    if pair.as_rule() == Rule::date {
        Ok(pair.as_str().to_string())
    } else {
        Err(AdEventLogError::InvalidDate)
    }
}

pub fn parse_time(pair: pest::iterators::Pair<Rule>) -> Result<String, AdEventLogError> {
    if pair.as_rule() == Rule::time {
        Ok(pair.as_str().to_string())
    } else {
        Err(AdEventLogError::InvalidTime)
    }
}

pub fn parse_uuid(pair: pest::iterators::Pair<Rule>) -> Result<String, AdEventLogError> {
    if pair.as_rule() == Rule::uuid {
        Ok(pair.as_str().to_string())
    } else {
        Err(AdEventLogError::InvalidUuid)
    }
}

pub fn parse_ip(pair: pest::iterators::Pair<Rule>) -> Result<String, AdEventLogError> {
    if pair.as_rule() == Rule::ip {
        Ok(pair.as_str().to_string())
    } else {
        Err(AdEventLogError::InvalidIp)
    }
}

pub fn parse_price(pair: pest::iterators::Pair<Rule>) -> Result<f32, AdEventLogError> {
    if pair.as_rule() == Rule::price {
        let price_str = &pair.as_str()[1..];
        price_str.parse().map_err(|_| AdEventLogError::InvalidPrice)
    } else {
        Err(AdEventLogError::InvalidPrice)
    }
}

pub fn parse_ad_event(log: &str) -> Result<AdEventLog, AdEventLogError> {
    let mut date = String::new();
    let mut time = String::new();
    let mut uuid = String::new();
    let mut ip = String::new();
    let mut price = 0.0;

    let mut pairs = AdEventLogParser::parse(Rule::ad_event, log)
        .map_err(|e| AdEventLogError::ParseError(e.to_string()))?;

    let pair = pairs
        .next()
        .ok_or_else(|| AdEventLogError::ParseError("No pair found".to_string()))?;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::date => date = pair.as_str().trim().to_string(),
            Rule::time => time = pair.as_str().trim().to_string(),
            Rule::uuid => uuid = pair.as_str().trim().to_string(),
            Rule::ip => ip = pair.as_str().trim().to_string(),
            Rule::price => {
                let price_str = pair.as_str().trim();

                let cleaned_price_str = price_str.strip_prefix('$').unwrap_or(price_str);

                price = cleaned_price_str
                    .parse::<f32>()
                    .map_err(|_| AdEventLogError::InvalidPrice)?;
            }

            _ => {}
        }
    }

    Ok(AdEventLog {
        date,
        time,
        uuid,
        ip,
        price,
    })
}
