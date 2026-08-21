//! Shared color constants for aaai GUI.

use iced::Color;

pub const OK_COLOR: Color     = Color { r: 0.18, g: 0.65, b: 0.32, a: 1.0 };
pub const PENDING_COLOR: Color = Color { r: 0.88, g: 0.60, b: 0.12, a: 1.0 };
pub const FAILED_COLOR: Color  = Color { r: 0.82, g: 0.18, b: 0.18, a: 1.0 };
pub const ERROR_COLOR: Color   = Color { r: 0.70, g: 0.18, b: 0.70, a: 1.0 };
pub const IGNORED_COLOR: Color = Color { r: 0.55, g: 0.55, b: 0.55, a: 1.0 };

#[allow(dead_code)]
pub const ADDED_COLOR: Color   = Color { r: 0.18, g: 0.65, b: 0.32, a: 1.0 };
#[allow(dead_code)]
pub const REMOVED_COLOR: Color = Color { r: 0.82, g: 0.18, b: 0.18, a: 1.0 };
#[allow(dead_code)]
pub const MODIFIED_COLOR: Color = Color { r: 0.88, g: 0.60, b: 0.12, a: 1.0 };
