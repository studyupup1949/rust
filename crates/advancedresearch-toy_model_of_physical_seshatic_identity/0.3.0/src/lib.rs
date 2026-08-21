#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use Cell::*;

pub mod games;

/// Represents a cell in the map.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    /// An indetermine state.
    Unknown,
    /// The player.
    Player,
    /// A previous position of the player.
    ///
    /// This is intended to make visualization easier.
    Follower,
    /// An observable.
    ///
    /// This is either `true` or `false`.
    Val(bool),
}

/// Represents the map of a game.
#[derive(Clone)]
pub struct Map {
    /// The current state of cells.
    pub cells: [[Cell; 4]; 4],
    /// The "laws of nature".
    pub f: fn(&mut u16),
    /// The current player position.
    pub player_pos: [usize; 2],
}

impl Map {
    /// Creates a new map with default initial configuration.
    ///
    /// One can modify this state to e.g. create observables.
    pub fn new(f: fn(&mut u16)) -> Map {
        Map {
            cells: [
                [Player, Unknown, Unknown, Unknown],
                [Unknown; 4],
                [Unknown; 4],
                [Unknown; 4],
            ],
            f,
            player_pos: [0, 0],
        }
    }

    /// Returns `true` if the player has reached the goal, `false` otherwise.
    pub fn has_won(&self) -> bool {
        if let Player = self.cells[3][3] {true} else {false}
    }

    /// Returns `true` if the player is dead, `false` otherwise.
    pub fn has_lost(&self) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if let Player = self.cells[i][j] {return false}
            }
        }
        true
    }

    /// Gets the amount of information in the map.
    pub fn information(&self) -> u8 {
        let mut sum = 0;
        for i in 0..4 {
            for j in 0..4 {
                if let Val(_) = self.cells[i][j] {sum += 1}
            }
        }
        sum
    }

    /// Gets the amount of followers in the map.
    pub fn followers(&self) -> u8 {
        let mut sum = 0;
        for i in 0..4 {
            for j in 0..4 {
                if let Follower = self.cells[i][j] {sum += 1}
            }
        }
        sum 
    }

    /// Performs a move.
    /// Returns `true` if it was valid, `false` otherwise.
    pub fn mov(&mut self, pos: [usize; 2]) -> bool {
        if pos == [0, 0] {return false}
        let information = self.information();

        let old_pos = self.player_pos;
        self.cells[self.player_pos[1]][self.player_pos[0]] = Follower;
        self.cells[pos[1]][pos[0]] = Player;
        self.player_pos = pos;

        let mut valid = true;
        let mut filter1: u16 = u16::MAX;
        let mut filter2: u16 = 0;
        'state: for s in 0..=u16::MAX {
            for i in 0..4 {
                for j in 0..4 {
                    let b = i * 4 + j;
                    let v = (s >> b) & 1 == 1;
                    match self.cells[i][j] {
                        Player | Follower | Unknown => {}
                        Val(a) => if v != a {continue 'state}
                    }
                }
            }

            let b = old_pos[1] * 4 + old_pos[0];
            let old_player_state = (s >> b) & 1 == 1;
            let mut t = s;
            (self.f)(&mut t);
            let b2 = pos[1] * 4 + pos[0];
            let new_player_state = (t >> b2) & 1 == 1;
            if new_player_state != old_player_state {
                valid = false;
                break;
            }
            filter1 &= t;
            filter2 |= t;
        }
        if !valid {
            self.cells[self.player_pos[1]][self.player_pos[0]] = Follower;
        }

        // Update cells with new observables.
        let old_cells = self.cells.clone();
        for i in 0..4 {
            for j in 0..4 {
                let b = i * 4 + j;
                let set_to_1 = (filter1 >> b) & 1 == 1;
                let set_to_0 = (filter2 >> b) & 1 == 0;
                match (set_to_1, set_to_0) {
                    (true, true) | (false, false) => match self.cells[i][j] {
                        Player | Follower | Unknown => {}
                        Val(_) => self.cells[i][j] = Unknown,
                    },
                    (true, false) => self.cells[i][j] = Val(true),
                    (false, true) => self.cells[i][j] = Val(false),
                }
            }
        }
        if self.information() != information {return false}

        // Check that no other states outside possible worlds
        // can result in a match against this new world.
        'other_state: for s2 in 0..=u16::MAX {
            let mut match_all = true;
            for i in 0..4 {
                for j in 0..4 {
                    let b = i * 4 + j;
                    let v = (s2 >> b) & 1 == 1;
                    match old_cells[i][j] {
                        Player | Follower | Unknown => {}
                        Val(a) => if v != a {match_all = false}
                    }
                }
            }
            if match_all {continue 'other_state}

            let mut t2 = s2;
            (self.f)(&mut t2);
            let mut match_all = true;
            for i in 0..4 {
                for j in 0..4 {
                    let b = i * 4 + j;
                    let v = (t2 >> b) & 1 == 1;
                    match self.cells[i][j] {
                        Player | Follower | Unknown => {}
                        Val(a) => if v != a {match_all = false}
                    }
                }
            }
            if match_all {
                valid = false;
                break 'other_state;
            }
        }

        valid
    }
}

/// Represents a game.
pub struct Game {
    /// The name of the game.
    pub name: String,
    /// Laws of nature.
    pub f: fn(&mut u16),
    /// Configurate the map.
    pub config: fn(&mut Map),
}

/// Copy bit from one location to another.
pub fn move_bit(from: [usize; 2], to: [usize; 2], s: &mut u16) {
    let b = from[1] * 4 + from[0];
    let b2 = to[1] * 4 + to[0];
    if (*s >> b) & 1 == 1 {*s |= 1 << b2} else {*s &= !(1 << b2)}
}

/// Chain moves together.
pub fn snake_bits(ps: &[[usize; 2]], s: &mut u16) {
    for i in (1..ps.len()).rev() {
        let j = i - 1;
        move_bit(ps[j], ps[i], s);
    }
}

/// Swap bits from one location with another.
pub fn swap_bits(from: [usize; 2], to: [usize; 2], s: &mut u16) {
    let a = get_bit(from, *s);
    set_bit(from, get_bit(to, *s), s);
    set_bit(to, a, s);
}

/// Toggles bit at some location.
pub fn toggle_bit(at: [usize; 2], s: &mut u16) {
    set_bit(at, !get_bit(at, *s), s);
}

/// Set bit in `u16` state.
pub fn set_bit(a: [usize; 2], val: bool, s: &mut u16) {
    let b = a[1] * 4 + a[0];
    if val {*s |= 1 << b} else {*s &= !(1 << b)}
}

/// Get bit from `u16` state.
pub fn get_bit(a: [usize; 2], s: u16) -> bool {
    let b = a[1] * 4 + a[0];
    (s >> b) & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sets the output bit directly from the input bit.
    pub fn direct_win(s: &mut u16) {
        move_bit([0, 0], [3, 3], s);
    }

    /// Sets the bit at `(1, 0)` to `1`.
    pub fn set_1_0_to_1(s: &mut u16) {
        set_bit([1, 0], true, s);
    }

    /// Allows moving diagonally.
    pub fn diagonal(s: &mut u16) {
        move_bit([2, 2], [3, 3], s);
        move_bit([1, 1], [2, 2], s);
        move_bit([0, 0], [1, 1], s);
    }

    /// Allows moving anywhere.
    pub fn anywhere(s: &mut u16) {
        for i in 0..4 {
            for j in 0..4 {
                let pos = [i, j];
                if pos == [0, 0] {continue}
                move_bit([0, 0], pos, s);
            }
        }
    }

    /// Allow moving to either corner,
    /// but only allow reaching goal from top right corner.
    pub fn fake_choice(s: &mut u16) {
        move_bit([3, 0], [3, 3], s);
        move_bit([0, 0], [3, 0], s);
        move_bit([0, 0], [0, 3], s);
    }

    /// Allow moving to either corner,
    /// but provides no way to reach the goal from either corner.
    ///
    /// This is because there is some possible worlds where corners are not equal.
    pub fn fake_choice2(s: &mut u16) {
        if get_bit([3, 0], *s) == get_bit([0, 3], *s) {
            move_bit([3, 0], [3, 3], s);
            assert_eq!(get_bit([0, 3], *s), get_bit([3, 0], *s));
        }
        move_bit([0, 0], [3, 0], s);
        move_bit([0, 0], [0, 3], s);
    }

    /// Sets goal to zero.
    pub fn set_goal_to_zero(s: &mut u16) {
        set_bit([3, 3], false, s);
    }

    #[test]
    fn test_direct_win() {
        let mut map = Map::new(direct_win);
        assert!(!map.has_won());
        assert!(!map.has_lost());

        assert!(map.mov([3, 3]));
        assert!(map.has_won());
    }

    #[test]
    fn test_set_1_0_to_1() {
        let mut map = Map::new(set_1_0_to_1);
        assert_eq!(map.cells[0][1], Unknown);

        assert!(!map.mov([1, 1]));
        assert_eq!(map.cells[0][1], Val(true));
    }

    #[test]
    fn test_diagonal() {
        let mut map = Map::new(diagonal);
        assert!(map.mov([1, 1]));
        assert!(map.mov([2, 2]));
        assert!(map.mov([3, 3]));
        assert!(map.has_won());
    }

    #[test]
    fn test_anywhere() {
        let mut map = Map::new(anywhere);
        assert!(map.mov([2, 3]));
        // You have to try reach the goal immediately.
        assert!(!map.mov([3, 3]));
        assert!(!map.has_won());
    }

    #[test]
    fn test_fake_choice() {
        let mut map = Map::new(fake_choice);
        assert!(map.mov([3, 0]));
        assert!(map.mov([3, 3]));

        let mut map = Map::new(fake_choice);
        assert!(map.mov([0, 3]));
        assert!(!map.mov([3, 3]));
    }

    #[test]
    fn test_fake_choice2() {
        let mut map = Map::new(fake_choice2);
        assert!(map.mov([3, 0]));
        assert!(!map.mov([3, 3]));

        let mut map = Map::new(fake_choice2);
        assert!(map.mov([0, 3]));
        assert!(!map.mov([3, 3]));
    }

    #[test]
    fn test_set_goal_to_zero() {
        let mut map = Map::new(set_goal_to_zero);
        assert!(!map.mov([3, 3]));
        assert!(map.has_lost());
    }
}

