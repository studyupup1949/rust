use anyhow::{Result, Context};
use reqwest::blocking::Client;
use reqwest::header::{COOKIE, USER_AGENT};
use std::io;
use std::path::PathBuf;

pub(crate) struct AOCEnvironment {
    pub(crate) year: String,
    inputs_dir: PathBuf,
    session_cookie: String,
    http_client: Client
}

const SESSION_FILENAME: &str = "session";
const INPUT_DIRNAME: &str = "inputs";
const AOC_BASE_URL: &str = "https://adventofcode.com";
const USER_AGENT_STRING: &str = "github.com/ThePants999/advent-of-code-rust-runner by chris@chrispaterson.co.uk";

impl AOCEnvironment {
    pub(crate) fn initialize(year: &str) -> Result<Self> {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        log::debug!("Current directory: {:?}", current_dir);
        let inputs_dir = current_dir.join(INPUT_DIRNAME);
        if !inputs_dir.try_exists().context("Failed to check if inputs directory exists")? {
            log::debug!("Creating inputs directory at {:?}", inputs_dir);
            std::fs::create_dir(&inputs_dir).context("Failed to create inputs directory")?;
        }
        log::debug!("Using inputs directory at {:?}", inputs_dir);

        let session_filename = current_dir.join(SESSION_FILENAME);
        let session_value: String;
        log::debug!("Checking for session file {:?}", session_filename);
        if !session_filename.try_exists().context("Failed to check if session file exists")? {
            log::debug!("Session file not found, prompting for session cookie value");
            println!("In order to download the inputs from the Advent of Code website, this program requires your session cookie.");
            println!("Please log into the Advent of Code website, then check your browser cookies and enter the value of the 'session' cookie now.");
            let mut input = String::new();
            io::stdin().read_line(&mut input).context("Failed to read session cookie from stdin")?;
            log::debug!("Session cookie provided, saving to file");
            session_value = input.trim().to_string();
            std::fs::write(&session_filename, &session_value).context("Failed to write session file")?;
        } else {
            log::debug!("Session file found, reading session cookie value");
            session_value = std::fs::read_to_string(&session_filename).context("Failed to read session file")?.trim().to_string();
        }

        Ok(AOCEnvironment {
            year: year.to_string(),
            inputs_dir,
            session_cookie: format!("session={}", session_value),
            http_client: Client::new()
        })
    }

    pub(crate) fn fetch_input(&self, day: u8) -> Result<String> {
        let input_filename = self.inputs_dir.join(format!("day{:02}", day));
        log::debug!("Checking for input file for day {} at {:?}", day, input_filename);
        if input_filename.try_exists().context("Failed to check if input file exists")? {
            log::debug!("Input file found");
            let input = std::fs::read_to_string(&input_filename).context("Failed to read input file")?;
            return Ok(input);
        }

        log::debug!("Input file not found, fetching from Advent of Code website");
        let url = format!("{}/{}/day/{}/input", AOC_BASE_URL, self.year, day);
        let response = self.http_client
            .get(&url)
            .header(COOKIE, &self.session_cookie)
            .header(USER_AGENT, USER_AGENT_STRING)
            .send()
            .context("Failed to send request for input")?;
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch input: HTTP {}", response.status());
        }
        let input = response.text().context("Failed to read response text")?;

        log::debug!("Saving input to {:?}", input_filename);
        std::fs::write(&input_filename, &input).context("Failed to write input file")?;

        Ok(input)
    }
}