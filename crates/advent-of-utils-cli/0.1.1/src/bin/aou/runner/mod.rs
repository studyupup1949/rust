mod executor;

use advent_of_utils_cli::{error::AocError, types::display::Table};

use crate::{config::RunConfig, loader};

pub fn run(config: &RunConfig) -> Result<(), AocError> {
    // Load solutions
    let solutions = loader::load_solutions(config)?;

    // Execute solutions
    let execution_result = executor::run_solutions(config, &solutions)?;

    // Display results with metrics
    execution_result.table();

    Ok(())
}
