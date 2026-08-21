// Aldaron's Device Interface
// Copyright (c) 2017-2018 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/lib.rs

//! Aldaron's Device Interface is a library developed by Plop Grizzly for
//! creating platform-agnostic apps and video games (similar to SDL).

// TODO: all dependencies support no_std
// #![no_std]
#![warn(missing_docs)]
#![doc(html_logo_url = "http://plopgrizzly.com/adi/icon.png",
	html_favicon_url = "http://plopgrizzly.com/adi/icon.ico",
	html_root_url = "http://plopgrizzly.com/adi/")]

// Screen
extern crate adi_screen;

/// Interface with a monitor or computer/tablet/phone screen to render graphics
pub mod screen {
	pub use adi_screen::*;
}

/// Interface with the CPU's Timing Mechanisms to time operations, get the time
/// (TODO), sleep with precision, and animate smoothly
pub mod clock {
	pub use adi_screen::adi_clock::*;
}

// Re-Definition of adi_screen's macros
/// Macro to load multiple textures into an array.
#[macro_export] macro_rules! textures {
	( $window:expr, $decode:expr, $( $x:expr ),*) => {
		&[ $( $crate::screen::Texture::new($window,
			$decode(include_bytes!($x)).unwrap()) ),* ]
	}
}