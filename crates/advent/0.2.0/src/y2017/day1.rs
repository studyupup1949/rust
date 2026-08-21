// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>

//! Inverse Captcha ([Statement](https://adventofcode.com/2017/day/1)).

/// Review a sequence of digits (your puzzle input) and find the sum of all digits that match the
/// next digit in the list. The list is circular, so the digit after the last digit is the first
/// digit in the list. `O(n)` with no additional space.
pub fn part1(input: &[u8]) -> u32 {
    helpers::skip_take_sum(input, 1)
}

/// Same as Part 1, but instead of considering the next digit, it wants you to consider the digit
/// halfway around the circular list. `O(n)` with no additional space.
pub fn part2(input: &[u8]) -> u32 {
    debug_assert!(input.len() % 2 == 0);
    helpers::skip_take_sum(input, input.len() / 2)
}

/// Helper routines
pub mod helpers {
    /// Read digits from the input.
    pub fn read_digits(input: &str) -> Vec<u8> {
        input
            .chars()
            .filter(|c| char::is_digit(*c, 10))
            .map(|c| char::to_digit(c, 10).unwrap() as u8)
            .collect()
    }

    /// Use iterator `skip` and `take` to setup a pair of shifted iterators over the input to zip
    /// over and then filter the required digits to sum up. `O(n)` with no additional space
    pub fn skip_take_sum(digits: &[u8], skip: usize) -> u32 {
        // Wrap the iterators by the appropriate skip and take
        let shifted = digits.iter().skip(skip).chain(digits.iter().take(skip));

        // Zip through the iterators
        digits
            .iter()
            .zip(shifted)
            // Pick those that are equal across the shift
            .filter(|pair| pair.1 == pair.0)
            // Extract the matching value
            .map(|pair| u32::from(*pair.0))
            // Add it up and we're done
            .sum()
    }
}

#[test]
fn examples() {
    assert_eq!(part1(&helpers::read_digits("1122")), 3);
    assert_eq!(part1(&helpers::read_digits("1111")), 4);
    assert_eq!(part1(&helpers::read_digits("1234")), 0);
    assert_eq!(part1(&helpers::read_digits("91212129")), 9);

    assert_eq!(part2(&helpers::read_digits("1212")), 6);
    assert_eq!(part2(&helpers::read_digits("1221")), 0);
    assert_eq!(part2(&helpers::read_digits("123425")), 4);
    assert_eq!(part2(&helpers::read_digits("123123")), 12);
    assert_eq!(part2(&helpers::read_digits("12131415")), 4);
}

#[test]
fn solution() {
    let digits = helpers::read_digits(include_str!("input/1"));
    assert_eq!(part1(&digits), 1216);
    assert_eq!(part2(&digits), 1072);
}
