// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

//! Vulkan implementation for adi_gpu.

extern crate libc;

use std::os::raw::c_void;

mod asi;
/// Transform represents a transformation matrix.
pub(crate) mod renderer;

pub use self::base::Gradient;
pub use self::base::Model;
pub use self::base::Shape;
pub use self::base::TexCoords;
pub use self::base::Texture;

use super::base;
use super::base::*;

use crate::screen::ffi::Matrix;

/// To render anything with adi_gpu, you have to make a `Display`
pub struct Display {
    window: crate::screen::ffi::Window,
    renderer: renderer::Renderer,
}

pub(crate) use self::asi::{Buffer, BufferBuilderType, VULKAN};

pub(crate) fn new() -> Result<Box<Display>, String> {
    let (renderer, window) = renderer::Renderer::new(vector!())?;

    Ok(Box::new(Display { window, renderer }))
}

impl base::Display for Display {
    fn color(&mut self, color: (u8, u8, u8)) {
        self.renderer.bg_color(vector!(
            color.0 as f32 / 255.0,
            color.1 as f32 / 255.0,
            color.2 as f32 / 255.0
        ));
    }

    fn update(&mut self) -> f32 {
        let dt = self.renderer.update(&mut self.window);

        dt
    }

    fn model(&mut self, vertices: &[f32], fans: Vec<(u32, u32)>) -> Model {
        Model(self.renderer.model(vertices, fans))
    }

    fn texture(&mut self, wh: (u16, u16), graphic: &VFrame) -> Texture {
        let (w, h) = wh;
        let pixels = graphic.0.as_slice();

        Texture(self.renderer.texture(w, h, pixels), wh.0, wh.1)
    }

    fn gradient(&mut self, colors: &[f32]) -> Gradient {
        Gradient(self.renderer.colors(colors))
    }

    fn texcoords(&mut self, texcoords: &[(f32, f32)]) -> TexCoords {
        TexCoords(self.renderer.texcoords(texcoords))
    }

    fn set_texture(&mut self, texture: &mut Texture, wh: (u16, u16), graphic: &VFrame) {
        if texture.1 == wh.0 && texture.2 == wh.1 {
            self.renderer.set_texture(texture.0, graphic.0.as_slice());
        } else {
            // resize
            self.renderer
                .resize_texture(texture.0, wh.0, wh.1, graphic.0.as_slice());
        }
    }

    #[inline(always)]
    fn shape_solid(
        &mut self,
        model: &Model,
        transform: Matrix,
        color: [f32; 4],
        blending: bool,
        camera: *const c_void, // native buffer
    ) -> Shape {
        base::new_shape(
            self.renderer
                .solid(model.0, transform, color, blending, unsafe {
                    (camera as *const Buffer).as_ref().unwrap()
                }),
        )
    }

    #[inline(always)]
    fn shape_gradient(
        &mut self,
        model: &Model,
        transform: Matrix,
        colors: Gradient,
        blending: bool,
        camera: *const c_void, // native buffer
    ) -> Shape {
        base::new_shape(
            self.renderer
                .gradient(model.0, transform, colors.0, blending, unsafe {
                    (camera as *const Buffer).as_ref().unwrap()
                }),
        )
    }

    #[inline(always)]
    fn shape_texture(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        blending: bool,
        camera: *const c_void, // native buffer
    ) -> Shape {
        base::new_shape(self.renderer.textured(
            model.0,
            transform,
            texture.0,
            tc.0,
            blending,
            unsafe { (camera as *const Buffer).as_ref().unwrap() },
        ))
    }

    #[inline(always)]
    fn shape_faded(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        alpha: f32,
        camera: *const c_void, // native buffer
    ) -> Shape {
        base::new_shape(
            self.renderer
                .faded(model.0, transform, texture.0, tc.0, alpha, unsafe {
                    (camera as *const Buffer).as_ref().unwrap()
                }),
        )
    }

    #[inline(always)]
    fn shape_tinted(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        tint: [f32; 4],
        blending: bool,
        camera: *const c_void, // native buffer
    ) -> Shape {
        base::new_shape(self.renderer.tinted(
            model.0,
            transform,
            texture.0,
            tc.0,
            tint,
            blending,
            unsafe { (camera as *const Buffer).as_ref().unwrap() },
        ))
    }

    #[inline(always)]
    fn shape_complex(
        &mut self,
        model: &Model,
        transform: Matrix,
        texture: &Texture,
        tc: TexCoords,
        tints: Gradient,
        blending: bool,
        camera: *const c_void, // native buffer
    ) -> Shape {
        base::new_shape(self.renderer.complex(
            model.0,
            transform,
            texture.0,
            tc.0,
            tints.0,
            blending,
            unsafe { (camera as *const Buffer).as_ref().unwrap() },
        ))
    }

    #[inline(always)]
    fn drop_shape(&mut self, shape: &Shape) {
        self.renderer.drop_shape(get_shape(&shape));
    }

    fn transform(&self, shape: &Shape, transform: Matrix) {
        self.renderer.transform(&base::get_shape(shape), transform);
    }

    fn wh(&self) -> (u16, u16) {
        self.window.wh()
    }

    fn pitch(&self) -> usize {
        self.renderer.pitch()
    }

    fn draw(&mut self, writer: &mut FnMut(*mut u8) -> ()) {
        self.renderer.draw(self.window.wh(), writer)
    }
}
