// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

//! Render graphics to a computer or phone screen, and get input.  Great for
//! both video games and apps!

#![warn(missing_docs)]
#![doc(html_logo_url = "http://plopgrizzly.com/adi_screen/icon.png",
	html_favicon_url = "http://plopgrizzly.com/adi_screen/icon.ico",
	html_root_url = "http://plopgrizzly.com/adi_screen/")]

mod window;
mod sprite;
mod gui;
mod texture;
mod gpu_data;
#[doc(hidden)]
pub mod prelude;

pub use prelude::{ Transform, Vec3 };
pub use window::{ WindowBuilder, Window, Widget };
pub use sprite::{ Sprite };
pub use texture::Texture;
pub use gpu_data::{ Model, ModelBuilder };

extern crate adi_gpu;
extern crate aci_png;
extern crate fonterator;
#[allow(unused)]
mod footile; // TODO: extern crate
extern crate adi_clock;

pub use adi_gpu::{ Input, Mat4 };
pub use adi_gpu::afi::*;
pub use adi_clock::*;
