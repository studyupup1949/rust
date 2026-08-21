use chrono::Duration;
use regex::Regex;

use crate::{data::CheckResult, Answer, Day, Part, Year};

use super::{ClientError, Config};

/// Abstraction of the communication protocol used to communicate with the
/// Advent of Code backend web service enabling test mocks.
pub trait AdventOfCodeProtocol: std::fmt::Debug {
    fn get_input(&self, day: Day, year: Year) -> Result<String, ClientError>;
    fn submit_answer(
        &self,
        answer: &Answer,
        part: Part,
        day: Day,
        year: Year,
    ) -> Result<(CheckResult, Option<Duration>), ClientError>;
}

#[derive(Debug)]
pub struct AdventOfCodeHttpProtocol {
    session_id: String,
    http_client: reqwest::blocking::Client,
}

impl AdventOfCodeHttpProtocol {
    const ADVENT_OF_CODE_DOMAIN: &'static str = "adventofcode.com";
    const ADVENT_OF_CODE_URL: &'static str = "https://adventofcode.com";

    pub fn new(config: &Config) -> Self {
        // Create an HTTP client for interacting with the Advent of Code website.
        // TODO: verify dev@smacdo.com email OK
        let cookies: reqwest::cookie::Jar = Default::default();
        let cookie_data = format!(
            "session={}; Domain={}",
            config.session_id,
            Self::ADVENT_OF_CODE_DOMAIN
        );

        tracing::debug!(
            "adding session id to cookie jar with value `{}`",
            cookie_data
        );

        cookies.add_cookie_str(
            &cookie_data,
            &Self::ADVENT_OF_CODE_URL.parse::<reqwest::Url>().unwrap(),
        );

        Self {
            session_id: config.session_id.clone(),
            http_client: reqwest::blocking::ClientBuilder::new()
                .cookie_provider(cookies.into())
                .user_agent("github.com/smacdo/advent-of-code-rust [email: dev@smacdo.com]")
                .build()
                .expect("unexpected error when constructing reqwest http client"),
        }
    }
}

impl AdventOfCodeProtocol for AdventOfCodeHttpProtocol {
    fn get_input(&self, day: Day, year: Year) -> Result<String, ClientError> {
        // TODO: Convert expects and unwraps into errors.
        // TODO:  Handle "Please don't repeatedly request this endpoint before it unlocks! The calendar countdown is synchronized with the server time; the link will be enabled on the calendar the instant this puzzle becomes available.""
        let url = format!("{}/{}/day/{}/input", Self::ADVENT_OF_CODE_URL, year, day);

        tracing::debug!(
            "url to get puzzle input for day {} year {} is `{}`",
            day,
            year,
            url
        );

        let response = self.http_client.get(url).send().unwrap();
        tracing::debug!("server responed with HTTP {}", response.status());

        match response.status() {
            reqwest::StatusCode::OK => Ok(response.text().unwrap()),
            reqwest::StatusCode::BAD_REQUEST => {
                Err(ClientError::BadSessionId(self.session_id.to_string()))
            }
            reqwest::StatusCode::NOT_FOUND => {
                // TODO: Return "Not available _yet_" if the requested data in the future.
                Err(ClientError::PuzzleNotFound(day, year))
            }
            _ => Err(ClientError::Http(response.status())),
        }
    }

    fn submit_answer(
        &self,
        answer: &Answer,
        part: Part,
        day: Day,
        year: Year,
    ) -> Result<(CheckResult, Option<Duration>), ClientError> {
        // TODO: Convert expects and unwraps into errors.
        let url = format!("{}/{}/day/{}/answer", Self::ADVENT_OF_CODE_URL, year, day);

        tracing::debug!(
            "creating url to post puzzle answer for part {:?} day {} year {} answer `{}` with url = `{}`",
            part,
            day,
            year,
            answer,
            url
        );

        let response = self
            .http_client
            .post(url)
            .form(&[
                (
                    "level",
                    if part == Part::One {
                        "1".to_string()
                    } else {
                        "2".to_string()
                    },
                ),
                ("answer", answer.to_string()),
            ])
            .send()
            .unwrap();
        tracing::debug!("server responed with HTTP {}", response.status());

        // Exit early if there were any HTTP errors from the Advent of Code
        // service.
        if response.status().is_client_error() || response.status().is_server_error() {
            return match response.status() {
                reqwest::StatusCode::BAD_REQUEST => {
                    Err(ClientError::BadSessionId(self.session_id.to_string()))
                }
                reqwest::StatusCode::NOT_FOUND => {
                    // TODO: Return "Not available _yet_" if the requested data in the future.
                    Err(ClientError::PuzzleNotFound(day, year))
                }
                _ => Err(ClientError::Http(response.status())),
            };
        }

        // Parse the text contents of the response, and transform it into a
        // returnable value.
        let response_text = response.text().unwrap();
        tracing::debug!("got advent of code server response for answer: {answer}");

        // Look for a minimum wait time in the text.
        let extract_wait_time_funcs = &[
            Self::extract_error_time_to_wait,
            Self::extract_one_minute_time_to_wait,
            Self::extract_wrong_answer_time_to_wait,
        ];

        let time_to_wait = extract_wait_time_funcs
            .iter()
            .filter_map(|f| f(&response_text))
            .next();

        // Handle special cases.
        // TODO: Remove this special casing if possible.
        // TODO: Look into "You don't seem to be solving the right level.  Did you already complete it?"
        //       Is this only returned for errors on solved levels?
        if response_text.contains("gave an answer too recently") {
            return Err(ClientError::SubmitTimeOut(time_to_wait.unwrap()));
        }

        if response_text.contains("you already complete it") {
            return Err(ClientError::AlreadySubmittedAnswer(day, year));
        }

        // Translate the response text into a result.
        let responses_texts = &[
            ("not the right answer", CheckResult::Wrong),
            ("the right answer", CheckResult::Correct),
            ("answer is too low", CheckResult::TooLow),
            ("answer is too high", CheckResult::TooHigh),
        ];

        let check_result = responses_texts
            .iter()
            .find(|x| response_text.contains(x.0))
            .map(|x| x.1.clone())
            .unwrap_or_else(|| panic!("expected server response text to map to predetermined response in LUT. Response:\n```\n{response_text}\n```\n"));

        Ok((check_result, time_to_wait))
    }
}

impl AdventOfCodeHttpProtocol {
    fn extract_one_minute_time_to_wait(response: &str) -> Option<Duration> {
        match response.contains("Please wait one minute before trying again") {
            true => Some(Duration::minutes(5)),
            false => None,
        }
    }

    fn extract_wrong_answer_time_to_wait(response: &str) -> Option<Duration> {
        let regex = Regex::new(r"please wait (\d) minutes?").unwrap();
        regex
            .captures(response)
            .map(|c| Duration::minutes(c[1].parse::<i64>().unwrap()))
    }

    fn extract_error_time_to_wait(response: &str) -> Option<Duration> {
        let regex = Regex::new(r"You have (\d+)m( (\d+)s)? left to wait").unwrap();
        regex.captures(response).map(|c| {
            let mut time_to_wait = Duration::minutes(c[1].parse::<i64>().unwrap());

            if let Some(secs) = c.get(3) {
                time_to_wait += Duration::seconds(secs.as_str().parse::<i64>().unwrap());
            }

            time_to_wait
        })
    }
}
