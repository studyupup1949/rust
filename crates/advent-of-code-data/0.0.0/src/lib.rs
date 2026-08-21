use std::str::FromStr;

use client::WebClient;
use data::CheckResult;
use thiserror::Error;

pub mod cache;
pub mod client;
pub mod data;
pub mod registry;
pub mod settings;
pub mod utils;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Day(pub usize);

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for Day {
    fn from(value: i32) -> Self {
        assert!(value >= 0);
        Day(value as usize)
    }
}

impl From<u32> for Day {
    fn from(value: u32) -> Self {
        Day(value as usize)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Year(pub usize);

impl std::fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for Year {
    fn from(value: i32) -> Self {
        assert!(value >= 0);
        Year(value as usize)
    }
}

impl From<Year> for i32 {
    fn from(value: Year) -> Self {
        value.0 as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Part {
    One,
    Two,
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Part::One => "One",
                Part::Two => "Two",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum AnswerParseError {
    #[error("answers with empty strings or only whitespace chars are not allowed")]
    EmptyNotAllowed,
    #[error("answers with newline characters are not allowed")]
    NewlinesNotAllowed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Answer {
    String(String),
    Int(i64),
}

impl Answer {
    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Answer::String(_) => None,
            Answer::Int(v) => Some(*v),
        }
    }
}

impl FromStr for Answer {
    type Err = AnswerParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.chars().all(|c| c.is_whitespace()) {
            return Err(AnswerParseError::EmptyNotAllowed);
        }

        if s.chars().any(|c| c == '\n') {
            return Err(AnswerParseError::NewlinesNotAllowed);
        }

        Ok(s.parse::<i64>()
            .map_or_else(|_| Answer::String(s.to_string()), Answer::Int))
    }
}

impl std::fmt::Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Answer::String(v) => write!(f, "{}", v),
            Answer::Int(v) => write!(f, "{}", v),
        }
    }
}

impl From<String> for Answer {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i64> for Answer {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<i32> for Answer {
    fn from(value: i32) -> Self {
        Self::Int(value as i64)
    }
}

impl From<isize> for Answer {
    fn from(value: isize) -> Self {
        Self::Int(value as i64)
    }
}

impl From<usize> for Answer {
    fn from(value: usize) -> Self {
        assert!(value as u64 <= i64::MAX as u64);
        Self::Int(value as i64)
    }
}

pub fn get_input(day: Day, year: Year) -> String {
    let client: WebClient = Default::default();
    client.get_input(day, year)
}

pub fn submit_answer(answer: Answer, part: Part, day: Day, year: Year) -> CheckResult {
    let mut client: WebClient = Default::default();
    client.submit_answer(answer, part, day, year)
}
