// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>

//! Corruption Checksum ([Statement](https://adventofcode.com/2017/day/2))

/// Calculate the spreadsheet's checksum. For each row, determine the difference between the largest
/// value and the smallest value; the checksum is the sum of all of these differences.
pub fn part1(matrix: &[Vec<u32>]) -> u32 {
    matrix.iter().map(Vec::as_slice).map(range).sum()
}

/// Find the only two numbers in each row where one evenly divides the other, divide them, and add
/// up each line's result
pub fn part2(matrix: &mut [Vec<u32>]) -> u32 {
    matrix.iter_mut().map(multiples).sum()
}

/// Range of values in the row (i.e. difference between the max and the min values) `O(n)`.
pub fn range(row: &[u32]) -> u32 {
    let max = row.iter().max().unwrap();
    let min = row.iter().min().unwrap();
    max - min
}

/// Read a 2-dimensional array of numbers delimited by newlines and whitespace
pub fn read_matrix(input: &str) -> Vec<Vec<u32>> {
    use std::str::FromStr;
    input
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.split_whitespace()
                .filter_map(|s| u32::from_str(s).ok())
                .collect()
        }).collect()
}

/// Each row has 2 numbers where the larger is an whole multiple of the smaller. Find those
/// and return the quotient
pub fn multiples(row: &mut Vec<u32>) -> u32 {
    // Sort the numbers to go from low to high
    row.sort();
    for i in 0..row.len() {
        for j in i + 1..row.len() {
            if row[j] % row[i] == 0 {
                return row[j] / row[i];
            }
        }
    }

    // We're told the input always has a pair we're looking for
    unreachable!()
}

#[test]
fn examples() {
    let one = r"
    5 1 9 5
    7 5 3
    2 4 6 8
    ";
    let two = r"
    5 9 2 8
    9 4 7 3
    3 8 6 5
    ";
    assert_eq!(part1(&read_matrix(one)), 18);
    assert_eq!(part2(&mut read_matrix(two)), 9);
}

#[test]
fn solution() {
    let mut digits = read_matrix(include_str!("input/2"));
    assert_eq!(part1(&digits), 32020);
    assert_eq!(part2(&mut digits), 236);
}
