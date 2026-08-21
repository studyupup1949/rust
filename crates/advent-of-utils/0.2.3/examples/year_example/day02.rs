use advent_of_utils::{AocOption, Solution};

#[derive(Clone)]
pub struct Day02;

// You can also just implement one part and not the other
impl Solution for Day02 {
    fn part1(&self, input: String) -> AocOption {
        let first_line = input.lines().next();
        first_line.into()
    }
}
