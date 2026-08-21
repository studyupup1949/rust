mod canvas;
mod instruction;
mod shape;

pub use canvas::{ClockCanvas, ClockCanvasBuilder};
pub use shape::{ClockMovement, ClockShape};


#[cfg(test)]
mod test;
