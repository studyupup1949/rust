#![doc = include_str!("../README.md")]

use std::{process, time::Duration};

use anyhow::anyhow;
use clap::{CommandFactory, Parser};
use time::{Month, OffsetDateTime, UtcOffset};

mod day;
mod environment;

pub use day::{Day, DayImplementation};
pub use anyhow::{Context, Result};
use day::TestCaseResult;

const DAY_SEPARATOR: &str = "-----------------------";

#[derive(Parser, Debug)]
#[command(
    name = "aoc-runner",
    about = "Run Advent of Code solutions"
)]
pub struct RunnerArgs {
    /// Run a specific day (mutually exclusive with --all)
    #[arg(
        short = 'd',
        long = "day",
        value_parser = clap::value_parser!(u8),
        conflicts_with = "all_days"
    )]
    pub specific_day: Option<u8>,

    /// Run all days
    #[arg(short = 'a', long = "all", conflicts_with = "specific_day")]
    pub all_days: bool,

    /// Skip example tests
    #[arg(short = 'k', long = "skip-tests", conflicts_with = "tests_only")]
    pub skip_tests: bool,

    /// Only run example tests
    #[arg(short = 't', long = "tests-only", conflicts_with = "skip_tests")]
    pub tests_only: bool,

    /// Number of runs for timing stats (>=1)
    #[arg(
        short = 's',
        long = "stats",
        default_value_t = 1,
        value_parser = clap::value_parser!(usize)
    )]
    pub num_runs: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct RunStats {
    min: Duration,
    max: Duration,
    median: Duration,
    mean: Duration,
}

pub struct Runner {
    env: environment::AOCEnvironment,
    days: Vec<Box<dyn Day>>
}

impl Runner {
    pub fn new(year: &str, days: Vec<Box<dyn Day>>) -> Result<Self> {
        let env = environment::AOCEnvironment::initialize(year)?;
        Ok(Self {
            env,
            days
        })
    }

    /// Owns process exit; prints errors/usage on failure.
    pub fn run_with_args(&self, args: RunnerArgs) -> ! {
        if let Err(e) = self.run_inner(args) {
            eprintln!("error: {e}");
            let _ = RunnerArgs::command().print_help();
            eprintln!();
            process::exit(1);
        }
        process::exit(0);
    }

    pub fn run(&self) -> ! {
        let args = RunnerArgs::parse();
        self.run_with_args(args)
    }

    fn run_inner(&self, args: RunnerArgs) -> Result<()> {
        let max_day = self.max_day()?;
        if let Some(day) = args.specific_day
            && (day == 0 || day > max_day)
        {
            return Err(anyhow!("Day must be between 1 and {}", max_day));
        }
        if args.num_runs == 0 {
            return Err(anyhow!("Stats runs must be at least 1"));
        }

        if args.all_days {
            self.run_all_days(&args)
        } else {
            let day_to_run = match args.specific_day {
                Some(d) => d,
                None => self
                    .current_aoc_day()
                    .context("No day specified; only allowed when running the configured year in December (AoC time)")?,
            };
            self.run_single_day(day_to_run, &args)
        }
    }

    fn current_aoc_day(&self) -> Option<u8> {
        let max_day = self.max_day().ok()?;
        // AoC is Eastern; use fixed UTC−5 to avoid extra dependencies.
        let offset = UtcOffset::from_hms(-5, 0, 0).ok()?;
        let now = OffsetDateTime::now_utc().to_offset(offset);
        if now.year().to_string() == self.env.year && now.month() == Month::December {
            let today = now.day();
            (today <= max_day).then_some(today)
        } else {
            None
        }
    }

    fn run_all_days(&self, args: &RunnerArgs) -> Result<()> {
        let mut medians = Vec::with_capacity(self.days.len());
        let mut totals = RunStats::default();
        let mut max_time = Duration::ZERO;

        for (idx, day) in self.days.iter().enumerate() {
            let stats = self.run_day(day.as_ref(), args)?;
            medians.push(stats.median);
            totals.min += stats.min;
            totals.max += stats.max;
            totals.mean += stats.mean;
            totals.median += stats.median;
            if stats.median > max_time {
                max_time = stats.median;
            }
            println!("Day {} complete\n", idx + 1);
        }

        println!("{DAY_SEPARATOR}");
        if !args.tests_only && max_time > Duration::ZERO {
            if args.num_runs < 2 {
                println!("Total time: {:?}", totals.median);
            } else {
                println!(
                    "Total time: {:?} median, {:?} mean, {:?} min, {:?} max",
                    totals.median, totals.mean, totals.min, totals.max
                );
            }

            for threshold in (1..=10).rev().map(|t| t as f32 / 10.0) {
                print!("| ");
                for t in &medians {
                    if max_time.as_secs_f64() > 0.0
                        && (t.as_secs_f64() / max_time.as_secs_f64()) >= threshold as f64
                    {
                        print!("#");
                    } else {
                        print!(" ");
                    }
                }
                println!();
            }
            print!("|-");
            println!("{}", "-".repeat(self.days.len()));
        }
        Ok(())
    }

    fn run_single_day(&self, day_number: u8, args: &RunnerArgs) -> Result<()> {
        let day = self
            .days
            .iter()
            .find(|d| d.day() == day_number)
            .with_context(|| format!("Day {} not registered", day_number))?;
        self.run_day(day.as_ref(), args).map(|_| ())
    }

    fn run_day(&self, day_impl: &dyn Day, args: &RunnerArgs) -> Result<RunStats> {
        println!("{DAY_SEPARATOR}");
        println!("Day {}", day_impl.day());

        let max_day = self.max_day()?;
        if day_impl.day() > max_day {
            return Err(anyhow!(
                "Day {} is not valid for {} (max {})",
                day_impl.day(),
                self.env.year,
                max_day
            ));
        }

        // Skip if not published yet (same year, December, AoC day not reached)
        if let Some(today) = self.current_aoc_day()
            && today < day_impl.day()
        {
            println!("Skipping - day not published yet");
            return Ok(RunStats::default());
        }

        if !args.skip_tests {
            match day_impl.test_day() {
                Ok(test_result) => {
                    self.print_test_result("Part 1", &test_result.part1);
                    self.print_test_result("Part 2", &test_result.part2);
                }
                Err(e) => log::warn!("Day {}: failed to run tests: {}", day_impl.day(), e),
            }
        }

        if args.tests_only {
            return Ok(RunStats::default());
        }

        let input = self
            .env
            .fetch_input(day_impl.day())
            .with_context(|| format!("Failed to fetch input for day {}", day_impl.day()))?;

        if args.num_runs < 2 {
            let res = day_impl
                .execute_day(input.as_str())
                .with_context(|| format!("Day {} execution failed", day_impl.day()))?;
            let total = res.part_1_time + res.part_2_time;
            println!(
                "Part 1 real: {} ({:?})\nPart 2 real: {} ({:?})\nTotal time: {:?}",
                res.part_1_result, res.part_1_time, res.part_2_result, res.part_2_time, total
            );
            return Ok(RunStats {
                min: total,
                max: total,
                median: total,
                mean: total,
            });
        }

        let mut results = Vec::with_capacity(args.num_runs);
        for _ in 0..args.num_runs {
            results.push(
                day_impl
                    .execute_day(input.as_str())
                    .with_context(|| format!("Day {} execution failed", day_impl.day()))?,
            );
        }

        let mut p1_times: Vec<_> = results.iter().map(|r| r.part_1_time).collect();
        let mut p2_times: Vec<_> = results.iter().map(|r| r.part_2_time).collect();

        let p1_stats = build_stats(&mut p1_times);
        let p2_stats = build_stats(&mut p2_times);
        let totals = RunStats {
            min: p1_stats.min + p2_stats.min,
            max: p1_stats.max + p2_stats.max,
            median: p1_stats.median + p2_stats.median,
            mean: p1_stats.mean + p2_stats.mean,
        };

        println!(
            "Part 1: {} (median {:?}, mean {:?}, min {:?}, max {:?})",
            results[0].part_1_result, p1_stats.median, p1_stats.mean, p1_stats.min, p1_stats.max
        );
        println!(
            "Part 2: {} (median {:?}, mean {:?}, min {:?}, max {:?})",
            results[0].part_2_result, p2_stats.median, p2_stats.mean, p2_stats.min, p2_stats.max
        );
        println!(
            "Total time: median {:?}, mean {:?}, min {:?}, max {:?}",
            totals.median, totals.mean, totals.min, totals.max
        );

        Ok(totals)
    }

    fn print_test_result(&self, label: &str, result: &TestCaseResult) {
        match result {
            TestCaseResult::NotExecuted => println!("{label} test: (skipped)"),
            TestCaseResult::Passed(t) => println!("{label} test: CORRECT ({t:?})"),
            TestCaseResult::Failed(expected, actual) => {
                println!("{label} test: INCORRECT");
                println!("  Expected: {expected}");
                println!("  Received: {actual}");
            }
        }
    }

    fn max_day(&self) -> Result<u8> {
        let year: u32 = self
            .env
            .year
            .parse()
            .with_context(|| format!("Invalid year value {}", self.env.year))?;
        Ok(if year >= 2025 { 12 } else { 25 })
    }
}

fn build_stats(times: &mut [Duration]) -> RunStats {
    if times.is_empty() {
        return RunStats::default();
    }
    times.sort();
    let min = times[0];
    let max = *times.last().unwrap();
    let median = if times.len() % 2 == 1 {
        times[times.len() / 2]
    } else {
        (times[times.len() / 2 - 1] + times[times.len() / 2]) / 2
    };
    let sum: Duration = times.iter().copied().sum();
    let mean = sum / (times.len() as u32);
    RunStats {
        min,
        max,
        median,
        mean,
    }
}
