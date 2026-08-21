// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use Window;
use adi_gpu::*;

#[must_use]
/// Sprite represents anything that is rendered onto the screen.
pub struct Sprite(pub(crate) Shape);

impl Sprite {
	#[doc(hidden)]
	pub fn new(window: &mut Window, model: usize,
		texture: Option<usize>, transform: Transform, alpha: bool,
		fog: bool, camera: bool) -> Self
	{
		if let Some(gradient) = window.models[model].1 {
			if let Some(texcoords) = window.models[model].2 {
				// Complex
				Sprite(window.window.shape_complex(
					&window.models[model].0, transform,
					&window.textures[texture.unwrap()].0,
					texcoords, gradient, alpha, fog, camera)
				)
			} else {
				// Gradient
				Sprite(window.window.shape_gradient(
					&window.models[model].0, transform,
					gradient, alpha, fog, camera)
				)
			}
		} else if let Some(texcoords) = window.models[model].2 {
			if let Some(color) = window.models[model].3 {
				// Tinted
				Sprite(window.window.shape_tinted(
					&window.models[model].0, transform,
					&window.textures[texture.unwrap()].0,
					texcoords, color, alpha, fog, camera))
			} else if let Some(opacity) = window.models[model].4 {
				// Faded
				Sprite(window.window.shape_faded(
					&window.models[model].0, transform,
					&window.textures[texture.unwrap()].0,
					texcoords, opacity, fog, camera))
			} else {
				// Texture
				Sprite(window.window.shape_texture(
					&window.models[model].0, transform,
					&window.textures[texture.unwrap()].0,
					texcoords, alpha, fog, camera))
			}
		} else if let Some(color) = window.models[model].3 {
			// Solid
			Sprite(window.window.shape_solid(&window.models[model].0,
				transform, color, alpha, fog, camera))
		} else {
			panic!("Not enough information to make Sprite!")
		}
	}

	/// Apply a Transform onto Sprite.
	#[inline(always)]
	pub fn transform(&self, window: &mut Window, transform: Transform) {
		window.window.transform(&self.0, transform);
	}
}
