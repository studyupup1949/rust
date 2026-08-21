use crate::error::AocError;
use crate::types::AocDatabase;
use reqwest::blocking::Client;
use std::{fmt::Display, sync::Arc, time::Instant};

use crate::error::InputError;

const USER_AGENT: &str =
    "Advent of Utils by Itron-al-Lenn found on github.com/Itron-al-Lenn/Advent-of-Utils";
const AOC_BASE_URL: &str = "https://adventofcode.com";

#[derive(Debug, Clone)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn new() -> Result<Self, InputError> {
        match std::env::var("AOC_SESSION") {
            Ok(token) => Ok(Self(token)),
            Err(e) => Err(InputError::VarError {
                key: "AOC_SESSION".to_string(),
                reason: "Faield fetching the session token".to_string(),
                source: Some(e),
            }),
        }
    }
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

fn create_client() -> Result<Client, InputError> {
    let cookie = reqwest::cookie::Jar::default();
    cookie.add_cookie_str(
        &format!("session={}", SessionToken::new()?),
        &AOC_BASE_URL.parse::<reqwest::Url>().unwrap(),
    );

    Ok(Client::builder()
        .cookie_provider(Arc::new(cookie))
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to create HTTP client"))
}

fn fetch_input(year: i32, day: u8) -> Result<String, InputError> {
    let url = format!("{}/{}/day/{}/input", AOC_BASE_URL, year, day);

    create_client()?
        .get(&url)
        .send()
        .map_err(|e| InputError::FetchFailed {
            year,
            day,
            reason: "Network request failed".to_string(),
            source: Some(e),
        })?
        .error_for_status()
        .map_err(|e| InputError::FetchFailed {
            year,
            day,
            reason: "Server returned error status".to_string(),
            source: Some(e),
        })?
        .text()
        .map_err(|e| InputError::FetchFailed {
            year,
            day,
            reason: "Failed to read response text".to_string(),
            source: Some(e),
        })
}

pub fn get_input(
    year: i32,
    day: u8,
    db: &AocDatabase,
    test: bool,
) -> Result<(String, Instant), AocError> {
    if db.has_input(year, day, test)? {
        let time = Instant::now();
        let input = db.get_input(year, day, test)?;
        Ok((input, time))
    } else if !test {
        println!("Fetching online...");
        let input = fetch_input(year, day)?;
        let time = Instant::now();
        db.set_input(year, day, test, input.clone())?;
        Ok((input, time))
    } else {
        Err(AocError::Input(InputError::NoTestInput { day }))
    }
}
