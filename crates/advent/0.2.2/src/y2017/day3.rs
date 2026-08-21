// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>

//! Spiral Memory ([Statement](https://adventofcode.com/2017/day/3))

use std::collections::HashMap;
use std::fmt;
use std::ops::Add;

/// `O(n)` with no storage. Each square on the grid is allocated in a spiral pattern starting at a
/// location marked 1 and then counting up while spiraling outward. How many steps are required to
/// carry the data from the square identified in your puzzle input all the way to the access port?
pub fn part1(input: usize) -> i32 {
    spiral()
        .take(input - 1)
        .fold(Move::default(), |a, x| a + x)
        .manhattan_distance()
}

/// In the same allocation order as shown above, they store the sum of the values in all adjacent
/// squares, including diagonals.
pub fn part2(input: usize) -> usize {
    // The set of moves to reach 8 potential neighbors
    let neighbors = vec![R, L, U, D, R + U, R + D, L + D, L + U];

    // Keep a record of all the slots we've filled so far
    let mut position = Move::default();
    let mut grid = HashMap::new();
    grid.insert(position, 1);

    for next in spiral() {
        // Make the next move
        position = position + next;

        // Compute its value based its neighbors we have values for
        let mut value = 0;
        for diff in neighbors.iter() {
            let neighbor = position + *diff;
            if let Some(&neighbor_value) = grid.get(&neighbor) {
                value += neighbor_value;
            }
        }

        // Fill this position in the grid
        grid.insert(position, value);

        // If the value
        if value > input {
            return value;
        }
    }

    unreachable!()
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
/// A move in 2 dimensions
pub struct Move {
    pub x: i32,
    pub y: i32,
}

/// Move right
pub const R: Move = Move { x: 1, y: 0 };

/// Move left
pub const L: Move = Move { x: -1, y: 0 };

/// Move up
pub const U: Move = Move { x: 0, y: 1 };

/// Move down
pub const D: Move = Move { x: 0, y: -1 };

/// The fixed sequence of moves we make in a Spiral
const SEQUENCE: [Move; 4] = [D, R, U, L];

impl Add for Move {
    type Output = Move;

    /// Add two moves to get aggregate move
    fn add(self, rhs: Move) -> Move {
        Move {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Move {
    /// [Manhattan Distance](https://en.wikipedia.org/wiki/Taxicab_geometry)
    pub fn manhattan_distance(self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

/// State for spiral iterator
#[derive(Debug, Default)]
struct Spiral {
    current: usize,
    repeat: usize,
    offset: usize,
}

/// Spiral iterator for moves in the grid
impl Iterator for Spiral {
    type Item = Move;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        // If we've repeated the required number of times
        if self.repeat == 0 {
            // Reset and advance
            self.current += 1;
            self.repeat = self.current * 2;
            self.advance_offset();
        }

        // If we've halfway through the repetitions
        if self.repeat == self.current {
            self.advance_offset();
        }

        // Track repeats
        self.repeat -= 1;

        // Return current move
        Some(SEQUENCE[self.offset])
    }
}

impl Spiral {
    /// Advance the move sequence offset
    fn advance_offset(&mut self) {
        self.offset += 1;
        self.offset %= 4;
    }
}

/// An iterator that returns the next move in the spiral
pub fn spiral() -> impl Iterator<Item = Move> {
    Spiral::default()
}

/// Pretty print moves
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.x.abs() != 0 {
            if self.x.abs() != 1 {
                write!(f, "{}", self.x.abs())?;
            }
            write!(f, "{}", if self.x > 0 { "R" } else { "L" })?;
        }

        if self.y.abs() != 0 {
            if self.y.abs() != 1 {
                write!(f, "{}", self.y.abs())?;
            }
            write!(f, "{}", if self.y > 0 { "U" } else { "D" })?;
        }

        Ok(())
    }
}

#[test]
fn examples() {
    assert_eq!(part1(1), 0);
    assert_eq!(part1(12), 3);
    assert_eq!(part1(23), 2);
    assert_eq!(part1(1024), 31);
}

#[test]
fn solution() {
    assert_eq!(part1(265149), 438);
    assert_eq!(part2(265149), 266330);
}
