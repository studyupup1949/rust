//! Shapes for drawing

mod clock;
mod direction;
mod euclidean;
mod traits;
mod tree;

pub (crate) use traits::canvas_sealed;

pub use clock::{ClockCanvas, ClockMovement, ClockShape};
pub use direction::{Direction, Orientation};
pub use euclidean::{EuclideanCanvas, EuclideanShape};
pub use traits::{AdicCanvas, DisplayShape};
pub use tree::{TreeCanvas, TreeShape};

/// Builder functions for canvases
pub mod builder {
    pub use super::clock::ClockCanvasBuilder;
    pub use super::euclidean::EuclideanCanvasBuilder;
    pub use super::tree::TreeCanvasBuilder;
}
