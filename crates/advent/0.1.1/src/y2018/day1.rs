// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>
//! Chronal Calibration ([Statement](https://adventofcode.com/2018/day/1)).

use std::collections::HashSet;

/// `O(n)` one-liner with no additional space
pub fn part1(input: &[i32]) -> i32 {
    input.iter().sum()
}

/// `O(n)` with `O(n)` space for a hash-set to track seen numbers.
pub fn part2(input: &[i32]) -> i32 {
    let mut current: i32 = 0;
    let mut seen = HashSet::new();
    seen.insert(current);

    for x in input.iter().cycle() {
        current += x;
        if !seen.insert(current) {
            return current;
        }
    }
    unreachable!()
}

#[test]
fn examples() {
    assert_eq!(part1(&[1, 1, 1]), 3);
    assert_eq!(part1(&[1, 1, -2]), 0);
    assert_eq!(part1(&[-1, -2, -3]), -6);
    assert_eq!(part2(&[1, -1]), 0);
    assert_eq!(part2(&[3, 3, 4, -2, -4]), 10);
    assert_eq!(part2(&[-6, 3, 8, 5, -6]), 5);
    assert_eq!(part2(&[7, 7, -2, -7, -4]), 14);
}

#[test]
fn solution() {
    use std::str::FromStr;
    let input = include_str!("input/1");
    let numbers: Vec<i32> = input
        .lines()
        .filter_map(|s| i32::from_str(s).ok())
        .collect();
    assert_eq!(part1(&numbers), 525);
    assert_eq!(part2(&numbers), 75749);
}
