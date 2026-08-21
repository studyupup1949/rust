use advent_of_utils_cli::{
    error::{AocError, SolutionError},
    input::get_input,
    types::AocTime,
    types::{AocResult, AocYear},
};
use std::collections::HashSet;

use crate::{config::RunConfig, loader};

pub(crate) fn run_solutions(
    config: &RunConfig,
    solutions: &loader::Solutions,
) -> Result<AocYear, AocError> {
    let mut tasks: HashSet<AocResult> = HashSet::new();
    let year = config.year;

    match config.day {
        Some(day) => {
            let solver = solutions
                .get(day)
                .ok_or(AocError::Solution(SolutionError::NotImplemented))?;

            schedule_day_tasks(&mut tasks, solver, day, config)?;
        }
        None => {
            let time = AocTime::now();
            for (day, solver) in solutions
                .iter()
                .filter(|(day, _)| time.is_puzzle_available(year, *day))
            {
                schedule_day_tasks(&mut tasks, solver, day, config)?;
            }
        }
    }

    collect_results(tasks)
}

fn schedule_day_tasks(
    tasks: &mut HashSet<AocResult>,
    solver: &dyn advent_of_utils::Solution,
    day: u8,
    config: &RunConfig,
) -> Result<(), AocError> {
    let part = match config.part {
        Some(part) => part as u8,
        None => 0,
    };
    if part != 2 {
        schedule_part_task(tasks, 1, day, solver, config)?;
    }
    if part != 1 {
        schedule_part_task(tasks, 2, day, solver, config)?;
    }
    Ok(())
}

fn schedule_part_task(
    tasks: &mut HashSet<AocResult>,
    part: u8,
    day: u8,
    solver: &dyn advent_of_utils::Solution,
    config: &RunConfig,
) -> Result<(), AocError> {
    let (input, time) = get_input(config.year, day, &config.database, config.test)?;

    let result = if part == 1 {
        solver.part1(input)
    } else {
        solver.part2(input)
    };
    let time = time.elapsed();

    tasks.insert(AocResult::new(day, part, result, time));
    Ok(())
}

fn collect_results(tasks: HashSet<AocResult>) -> Result<AocYear, AocError> {
    let mut results: Vec<AocResult> = tasks.into_iter().collect();

    results.sort_by_key(|r| (r.day(), r.part() as u8));

    Ok(AocYear::from_vec(results))
}
