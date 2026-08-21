pub use adventage_macros::{day, part1demo, part2demo};

use reqwest::blocking::Client;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use dirs::home_dir;
use std::time::Duration;

fn fetch_from_file() -> io::Result<String> {
    let mut file = File::open("input")?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn fetch_token() -> String {
    let mut path = home_dir().unwrap();
    path.push(".aoc-token");
    let mut file = File::open(path.to_str().unwrap()).unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();
    String::from(contents.trim())
}

fn write_to_file(contents: &str) -> io::Result<()> {
    let mut file = File::create("input")?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn fetch_day(year: u32, day: u32) -> String {
    if let Ok(input) = fetch_from_file() {
        input 
    } else {
        let client = Client::new();
        let uri = format!("https://adventofcode.com/{year}/day/{day}/input");
        let session = format!("session={};", fetch_token());

        let contents = client.get(uri)
            .header("Cookie", &session)
            .send().unwrap()
            .text().unwrap();

        let _ = write_to_file(&contents);
        contents
    }
}

pub fn format_runtime(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{} s", duration.as_secs())
    } else if duration.as_millis() > 0 {
        format!("{} ms", duration.as_millis())
    } else if duration.as_micros() > 0 {
        format!("{} us", duration.as_micros())
    } else if duration.as_nanos() > 0 {
        format!("{} ns", duration.as_nanos())
    } else {
        String::from("Less than one nanosecond. Did you get much better at programming?")
    }
}