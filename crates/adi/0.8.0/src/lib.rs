// "adi" crate - Licensed under the MIT LICENSE
//  * Copyright (c) 2017-2018  Jeron A. Lau <jeron.lau@plopgrizzly.com>

//! Aldaron's Device Interface is a library developed by Plop Grizzly for
//! creating platform-agnostic apps and video games (similar to SDL).

#![warn(missing_docs)]
#![doc(html_logo_url = "http://plopgrizzly.com/adi/icon.png",
	html_favicon_url = "http://plopgrizzly.com/adi/icon.ico",
	html_root_url = "http://plopgrizzly.com/adi/")]

// Screen
extern crate adi_screen;

pub use adi_screen::{
	// adi_screen
	Gui, Mat4, Model, ModelBuilder, Sprite, Texture, Transform,
	Window, WindowBuilder, Input, Widget, IDENTITY,
	// afi
	Audio, Graphic, GraphicBuilder, Text, GraphicDecodeErr,
	// adi_clock
	Clock, Timer, Pulse,
};

#[doc(hidden)]
pub use adi_screen::{ * };
