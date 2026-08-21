// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>

//! High-Entropy Passphrases ([Statement](https://adventofcode.com/2017/day/4))

use std::collections::HashSet;

/// `O(n)` with `O(m)` space where `m` is the number of distinct words in a line.
pub fn part1(input: &str) -> usize {
    input.lines().filter(no_repeat).count()
}

/// `O(n)` with `O(m)` space where `m` is the number of distinct words in a line.
pub fn part2(input: &str) -> usize {
    input.lines().filter(no_anagram).count()
}

/// Check if the line has any repeated words
pub fn no_repeat(line: &&str) -> bool {
    let mut seen = HashSet::new();
    line.split_whitespace().all(|w| seen.insert(w))
}

/// Check if the line has words that are anagrams of each other
pub fn no_anagram(line: &&str) -> bool {
    let mut seen = HashSet::new();
    line.split_whitespace()
        .map(|m| {
            let mut vec: Vec<char> = m.chars().collect();
            vec.sort();
            vec
        }).all(|w| seen.insert(w))
}

#[test]
fn examples() {
    assert!(no_repeat(&"aa bb cc dd ee"));
    assert!(!no_repeat(&"aa bb cc dd aa"));
    assert!(no_repeat(&"aa bb cc dd aaa"));
    assert!(no_anagram(&"abcde fghij"));
    assert!(!no_anagram(&"abcde xyz ecdab"));
    assert!(no_anagram(&"a ab abc abd abf abj"));
    assert!(no_anagram(&"iiii oiii ooii oooi oooo"));
    assert!(!no_anagram(&"oiii ioii iioi iiio"));
}

#[test]
fn solution() {
    assert_eq!(part1(include_str!("input/4")), 451);
    assert_eq!(part2(include_str!("input/4")), 223)
}
