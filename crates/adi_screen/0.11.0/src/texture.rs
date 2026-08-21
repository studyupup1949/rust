// "adi_screen" - Aldaron's Device Interface / Screen
//
// Copyright Jeron A. Lau 2017 - 2018.
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use Graphic;
use GraphicBuilder;
use GraphicDecodeErr;
use adi_gpu;
use Window;

/// A reference to an image in GPU memory.
pub struct Texture(pub(crate) adi_gpu::Texture, pub(crate) u32, pub(crate) u32);

impl Texture {
	#[doc(hidden)]
	pub fn new(window: &mut Window, image_data: Graphic) -> Texture {
		let (w, h, _) = image_data.as_slice();

		Texture(window.window.texture(&image_data), w, h)
	}

	/// Load an empty texture into gpu memory.
	pub fn empty(window: &mut Window, w: u32, h: u32) -> Texture {
		let size = (w as usize) * (h as usize);
		let graphic = GraphicBuilder::new().rgba(w, h, vec![0; size]);

		Texture(window.window.texture(&graphic), w, h)
	}

	/// Load multiple texture from graphic data into gpu memory.
	pub fn new_vec<F>(window: &mut Window,
		loader: F, files: &[&[u8]])
		-> Result<Vec<Texture>, GraphicDecodeErr>
		where F: Fn(&[u8]) -> Result<Graphic, GraphicDecodeErr>
	{
		let mut textures = Vec::new();

		for i in files {
			textures.push(Texture::new(window, loader(i)?));
		}

		Ok(textures)
	}

	/// Get the width and height of the texture.
	pub fn wh(&self) -> (u32, u32) {
		(self.1, self.2)
	}

	/// Set the pixels for the texture.
	pub fn set(&mut self, window: &mut Window, data: &[u32]) -> () {
		window.window.set_texture(&mut self.0, data);
	}
}
