# Advent of Code Runner for Rust

This crate is a simple framework library to help you focus on solving Advent of Code puzzles instead of wiring up input handling and timing. It:

- Downloads your input from the AoC website using your session cookie and caches it locally.
- Lets you provide optional example inputs/answers so your solution is automatically tested.
- Runs your solutions with simple CLI flags, and measures per-part timings (with optional multi-run stats).

## Quick start

1) Add the dependency - and a `log`-compatible logger. Note, the crate re-exports `anyhow::Result` and `anyhow::Context`, so downstream crates can use `Result`/`Context` without adding an `anyhow` dependency, but you might want to add the dependency anyway if you need other features like `anyhow::bail!()`.
```toml
[dependencies]
advent-of-code-rust-runner = "0.2"
log = "0.4"
env_logger = "0.11"
```

2) Implement each day by defining a type that implements `DayImplementation`. Outputs are stored/logged as strings, but your `Output` type can be any `Display + Eq` type (e.g., `i64`, `String`, or `&str`).
```rust
use advent_of_code_rust_runner::{DayImplementation, Result};

pub struct Day01;

impl DayImplementation for Day01 {
    type Output<'a> = &'a str;
    type Context<'a> = ();

    fn day(&self) -> u8 { 1 }

    fn execute_part_1<'a>(&self, input: &'a str) -> Result<(Self::Output<'a>, Self::Context<'a>)> {
        let words: Vec<&'a str> = input.split_whitespace().collect();
        Ok((words[0], ()))
    }

    fn execute_part_2<'a>(&self, input: &'a str, _ctx: Self::Context<'a>) -> Result<Self::Output<'a>> {
        let words: Vec<&'a str> = input.split_whitespace().collect();
        Ok(words[1])
    }
}
```

3) If you want, you can also pass context between your part 1 and part 2 implementations by specifying the type and returning it from your part 1 function, e.g. to avoid re-parsing the input.

```rust
pub struct Day01Context<'a> {
    words: Vec<&'a str>,
}

impl DayImplementation for Day01 {
    type Output<'a> = &'a str;
    type Context<'a> = Day01Context<'a>;

    fn day(&self) -> u8 { 1 }

    fn execute_part_1<'a>(&self, input: &'a str) -> Result<(Self::Output<'a>, Self::Context<'a>)> {
        let words: Vec<&'a str> = input.split_whitespace().collect();
        Ok((words[0], Day01Context { words }))
    }

    fn execute_part_2<'a>(&self, _input: &'a str, ctx: Self::Context<'a>) -> Result<Self::Output<'a>> {
        Ok(ctx.words[1])
    }
}
```

4) You can also optionally provide the example input/answers to enable built-in tests.
```rust
impl DayImplementation for Day01 {
    fn example_input(&self) -> Option<&'static str> { Some("hello world") }
    fn example_part_1_result(&self) -> Option<Self::Output<'static>> { Some("hello") }
    fn example_part_2_result(&self) -> Option<Self::Output<'static>> { Some("world") }
}
```

5) In `main`, register your days and run the runner:
```rust,ignore
use advent_of_code_rust_runner::{Runner, Day};
use env_logger;

mod day01;

fn main() {
    env_logger::init();
    let days: Vec<Box<dyn Day>> = vec![Box::new(day01::Day01 {})];
    let runner = Runner::new("2025", days).expect("runner init");
    runner.run(); // parses CLI args and exits on completion/failure
}
```

## Command-line flags

- `-d, --day <n>`: Run a specific day (1–25, or 1–12 for years >= 2025).
- `-a, --all`: Run all registered days.
- `-k, --skip-tests`: Skip example tests; run only real inputs.
- `-t, --tests-only`: Run only example tests.
- `-s, --stats <n>`: Run each part `n` times (>=1) and show min/median/mean/max times.
- `-h, --help`: Show usage (from clap).

By default (no `-d`/`-a`), the runner tries to infer “today” in AoC time (UTC−5) during December of the configured year.

## Input handling

On first run, you’ll be prompted for your AoC session cookie value. It’s saved to `./session`, and inputs are cached under `./inputs/dayXX`. Add both to `.gitignore` if you’re committing your solutions.
