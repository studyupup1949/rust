mod ffi;

use advent_of_utils::Solution;
use advent_of_utils_cli::error::AocError;
use std::collections::HashMap;

use crate::config::RunConfig;
use ffi::SolutionLibrary;

/// Represents a collection of loaded solutions for a specific year
pub struct Solutions {
    solutions: HashMap<u8, Box<dyn Solution>>,
    _library: SolutionLibrary, // Keeps the library loaded
}

impl Solutions {
    /// Get a solution for a specific day
    pub fn get(&self, day: u8) -> Option<&dyn Solution> {
        self.solutions.get(&day).map(|s| s.as_ref())
    }

    /// Get all solutions
    pub fn iter(&self) -> impl Iterator<Item = (u8, &dyn Solution)> {
        self.solutions
            .iter()
            .map(|(&day, solution)| (day, solution.as_ref()))
    }
}

/// Load solutions for a specific year
pub fn load_solutions(config: &RunConfig) -> Result<Solutions, AocError> {
    let library = SolutionLibrary::load(config)?;
    let solutions = library.get_solutions()?;

    Ok(Solutions {
        solutions,
        _library: library,
    })
}
