// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>
//! No Matter How You Slice It ([Statement](https://adventofcode.com/2018/day/3)).
//! Input size parameters `n`: Number of claims and `m`: dimension of the fabric grid.

use std::cmp::max;
use std::str::FromStr;

/// A single claim
pub struct Claim {
    /// The identifier
    id: usize,
    /// The x coordinate of the origin
    x: usize,
    /// The y coordinate of the origin
    y: usize,
    /// The length of the claim
    l: usize,
    /// The height of the claim
    h: usize,
}

/// The entire fabric
pub struct Fabric {
    /// The claims
    claims: Vec<Claim>,
    /// The overlap tracking grid
    overlaps: Vec<Vec<u8>>,
}

/// Read a single claim from the form
pub fn read_claim(input: &str) -> Claim {
    // Expecting  `#1 @ 1,3: 4x4`
    let num: Vec<usize> = input
        .split(|c| "#@,:x ".contains(c))
        .filter_map(|c| usize::from_str(c).ok())
        .collect();

    // Must have 5 numbers
    debug_assert!(num.len() == 5);

    // Unpack
    Claim {
        id: num[0],
        x: num[1],
        y: num[2],
        l: num[3],
        h: num[4],
    }
}

/// Read all claims from the input lines. `O(n)` with `O(m^2)` additional space.
pub fn read(input: &str) -> Fabric {
    let mut dimensions = (0, 0);

    // Read claims and update the dimensions
    let claims: Vec<Claim> = input
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(read_claim)
        .inspect(|c| {
            dimensions.0 = max(c.x + c.l, dimensions.0);
            dimensions.1 = max(c.y + c.h, dimensions.1);
        }).collect();

    // Create an overlap tracking grid
    let mut overlaps = vec![vec![0; dimensions.1]; dimensions.0];

    // Track the overlapping claims
    for claim in claims.iter() {
        for x in claim.x..claim.x + claim.l {
            for y in claim.y..claim.y + claim.h {
                overlaps[x][y] += 1;
            }
        }
    }

    // Done
    Fabric { claims, overlaps }
}

/// Count overlapping squared - `O(m^2)`
pub fn part1(fabric: &Fabric) -> usize {
    fabric
        .overlaps
        .iter()
        .flat_map(|v| v.iter())
        .filter(|c| **c > 1)
        .count()
}

/// Find the only claim that does not overlap. `O(n*m^2)`
pub fn part2(fabric: &Fabric) -> usize {
    for claim in fabric.claims.iter() {
        let mut overlap = false;
        for x in claim.x..claim.x + claim.l {
            for y in claim.y..claim.y + claim.h {
                if fabric.overlaps[x][y] > 1 {
                    overlap = true
                }
            }
        }

        if !overlap {
            return claim.id;
        }
    }

    0
}

#[test]
fn examples() {
    let fabric = read(
        r"
#1 @ 1,3: 4x4
#2 @ 3,1: 4x4
#3 @ 5,5: 2x2
    ",
    );
    assert_eq!(4, part1(&fabric));
    assert_eq!(3, part2(&fabric))
}

#[test]
fn solution() {
    let fabric = read(include_str!("input/3"));
    assert_eq!(116491, part1(&fabric));
    assert_eq!(707, part2(&fabric));
}
