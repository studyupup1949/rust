use crate::shape::{Direction, Orientation};


#[derive(Debug, Clone, Copy, PartialEq)]
/// Interactive options, set separately from the controls
pub struct InteractiveShapeOptions {
    /// Display numbers on the clock
    pub display_clock_numbers: bool,
    /// Direction of the tree
    pub tree_direction: Direction,
    /// Scale factor for the Euclidean
    pub euclidean_scale: f64,
    /// Direction of the euclidean
    pub euclidean_direction: Direction,
    /// Orientation of the euclidean
    pub euclidean_orientation: Orientation,
    /// Display enclosing disks for euclidean
    pub euclidean_enclosing_disks: isize,
}


impl Default for InteractiveShapeOptions {
    fn default() -> Self {
        InteractiveShapeOptions {
            display_clock_numbers: true,
            tree_direction: Direction::Up,
            euclidean_scale: 3.0,
            euclidean_direction: Direction::Up,
            euclidean_orientation: Orientation::CW,
            euclidean_enclosing_disks: 0,
        }
    }
}
