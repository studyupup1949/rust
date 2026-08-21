#![doc = include_str!("../README.md")]
use std::str::FromStr;

use client::{Client, ClientError, WebClient};
use data::CheckResult;

pub mod cache;
pub mod client;
pub mod config;
pub mod data;

mod utils;

/// Represents a day in an Advent of Code year. Days are typically in the range
/// [1, 25].
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

/// Represents an Advent of Code year, which is a year in which there was at
/// least one Advent of Code puzzle.
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

/// Advent of Code puzzles are split into two parts - `One` and `Two`. Both
/// parts will take the same input but typically produce different answers.
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

/// Represents an Advent of Code integer or string puzzle answer. Answers may or
/// may not be valid solutions.
///
/// ```
/// use advent_of_code_data::Answer;
///
/// let string_answer = Answer::String("hello world".to_string());
/// let int_answer = Answer::Int(42);
/// ```
///
/// # Automatic Conversions
/// `Answer` implements `From` for common string and integer types:
///
///   - `String`, &str` -> `Answer::String`
///   - Numeric types (`i8`, `i16`, `i32`, `i64`, `isize`, `usize`, etc) -> `Answer::Int`
///
/// ```
/// use advent_of_code_data::Answer;
///
/// let string_answer: Answer = "hello world".into();
/// assert_eq!(string_answer, Answer::String("hello world".to_string()));
///
/// let int_answer: Answer = 42.into();
/// assert_eq!(int_answer, Answer::Int(42));
/// ```
///
/// # FromStr (string parsing)
/// `Answer` supports string parsing for both integer and string values.
///
/// ```
/// use advent_of_code_data::Answer;
///
/// let answer: Answer = "testing 123".parse::<Answer>().unwrap();
/// assert_eq!(answer, Answer::String("testing 123".to_string()));
///
/// let answer: Answer = "-5713".parse::<Answer>().unwrap();
/// assert_eq!(answer, Answer::Int(-5713));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Answer {
    String(String),
    Int(i128),
}

impl Answer {
    pub fn to_i128(&self) -> Option<i128> {
        match self {
            Answer::String(_) => None,
            Answer::Int(v) => Some(*v),
        }
    }
}

impl FromStr for Answer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<i128>()
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

impl From<&str> for Answer {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<i8> for Answer {
    fn from(value: i8) -> Self {
        Self::Int(value as i128)
    }
}

impl From<i16> for Answer {
    fn from(value: i16) -> Self {
        Self::Int(value as i128)
    }
}

impl From<i32> for Answer {
    fn from(value: i32) -> Self {
        Self::Int(value as i128)
    }
}

impl From<i64> for Answer {
    fn from(value: i64) -> Self {
        Self::Int(value as i128)
    }
}
impl From<u8> for Answer {
    fn from(value: u8) -> Self {
        Self::Int(value as i128)
    }
}

impl From<u16> for Answer {
    fn from(value: u16) -> Self {
        Self::Int(value as i128)
    }
}

impl From<u32> for Answer {
    fn from(value: u32) -> Self {
        Self::Int(value as i128)
    }
}

impl From<u64> for Answer {
    fn from(value: u64) -> Self {
        Self::Int(value as i128)
    }
}

impl From<isize> for Answer {
    fn from(value: isize) -> Self {
        Self::Int(value as i128)
    }
}

impl From<usize> for Answer {
    fn from(value: usize) -> Self {
        Self::Int(value as i128)
    }
}

/// TODO: Good documentation.
pub fn get_input(day: Day, year: Year) -> Result<String, ClientError> {
    let client = WebClient::new()?;
    client.get_input(day, year)
}

/// TODO: Good documentation.
pub fn submit_answer(
    answer: Answer,
    part: Part,
    day: Day,
    year: Year,
) -> Result<CheckResult, ClientError> {
    let mut client: WebClient = WebClient::new()?;
    client.submit_answer(answer, part, day, year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_year() {
        let year = Year(2022);
        assert_eq!(&format!("{year}"), "2022");
    }

    #[test]
    fn print_day() {
        let day = Day(22);
        assert_eq!(&format!("{day}"), "22");
    }

    #[test]
    fn print_part() {
        assert_eq!(&format!("{}", Part::One), "One");
        assert_eq!(&format!("{}", Part::Two), "Two");
    }

    #[test]
    fn print_answer() {
        assert_eq!(
            &format!("{}", Answer::String("hello world".to_string())),
            "hello world"
        );

        assert_eq!(&format!("{}", Answer::Int(42)), "42");
    }

    #[test]
    #[allow(clippy::unnecessary_cast)]
    fn int_answer_conversions() {
        let answer: Answer = (-22 as i8).into();
        assert_eq!(answer, Answer::Int(-22));

        let answer: Answer = (-1317 as i16).into();
        assert_eq!(answer, Answer::Int(-1317));

        let answer: Answer = (-100_512 as i32).into();
        assert_eq!(answer, Answer::Int(-100_512));

        let answer: Answer = (-3_183_512_681 as i64).into();
        assert_eq!(answer, Answer::Int(-3_183_512_681));

        let answer: Answer = (22 as u8).into();
        assert_eq!(answer, Answer::Int(22));

        let answer: Answer = (1317 as u16).into();
        assert_eq!(answer, Answer::Int(1317));

        let answer: Answer = (100_512 as u32).into();
        assert_eq!(answer, Answer::Int(100_512));

        let answer: Answer = (3_183_512_681 as u64).into();
        assert_eq!(answer, Answer::Int(3_183_512_681));
    }

    #[test]
    fn string_answer_conversions() {
        let answer: Answer = "hello world".to_string().into();
        assert_eq!(answer, Answer::String("hello world".to_string()));

        let answer: Answer = "testing 123".into();
        assert_eq!(answer, Answer::String("testing 123".to_string()));
    }

    #[test]
    fn parse_string_to_answer() {
        let answer: Answer = "this is text".parse::<Answer>().unwrap();
        assert_eq!(answer, Answer::String("this is text".to_string()));

        let answer: Answer = "123".parse::<Answer>().unwrap();
        assert_eq!(answer, Answer::Int(123));
    }
}
