// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use VFrame;
use adi_gpu;
use Window;

/// A reference to an image in GPU memory.
pub struct Texture(pub(crate) adi_gpu::Texture, pub(crate) u16, pub(crate) u16);

impl Texture {
	#[doc(hidden)]
	pub fn new(window: &mut Window, wh: (u16,u16), image_data: &VFrame)
		-> Texture
	{
		Texture(window.window.texture(wh, image_data), wh.0, wh.1)
	}

	/// Load an empty texture into gpu memory.
	pub fn empty(window: &mut Window, wh: (u16,u16)) -> Texture {
		let size = (wh.0 as usize) * (wh.1 as usize);
		Texture(window.window.texture(wh, &VFrame(vec![0; 4 * size])),
			wh.0, wh.1)
	}

	/// Get the width and height of the texture.
	pub fn wh(&self) -> (u16, u16) {
		(self.1, self.2)
	}

	/// Set the pixels for the texture.
	pub fn set(&mut self, window: &mut Window, wh: (u16,u16), data: &VFrame)
	{
		window.window.set_texture(&mut self.0, wh, data);
	}
}
