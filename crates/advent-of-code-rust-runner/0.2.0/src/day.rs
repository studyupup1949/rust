use anyhow::Result;
use std::time::{Duration, Instant};

pub struct ExecutionResult<O: std::fmt::Display + Eq> {
    pub(crate) part_1_result: O,
    pub(crate) part_1_time: Duration,
    pub(crate) part_2_result: O,
    pub(crate) part_2_time: Duration
}

pub struct DayResult {
    pub(crate) part_1_result: String,
    pub(crate) part_1_time: Duration,
    pub(crate) part_2_result: String,
    pub(crate) part_2_time: Duration
}

impl<O: std::fmt::Display + Eq> From<ExecutionResult<O>> for DayResult {
    fn from(result: ExecutionResult<O>) -> Self {
        DayResult {
            part_1_result: result.part_1_result.to_string(),
            part_1_time: result.part_1_time,
            part_2_result: result.part_2_result.to_string(),
            part_2_time: result.part_2_time,
        }
    }
}

pub(crate) enum TestCaseResult {
    NotExecuted,
    Passed(Duration),
    Failed(String, String)
}

pub struct TestResult {
    pub(crate) part1: TestCaseResult,
    pub(crate) part2: TestCaseResult
}

impl TestResult {
    fn not_executed() -> Self {
        TestResult {
            part1: TestCaseResult::NotExecuted,
            part2: TestCaseResult::NotExecuted
        }
    }

    fn from_execution_result<O: std::fmt::Display + Eq>(result: ExecutionResult<O>, expected_part_1: Option<O>, expected_part_2: Option<O>) -> Self {
        let part1 = if let Some(expected) = expected_part_1 {
            if result.part_1_result == expected {
                TestCaseResult::Passed(result.part_1_time)
            } else {
                TestCaseResult::Failed(expected.to_string(), result.part_1_result.to_string())
            }
        } else {
            TestCaseResult::NotExecuted
        };
        let part2 = if let Some(expected) = expected_part_2 {
            if result.part_2_result == expected {
                TestCaseResult::Passed(result.part_2_time)
            } else {
                TestCaseResult::Failed(expected.to_string(), result.part_2_result.to_string())
            }
        } else {
            TestCaseResult::NotExecuted
        };
        TestResult {
            part1,
            part2,
        }
    }
}

pub trait DayImplementation {
    type Output<'a>: std::fmt::Display + Eq;
    type Context<'a>;

    fn day(&self) -> u8;
    fn example_input(&self) -> Option<&'static str> { None }
    fn example_part_1_result(&self) -> Option<Self::Output<'static>> { None }
    fn example_part_2_result(&self) -> Option<Self::Output<'static>> { None }

    fn execute_part_1<'a>(&self, input: &'a str) -> Result<(Self::Output<'a>, Self::Context<'a>)>;
    fn execute_part_2<'a>(&self, input: &'a str, context: Self::Context<'a>) -> Result<Self::Output<'a>>;

    fn run_with_input<'a>(&self, input: &'a str) -> Result<ExecutionResult<Self::Output<'a>>> {
        log::debug!("Starting part 1 for day {}", self.day());
        let start_part_1 = Instant::now();
        let (part_1_result, context) = self.execute_part_1(input)?;
        let part_1_time = start_part_1.elapsed();
        log::debug!("Part 1 completed in {:?}, result: {}", part_1_time, part_1_result);

        log::debug!("Starting part 2 for day {}", self.day());
        let start_part_2 = Instant::now();
        let part_2_result = self.execute_part_2(input, context)?;
        let part_2_time = start_part_2.elapsed();
        log::debug!("Part 2 completed in {:?}, result: {}", part_2_time, part_2_result);

        Ok(ExecutionResult {
            part_1_result,
            part_1_time,
            part_2_result,
            part_2_time
        })
    }
}

pub trait Day {
    fn day(&self) -> u8;

    fn test_day(&self) -> Result<TestResult>;
    fn execute_day(&self, input: &str) -> Result<DayResult>;
}

impl<T: DayImplementation> Day for T {
    fn day(&self) -> u8 { DayImplementation::day(self) }

    fn test_day(&self) -> Result<TestResult> {
        log::debug!("Running tests for day {}", self.day());
        if let Some(example_input) = DayImplementation::example_input(self) {
            log::debug!("Example input found for day {}", self.day());
            let result = self.run_with_input(example_input)?;
            Ok(TestResult::from_execution_result(
                result,
                DayImplementation::example_part_1_result(self),
                DayImplementation::example_part_2_result(self)
            ))
        } else {
            log::debug!("No example input for day {}, skipping tests", self.day());
            Ok(TestResult::not_executed())
        }
    }

    fn execute_day(&self, input: &str) -> Result<DayResult> {
        log::debug!("Executing day {} with actual input", self.day());
        Ok(DayResult::from(self.run_with_input(input)?))
    }
}