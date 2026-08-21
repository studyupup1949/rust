// "adi" - Aldaron's Device Interface
//
// Copyright Jeron A. Lau 2017 - 2018.
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)
//
//! Create platform-agnostic apps and video games (similar to SDL).

#![warn(missing_docs)]
#![doc(html_logo_url = "http://plopgrizzly.com/adi/icon.png",
	html_favicon_url = "http://plopgrizzly.com/adi/icon.png",
	html_root_url = "http://plopgrizzly.com/adi/")]

// Screen
extern crate adi_screen;

pub use adi_screen::{
	// adi_screen
	Gui, Mat4, Model, ModelBuilder, Sprite, Texture, Transform,
	Window, WindowBuilder, Input, Widget,
	// afi
	Audio, Graphic, GraphicBuilder, Text, GraphicDecodeErr,
	// adi_clock
	Clock, Timer, Pulse,
};

#[doc(hidden)]
pub use adi_screen::{ * };
