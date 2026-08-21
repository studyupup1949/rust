// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>
//! Inventory Management System ([Statement](https://adventofcode.com/2018/day/2)).

/// Find the checksum of the input defined as the product of the number of lines in the input that
/// have 2 characters repeated with the number of lines with three characters repeated.
/// `O(n*m)` for `n` lines with `m` chars each. `O(1)` extra space for a map to count characters.
pub fn part1(lines: &[&str]) -> u32 {
    let mut twice = 0;
    let mut thrice = 0;

    for line in lines {
        let (two, three) = helpers::repeated(*line);
        if two {
            twice += 1;
        }
        if three {
            thrice += 1;
        }
    }

    (twice * thrice)
}

/// Compute the "box identifier" from the input by finding 2 lines that differ in exactly 1 index
/// and returning all matching characters in sequence. `O(n^2*m)` to consider each pair of lines in
/// the input and compare `m` characters to find differences. No additional space.
pub fn part2(lines: &[&str]) -> String {
    for a in lines {
        for b in lines {
            if let Some(id) = helpers::extract(a, b) {
                return id;
            }
        }
    }
    unreachable!()
}

///! Helper routines
pub mod helpers {
    use std::collections::HashMap;

    /// Take two strings and if they differ at exactly one index, return the equal chars in sequence.
    pub fn extract(a: &str, b: &str) -> Option<String> {
        debug_assert!(a.len() == b.len());

        match a.chars().zip(b.chars()).filter(|x| x.0 != x.1).count() {
            1 => Some(
                a.chars()
                    .zip(b.chars())
                    .filter(|x| x.0 == x.1)
                    .map(|x| x.0)
                    .collect(),
            ),
            _ => None,
        }
    }

    /// Test if the line has characters that repeat the 2 or 3 times
    pub fn repeated(line: &str) -> (bool, bool) {
        let mut counts = HashMap::new();
        line.chars()
            .for_each(|c| *counts.entry(c).or_insert(0) += 1);

        let twice = counts.iter().any(|e| *e.1 == 2);
        let thrice = counts.iter().any(|e| *e.1 == 3);

        (twice, thrice)
    }
}

#[test]
fn examples() {
    let one: Vec<&str> = r"
    abcdef
    bababc
    abbcde
    abcccd
    aabcdd
    abcdee
    ababab
    ".lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let two: Vec<&str> = r"
    abcde
    fghij
    klmno
    pqrst
    fguij
    axcye
    wvxyz
    ".lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(part1(&one), 12);
    assert_eq!(part2(&two), String::from("fgij"));
}

#[test]
fn solution() {
    let input: Vec<&str> = include_str!("input/2")
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(part1(&input), 7192);
    assert_eq!(part2(&input), "mbruvapghxlzycbhmfqjonsie");
}
