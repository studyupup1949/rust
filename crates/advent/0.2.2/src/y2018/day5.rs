// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>
//! Alchemical Reduction ([Statement](https://adventofcode.com/2018/day/5)).
//! Input size parameters `n`: Length of input, `m` number of reacting units

use super::super::TRUST;

/// ASCII case difference
const CASE: u8 = b'a' - b'A';

/// Two units are inverses if they differ only in case (input is restricted to ASCII)
pub fn inverses(x: u8, y: u8) -> bool {
    CASE == match (x, y) {
        (a, b) if a > b => a - b,
        (a, b) if b > a => b - a,
        _ => 0,
    }
}

/// `O(n*m)` Reduce the given polymer to its smallest canceling inverse units
pub fn part1(input: &[u8]) -> usize {
    let mut last: Vec<u8> = Vec::from(input);
    let mut current: Vec<u8> = Vec::with_capacity(last.len());

    let mut reduced = true;

    while reduced {
        reduced = false;
        let mut index = 0;

        while index < last.len() {
            let unit = last[index];
            let next = if index + 1 < last.len() {
                last[index + 1]
            } else {
                0
            };

            if inverses(unit, next) {
                reduced = true;
                index += 2;
                continue;
            }

            current.push(unit);
            index += 1;
        }

        last.clear();
        last.extend(current.drain(..))
    }

    last.len()
}

/// `O(n*m)` Find the "problem" unit, i.e. the unit which when removed from the input gives the smallest reduction
pub fn part2(input: &str) -> usize {
    (b'a'..=b'z')
        .map(|u| (u as char, ((u - CASE) as char)))
        .map(|(l, u)| part1(input.replace(l, "").replace(u, "").as_bytes()))
        .min()
        .expect(TRUST)
}

#[test]
fn examples() {
    let input = "dabAcCaCBAcCcaDA";
    assert_eq!(10, part1(input.as_bytes()));
    assert_eq!(4, part2(input));
}

#[test]
fn solution() {
    let input = include_str!("input/5");
    assert_eq!(11310, part1(input.as_bytes()));
    assert_eq!(6020, part2(input))
}
