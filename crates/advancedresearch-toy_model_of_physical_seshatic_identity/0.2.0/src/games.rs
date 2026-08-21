//! Some example games.

use crate::*;

/// Move across the diagonal.
pub fn diagonal() -> Game {
    Game {
        name: "Diagonal".to_string(),
        f: |s: &mut u16| {
            move_bit([2, 2], [3, 3], s);
            move_bit([1, 1], [2, 2], s);
            move_bit([0, 0], [1, 1], s);
        },
        config: |_: &mut Map| {},
    }
}

/// A snaky path from start to the goal.
pub fn snake() -> Game {
    Game {
        name: "Snake".to_string(),
        f: |s: &mut u16| {
            snake_bits(&[
                [0, 0], [1, 0], [2, 0], [3, 0],
                [3, 1], [2, 1], [1, 1], [0, 1],
                [0, 2], [1, 2], [2, 2], [3, 2],
                [3, 3],
            ], s);
        },
        config: |_: &mut Map| {},
    }
}

/// Moving while a clock is ticking.
pub fn clock() -> Game {
    Game {
        name: "Clock".to_string(),
        f: |s: &mut u16| {
            move_bit([3, 2], [3, 3], s);
            move_bit([2, 2], [3, 2], s);
            move_bit([1, 2], [2, 2], s);
            move_bit([0, 2], [1, 2], s);
            move_bit([0, 1], [0, 2], s);
            move_bit([1, 1], [0, 1], s);
            move_bit([2, 1], [1, 1], s);
            move_bit([3, 1], [2, 1], s);
            move_bit([3, 0], [3, 1], s);
            move_bit([2, 0], [3, 0], s);
            move_bit([1, 0], [2, 0], s);
            move_bit([0, 0], [1, 0], s);
            if get_bit([0, 3], *s) {
                if get_bit([1, 3], *s) {
                    toggle_bit([2, 3], s);
                }
                toggle_bit([1, 3], s);
            }
            toggle_bit([0, 3], s);
        },
        config: |map: &mut Map| {
            map.cells[3][0] = Cell::Val(false);
            map.cells[3][1] = Cell::Val(false);
            map.cells[3][2] = Cell::Val(false);
        }
    }
}

