/**
 * adi - Aldaron's Device Interface - "lib.rs"
 * Copyright 2017 (c) Jeron Lau - Licensed under the MIT LICENSE
**/

pub const VERSION : &'static str = "adi 0.3.0";

// Screen
pub extern crate adi_screen;
pub use adi_screen as screen;

// Clock
pub use adi_screen::adi_clock;
pub use adi_screen as clock;
