pub mod protocol; // TODO: rename to service

use chrono::{Datelike, Duration};
use protocol::{AdventOfCodeService, ServiceConnector};
use regex::Regex;
use thiserror::Error;

use crate::{
    cache::{CacheError, PuzzleCache, PuzzleFsCache, SessionCache, SessionFsCache},
    client::protocol::ServiceError,
    config::{load_config, Config, ConfigError},
    data::{Answers, CheckResult, Puzzle},
    utils::get_puzzle_unlock_time,
    Answer, Day, Part, Year,
};

const HTTP_BAD_REQUEST: u16 = 400;
const HTTP_NOT_FOUND: u16 = 404;

/// Errors that can occur when interacting with the Advent of Code service.
#[derive(Debug, Error)]
pub enum ClientError {
    /// The answer was submitted too soon. The `DateTime` indicates when submission will be allowed.
    #[error("the answer was submitted too soon, please wait until {} trying again", .0)]
    TooSoon(chrono::DateTime<chrono::Utc>),
    /// A session id was expected but not provided.
    #[error(
        "session cookie required; read the advent-of-code-data README for instructions on setting this"
    )]
    SessionIdRequired,
    /// The session ID is invalid or has expired.
    #[error("the session id `{:?}` is invalid or has expired", .0)]
    BadSessionId(String),
    /// The puzzle for the given day and year could not be found.
    #[error("a puzzle could not be found for day {} year {}", .0, .1)]
    PuzzleNotFound(Day, Year),
    /// A submission timeout is active; the `Duration` indicates how long to wait before retrying.
    #[error("please wait {} before submitting another answer to the Advent of Code service", .0)]
    SubmitTimeOut(chrono::Duration),
    /// A correct answer has already been submitted for this puzzle.
    #[error("a correct answer has already been submitted for this puzzle")]
    AlreadySubmittedAnswer,
    /// An unexpected HTTP error was returned by the Advent of Code service.
    #[error("an unexpected HTTP {} error was returned by the Advent of Code service", .0)]
    ServerHttpError(u16),
    /// An error occurred while reading cached data.
    #[error("an unexpected error {} error happened when reading cached data", .0)]
    CacheError(#[from] CacheError),
    /// An error occured while loading configuration values.
    #[error("an unexpected error {} happened when reading configuration values", .0)]
    SettingsError(#[from] ConfigError),
    #[error("{}", .0)]
    ReqwestError(#[from] reqwest::Error),
}

/// Primary abstraction for interacting with the Advent of Code service.
///
/// This trait provides methods to fetch puzzle inputs, submit answers, and retrieve cached puzzle
/// data. Implementors of this trait must cache inputs and answers to minimize requests to the AoC
/// service.
///
/// # Caching Behavior
///
/// - **Inputs** are cached with encryption (configured at client creation). `get_input()` returns
///   cached data if possible.
/// - **Answers** are cached unencrypted. `submit_answer()` checks the cache first before submitting
///   to the service.
/// - **Submission timeouts** are persisted and enforced by the client. If a submission fails with a
///   retry timeout, the client will refuse further submissions until the timeout expires.
///
/// # Timezone Handling
///
/// The client uses **Eastern Time (UTC-5/-4)** for determining puzzle availability, matching the
/// Advent of Code event timezone. Internally, times are stored in UTC. Puzzle availability is based
/// on the Eastern Time date/time.
///
/// # Submission Constraints
///
/// The Advent of Code service enforces rate limiting on answer submissions:
/// - You can submit one answer per puzzle part per minute.
/// - After submitting an incorrect answer, you must wait an increasing duration before the next
///   attempt.
/// - After submitting a correct answer, that part is locked and cannot be resubmitted.
pub trait Client {
    /// Returns the list of available puzzle years starting at 2015. The current year is included
    /// when the current month is December.
    fn years(&self) -> Vec<Year>;
    /// Returns the list of available puzzle days for a given year. `None` is returned when `year`
    /// is the current year, and the current month is not December.
    fn days(&self, year: Year) -> Option<Vec<Day>>;
    /// Fetches the puzzle input for a given day and year. Cached inputs are returned without
    /// fetching from the service.
    fn get_input(&self, day: Day, year: Year) -> Result<String, ClientError>;
    /// Submits an answer for a puzzle part. Cached answers are returned immediately without
    /// submitting to the service.
    fn submit_answer(
        &mut self,
        answer: Answer,
        part: Part,
        day: Day,
        year: Year,
    ) -> Result<CheckResult, ClientError>;
    /// Fetches the complete puzzle data (input and cached answers) for a given day and year.
    fn get_puzzle(&self, day: Day, year: Year) -> Result<Puzzle, ClientError>;
}

/// HTTP-based implementation of the `Client` trait that talks with the Advent of Code website.
///
/// # Initialization Patterns
///
/// 1. **`new()`** - Creates a client with default configuration. Requires a valid user config, a
///    config in the local directory, or the `AOC_SESSION_ID` and `AOC_PASSPHRASE` environment
///    variables to be set.
///
/// 2. **`with_options(ClientOptions)`** - Creates a client with custom configuration options
///    (directories, passphrase, etc.). This is the standard path for most use cases.
///
/// 3. **`with_custom_impl(ClientConfig, Box<dyn AdventOfCodeProtocol>)`** - For testing usage.
///    Allows callers to inject a mock HTTP implementation. Caches are still created automatically
///    from the config.
///
/// # Dependencies
///
/// - **Session ID**: Required for authentication. Must be a valid Advent of Code session cookie.
/// - **Network Access**: Required for fetching new puzzles and submitting answers.
/// - **Passphrase**: Used to encrypt cached puzzle inputs on disk (as requested by AoC maintainer).
/// - **Cache Directories**: Created automatically if missing.
pub struct WebClient {
    /// The client configuration (session ID, cache directories, passphrase, etc)
    pub config: Config,
    protocol: Box<dyn ServiceConnector>,
    /// Stores encrypted puzzle inputs and answer data.
    pub puzzle_cache: Box<dyn PuzzleCache>,
    /// Stores submission timeout state.
    pub session_cache: Box<dyn SessionCache>,
}

impl WebClient {
    /// Creates a client with default configuration from environment variables.
    pub fn new() -> Result<Self, ClientError> {
        Ok(Self::with_config(load_config()?.build()?))
    }

    /// Creates a client with custom configuration options.
    pub fn with_config(config: Config) -> Self {
        let advent_protocol = Box::new(AdventOfCodeService {});
        Self::with_custom_impl(config, advent_protocol)
    }

    /// Creates a client with a custom HTTP protocol implementation.
    ///
    /// Useful for testing or using an alternative HTTP backend. Caches are automatically created
    /// from the provided config.
    pub fn with_custom_impl(config: Config, advent_protocol: Box<dyn ServiceConnector>) -> Self {
        // Convert client options into a actual configuration values.
        // TODO: validate config settings are sane.
        let puzzle_dir = config.puzzle_dir.clone();
        let sessions_dir = config.sessions_dir.clone();
        let passphrase = config.passphrase.clone();

        // Print configuration settings to debug log.
        tracing::debug!("puzzle cache dir: {puzzle_dir:?}");
        tracing::debug!("sessions dir: {sessions_dir:?}");
        tracing::debug!("using encryption: {}", !passphrase.is_empty());

        Self {
            config,
            protocol: advent_protocol,
            puzzle_cache: Box::new(PuzzleFsCache::new(puzzle_dir, Some(passphrase))),
            session_cache: Box::new(SessionFsCache::new(sessions_dir)),
        }
    }
}

impl Client for WebClient {
    fn years(&self) -> Vec<Year> {
        let start_time = self.config.start_time;
        let unlock_time = get_puzzle_unlock_time(start_time.year().into());

        let mut end_year = start_time.year();

        if start_time < unlock_time {
            end_year -= 1;
        }

        (2015..(end_year + 1)).map(|y| y.into()).collect()
    }

    fn days(&self, year: Year) -> Option<Vec<Day>> {
        let start_time = self.config.start_time;
        let eastern_start_time = start_time.with_timezone(&chrono_tz::US::Eastern);
        let requested_year = year.0 as i32;

        match (
            eastern_start_time.year().cmp(&requested_year),
            eastern_start_time.month() == 12,
        ) {
            (std::cmp::Ordering::Equal, true) => Some(
                (1..(eastern_start_time.day() + 1))
                    .map(|d| d.into())
                    .collect(),
            ),
            (std::cmp::Ordering::Greater, _) => Some((0..25).map(|d| d.into()).collect()),
            _ => None,
        }
    }

    fn get_input(&self, day: Day, year: Year) -> Result<String, ClientError> {
        // TODO: Convert expects and unwraps into errors.
        // TODO:  Handle "Please don't repeatedly request this endpoint before it unlocks! The calendar countdown is synchronized with the server time; the link will be enabled on the calendar the instant this puzzle becomes available.""
        // TODO: Convert trace into span.
        tracing::trace!("get_input(day=`{day}`, year=`{year}`)",);

        // Check if the input for this puzzle is cached locally before fetching it from the Advent
        // of Code service.
        if let Some(input) = self
            .puzzle_cache
            .load_input(day, year)
            .map_err(ClientError::CacheError)?
        {
            return Ok(input);
        }

        // Fetch the puzzle input from the Advent of Code service. Try to catch common error cases
        // so we can return an exact `ClieError` type to the caller, rather than a generic HTTP
        // status code.
        match self.protocol.get_input(
            day,
            year,
            &self
                .config
                .session_id
                .as_ref()
                .cloned()
                .ok_or(ClientError::SessionIdRequired)?,
        ) {
            Ok(input_text) => {
                assert!(!input_text.is_empty());

                // Cache the puzzle input on disk before returning to avoid repeatedly fetching
                // input from the Advent of Code service.
                self.puzzle_cache.save_input(&input_text, day, year)?;
                Ok(input_text)
            }
            Err(ServiceError::HttpStatusError(HTTP_BAD_REQUEST)) => Err(ClientError::BadSessionId(
                self.config
                    .session_id
                    .clone()
                    .expect("already checked that session id was provided"),
            )),
            Err(ServiceError::HttpStatusError(HTTP_NOT_FOUND)) => {
                // TODO: Return "Not available _yet_" if the requested data in the future.
                Err(ClientError::PuzzleNotFound(day, year))
            }
            Err(ServiceError::HttpStatusError(c)) => Err(ClientError::ServerHttpError(c)),
            Err(ServiceError::ReqwestError(x)) => Err(ClientError::ReqwestError(x)),
        }
    }

    fn submit_answer(
        &mut self,
        answer: Answer,
        part: Part,
        day: Day,
        year: Year,
    ) -> Result<CheckResult, ClientError> {
        tracing::trace!(
            "submit_answer(answer=`{:?}`, part=`{}`, day=`{}`, year=`{}`)",
            answer,
            part,
            day,
            year
        );

        // Can this answer be checked locally using the cache?
        let mut answers = match self.puzzle_cache.load_answers(part, day, year)? {
            Some(cached_answers) => {
                if let Some(check_result) = cached_answers.check(&answer) {
                    tracing::debug!("answer check result was found in the cache {check_result:?}");
                    return Ok(check_result);
                }

                cached_answers
            }
            _ => Answers::new(),
        };

        // Check if there is an active time out on new submissions prior to submitting to the
        // advent of code service.
        let mut session = self.session_cache.load(
            self.config
                .session_id
                .as_ref()
                .ok_or(ClientError::SessionIdRequired)?,
        )?;

        if let Some(submit_wait_until) = session.submit_wait_until {
            if self.config.start_time <= submit_wait_until {
                tracing::warn!("you cannot submit an answer until {submit_wait_until}");
                return Err(ClientError::SubmitTimeOut(
                    submit_wait_until - self.config.start_time,
                ));
            } else {
                // TODO: remove the timeout and save.
                tracing::debug!("the submission timeout has expired, ignoring");
            }
        }

        // Submit to the answer to Advent of Code service.
        match self.protocol.submit_answer(
            &answer,
            part,
            day,
            year,
            &self
                .config
                .session_id
                .as_ref()
                .cloned()
                .ok_or(ClientError::SessionIdRequired)?,
        ) {
            Ok(response_text) => {
                assert!(!response_text.is_empty());
                let (check_result, maybe_time_to_wait) = parse_submit_response(&response_text)?;

                // Write back the amount of time to wait to avoid hitting the server
                // on future submissions.
                if let Some(time_to_wait) = maybe_time_to_wait {
                    let wait_until = chrono::Utc::now() + time_to_wait;
                    tracing::debug!("setting time to wait ({time_to_wait}) to be {wait_until}");

                    session.submit_wait_until = Some(wait_until);
                    self.session_cache.save(&session)?;
                }

                // Write the response to the answers database and then save it back to
                // the puzzle cache.
                match check_result {
                    CheckResult::Correct => {
                        tracing::debug!("Setting correct answer as {answer}");
                        answers.set_correct_answer(answer);
                    }
                    CheckResult::Wrong => {
                        tracing::debug!("Setting wrong answer {answer}");
                        answers.add_wrong_answer(answer);
                    }
                    CheckResult::TooLow => {
                        tracing::debug!("Setting low bounds wrong answer {answer}");
                        answers.set_low_bounds(answer);
                    }
                    CheckResult::TooHigh => {
                        tracing::debug!("Setting high bounds wrong answer {answer}");
                        answers.set_high_bounds(answer);
                    }
                };

                tracing::debug!("Saving answers database to puzzle cache");
                self.puzzle_cache.save_answers(&answers, part, day, year)?;

                Ok(check_result)
            }
            Err(ServiceError::HttpStatusError(HTTP_BAD_REQUEST)) => Err(ClientError::BadSessionId(
                self.config
                    .session_id
                    .clone()
                    .expect("already checked that session id was provided"),
            )),
            Err(ServiceError::HttpStatusError(HTTP_NOT_FOUND)) => {
                // TODO: Return "Not available _yet_" if the requested data in the future.
                Err(ClientError::PuzzleNotFound(day, year))
            }
            Err(ServiceError::HttpStatusError(c)) => Err(ClientError::ServerHttpError(c)),
            Err(ServiceError::ReqwestError(x)) => Err(ClientError::ReqwestError(x)),
        }
    }

    fn get_puzzle(&self, day: Day, year: Year) -> Result<Puzzle, ClientError> {
        Ok(Puzzle {
            day,
            year,
            input: self.get_input(day, year)?,
            part_one_answers: self
                .puzzle_cache
                .load_answers(Part::One, day, year)?
                .unwrap_or_default(),
            part_two_answers: self
                .puzzle_cache
                .load_answers(Part::Two, day, year)?
                .unwrap_or_default(),
        })
    }

    // TODO: personal leaderboard
    // TODO: list of private leaderboards
    // TODO: show private leaderboard
}

// TODO: document this function
// Converts the HTML response text into a check result and optional time to wait.
fn parse_submit_response(
    response_text: &str,
) -> Result<(CheckResult, Option<Duration>), ClientError> {
    // Look for a minimum wait time in the text.
    let extract_wait_time_funcs = &[
        extract_error_time_to_wait,
        extract_one_minute_time_to_wait,
        extract_wrong_answer_time_to_wait,
    ];

    let time_to_wait = extract_wait_time_funcs
        .iter()
        .filter_map(|f| f(response_text))
        .next();

    // Handle special cases.
    // TODO: Remove this special casing if possible.
    // TODO: Look into "You don't seem to be solving the right level.  Did you already complete it?"
    //       Is this only returned for errors on solved levels?
    if response_text.contains("gave an answer too recently") {
        return Err(ClientError::SubmitTimeOut(time_to_wait.unwrap()));
    }

    if response_text.contains("you already complete it") {
        return Err(ClientError::AlreadySubmittedAnswer);
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

// TODO: refactor these functions below or move them back into parse_submit_response.

/// Parses `response` and returns a one minute duration if `response` has text indicating the
/// timeout should be one minute.
fn extract_one_minute_time_to_wait(response: &str) -> Option<Duration> {
    match response.contains("Please wait one minute before trying again") {
        true => Some(Duration::minutes(1)),
        false => None,
    }
}

/// Parses `response` and returns a time to wait if the text was succesfully parsed.
fn extract_wrong_answer_time_to_wait(response: &str) -> Option<Duration> {
    let regex = Regex::new(r"please wait (\d) minutes?").unwrap();
    regex
        .captures(response)
        .map(|c| Duration::minutes(c[1].parse::<i64>().unwrap()))
}

/// Parses `response` and returns a time to wait if the text was succesfully parsed.
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

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime, TimeZone};
    use chrono_tz::US::Eastern;

    use crate::config::ConfigBuilder;

    use super::*;

    fn web_client_with_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> WebClient {
        WebClient::with_config(
            ConfigBuilder::new()
                .with_session_id("UNIT_TEST_SESSION_ID")
                .with_passphrase("UNIT_TEST_PASSWORD")
                .with_puzzle_dir("DO_NOT_USE")
                .with_fake_time(
                    Eastern
                        .from_local_datetime(
                            &NaiveDate::from_ymd_opt(year, month, day)
                                .unwrap()
                                .and_time(NaiveTime::from_hms_opt(hour, min, sec).unwrap()),
                        )
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                )
                .build()
                .unwrap(),
        )
    }

    #[test]
    fn list_years_when_date_is_after_start() {
        let client = web_client_with_time(2018, 12, 1, 0, 0, 0);
        assert_eq!(
            client.years(),
            vec![Year(2015), Year(2016), Year(2017), Year(2018)]
        );
    }

    #[test]
    fn list_years_when_date_is_before_start() {
        let client = web_client_with_time(2018, 11, 30, 23, 59, 59);
        assert_eq!(client.years(), vec![Year(2015), Year(2016), Year(2017)]);
    }

    #[test]
    fn list_years_when_date_aoc_start() {
        let client = web_client_with_time(2015, 3, 10, 11, 15, 7);
        assert_eq!(client.years(), vec![]);
    }

    #[test]
    fn list_days_before_start() {
        let client = web_client_with_time(2020, 11, 30, 23, 59, 59);
        assert_eq!(client.days(Year(2020)), None);
    }

    #[test]
    fn list_days_at_start() {
        let client = web_client_with_time(2020, 12, 1, 0, 0, 0);
        assert_eq!(client.days(Year(2020)), Some(vec![Day(1)]));
    }

    #[test]
    fn list_days_in_middle() {
        let client = web_client_with_time(2020, 12, 6, 0, 0, 0);
        assert_eq!(
            client.days(Year(2020)),
            Some(vec![Day(1), Day(2), Day(3), Day(4), Day(5), Day(6)])
        );
    }
}
