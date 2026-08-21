use crate::AocOption;

/// Defines the interface for implementing Advent of Code daily puzzle solutions.
/// This trait must be implemented for each day's solution, providing methods
/// to solve both parts of the puzzle.
///
/// # Example
/// ```rust
/// use advent_of_utils::{Solution, AocOption};
///
/// struct Day01 {}
///
/// impl Solution for Day01 {
///     fn part1(&self, input: String) -> AocOption {
///         // Implement solution for part 1
///         42.into()
///     }
///
///     fn part2(&self, input: String) -> AocOption {
///         // Implement solution for part 2
///         "solution".into()
///     }
/// }
/// ```
pub trait Solution {
    /// Solves Part 1 of the daily puzzle.
    ///
    /// # Arguments
    /// * `input` - The puzzle input as a string, automatically fetched and cached
    ///             from Advent of Code.
    ///
    /// # Returns
    /// * `AocOption` - The solution result, which can be either:
    ///   - `AocOption::Int` for numeric answers
    ///   - `AocOption::Str` for string answers
    ///   - `AocOption::None` if not implemented (default)
    #[allow(unused)]
    fn part1(&self, input: String) -> AocOption {
        AocOption::None
    }

    /// Solves Part 2 of the daily puzzle.
    ///
    /// # Arguments
    /// * `input` - The puzzle input as a string, automatically fetched and cached
    ///             from Advent of Code.
    ///
    /// # Returns
    /// * `AocOption` - The solution result, which can be either:
    ///   - `AocOption::Int` for numeric answers
    ///   - `AocOption::Str` for string answers
    ///   - `AocOption::None` if not implemented (default)
    #[allow(unused)]
    fn part2(&self, input: String) -> AocOption {
        AocOption::None
    }
}
