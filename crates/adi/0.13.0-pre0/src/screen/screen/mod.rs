// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

#[cfg(not(target_arch = "wasm32"))]
use screen::ffi::render::{new_display, Display};
#[cfg(not(target_arch = "wasm32"))]
pub use screen::ffi::render::{Gradient, Model, Shape, TexCoords, Texture};

use afi::VFrame;

use ami::Matrix;

#[cfg(target_arch = "wasm32")]
mod win {
    mod wasm32;
    pub use self::wasm32::*;
}

#[cfg(target_arch = "wasm32")]
use self::win::Display;
#[cfg(target_arch = "wasm32")]
pub use self::win::{Gradient, Model, Shape, TexCoords, Texture};

/// A Window to the Screen.
pub struct App {
    // The platform-dependant implementation.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) display: Box<Display>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) display: Display,

    /* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */

    // Delta Time
    pub(crate) dt: f32,
}

impl App {
    /// Open a new Window to the Screen.
    pub(crate) fn new() -> Self {
        App {
            #[cfg(not(target_arch = "wasm32"))]
            display: new_display().unwrap(),
            #[cfg(target_arch = "wasm32")]
            display: Display::new(),

            dt: 0.0,
        }
    }

    /// Update the clear color of the Window.
    pub fn clear(&mut self, color: (u8, u8, u8)) {
        self.display.color(color)
    }

    /// Upload a model to the GPU.
    pub fn model(&mut self, vertices: &[f32], fans: Vec<(u32, u32)>) -> Model {
        self.display.model(vertices, fans)
    }

    /// Upload a texture to the GPU.
    pub fn texture(&mut self, wh: (u16, u16), graphic: &VFrame) -> Texture {
        self.display.texture(wh, graphic)
    }

    /// Create gradient object.
    pub fn gradient(&mut self, colors: &[f32]) -> Gradient {
        self.display.gradient(colors)
    }

    /// Create texture coordinate object.
    pub fn texcoords(&mut self, texcoords: &[(f32, f32)]) -> TexCoords {
        self.display.texcoords(texcoords)
    }

    /// Set the pixels of a texture to something other than the original.
    pub fn set_texture(&mut self, texture: &mut Texture, wh: (u16, u16), graphic: &VFrame) {
        self.display.set_texture(texture, wh, graphic)
    }

    /// Stop drawing a shape.
    pub fn drop_shape(&mut self, shape: &Shape) {
        self.display.drop_shape(shape)
    }

    /// Apply a matrix transform to a shape.
    pub fn transform(&self, shape: &Shape, matrix: Matrix) {
        self.display.transform(shape, matrix)
    }

    /// Get the width and height of the window.
    pub fn wh(&self) -> (u16, u16) {
        self.display.wh()
    }

    /// Get the pitch of the window's surface.
    pub fn pitch(&self) -> usize {
        self.display.pitch()
    }

    /// Update window's surface.
    pub fn draw(&mut self, writer: &mut FnMut(*mut u8) -> ()) {
        self.display.draw(writer)
    }
}
