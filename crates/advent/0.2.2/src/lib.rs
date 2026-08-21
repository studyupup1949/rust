// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>

//! # Advent of Code
//!
//! `advent` is a collection of my solutions to the [Advent of Code] puzzles posted in December each
//! year. The solutions are implemented as functions in the modules (e.g. `y2018` below) and can be
//! executed using `cargo test` with the appropriate scope selection
//!
//! ```bash
//! $ cargo test 2018::day1
//! running 2 tests
//! test y2018::day1::examples ... ok
//! test y2018::day1::solution ... ok
//! ```
//!
//! [Advent of Code]: https://adventofcode.com/

pub mod y2017;
pub mod y2018;

/// Results and Optionals are unwrapped on trust
pub const TRUST: &str = "Assume trusted input";

/// Read lines from input
pub fn lines(input: &str) -> impl Iterator<Item = &str> {
    input.lines().filter_map(|s| {
        let i = s.trim();
        if i.is_empty() {
            None
        } else {
            Some(i)
        }
    })
}

/// Parse delimited segments from the input
pub fn parse<'a, T: std::str::FromStr>(
    input: &'a str,
    delimit: &'static str,
) -> impl Iterator<Item = T> + 'a {
    input
        .split(move |c| delimit.contains(c))
        .filter_map(|s| s.parse().ok())
}
