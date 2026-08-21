//! Animated shapes and controls

mod controls;
mod frame;
mod options;
mod player;

pub use controls::{AnimationControls, PlayState};
pub use frame::{Frame, FrameReel};
pub use options::AnimationOptions;
pub use player::AnimationPlayer;
