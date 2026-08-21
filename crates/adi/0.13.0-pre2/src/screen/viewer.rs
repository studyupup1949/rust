// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

use ami::{Matrix, Rotation, Vector};
use std::os::raw::c_void;

use crate::screen::screen::{Gradient, Model, Shape, TexCoords, Texture};

/// A `Viewer` Object (viewing position, and direction).
pub struct Viewer(*mut c_void); // TODO: Needs window created before this is created.

fn posdir_to_matrix(pos: Vector, dir: Vector) -> Matrix {
    matrix!()
        .t(-pos)
        .r(Rotation::euler(-dir))
        .m(matrix!(unsafe { (super::CONTEXT.projection)() }))
}

impl Viewer {
    /// Create a new `Viewer` object.
    pub fn new(pos: Vector, dir: Vector) -> Self {
        // Load function
        let viewer_new = unsafe { (super::CONTEXT.viewer_new) };

        // Run Function & Return
        Viewer(viewer_new(posdir_to_matrix(pos, dir).into()))
    }

    /// Set the position of the viewer.
    pub fn set(&mut self, pos: Vector, dir: Vector) {
        // Load function
        let viewer_set = unsafe { super::CONTEXT.viewer_set };

        // Run Function
        viewer_set(&mut self.0, posdir_to_matrix(pos, dir).into());
    }

    /// Get the platform-specific pointer.
    pub fn get(&self) -> *mut c_void {
        self.0
    }

    /// Make a shape with solid color.
    pub fn add_solid(
        &self,
        //        screen: &mut App,
        model: &Model,
        matrix: Matrix,
        color: [f32; 4],
        blending: bool,
    ) -> Shape {
        crate::shared::screen()
            .display
            .shape_solid(model, matrix, color, blending, self.get())
    }

    /// Make a shape with gradient
    pub fn add_gradient(
        &self,
        //        screen: &mut App,
        model: &Model,
        matrix: Matrix,
        gradient: Gradient,
        blending: bool,
    ) -> Shape {
        crate::shared::screen().display.shape_gradient(
            model,
            matrix,
            gradient,
            blending,
            self.get(),
        )
    }

    /// Make a shape will solid texture.
    pub fn add_textured(
        &self,
        //        screen: &mut App,
        model: &Model,
        matrix: Matrix,
        texture: &Texture,
        tc: TexCoords,
        blending: bool,
    ) -> Shape {
        crate::shared::screen().display.shape_texture(
            model,
            matrix,
            texture,
            tc,
            blending,
            self.get(),
        )
    }

    /// Make a shape will texture and transparency
    pub fn add_faded(
        &self,
        //        screen: &mut App,
        model: &Model,
        matrix: Matrix,
        texture: &Texture,
        tc: TexCoords,
        alpha: f32,
    ) -> Shape {
        crate::shared::screen()
            .display
            .shape_faded(model, matrix, texture, tc, alpha, self.get())
    }

    /// Make a shape with texture, and tint (color)
    pub fn add_tinted(
        &self,
        //        screen: &mut App,
        model: &Model,
        matrix: Matrix,
        texture: &Texture,
        tc: TexCoords,
        tint: [f32; 4],
        blending: bool,
    ) -> Shape {
        crate::shared::screen().display.shape_tinted(
            model,
            matrix,
            texture,
            tc,
            tint,
            blending,
            self.get(),
        )
    }

    /// Make a shape with texture, and gradent
    pub fn add_complex(
        &self,
        //        screen: &mut App,
        model: &Model,
        matrix: Matrix,
        texture: &Texture,
        tc: TexCoords,
        gradient: Gradient,
        blending: bool,
    ) -> Shape {
        crate::shared::screen().display.shape_complex(
            model,
            matrix,
            texture,
            tc,
            gradient,
            blending,
            self.get(),
        )
    }
}

impl Drop for Viewer {
    fn drop(&mut self) {
        // Load function
        let viewer_old = unsafe { (super::CONTEXT.viewer_old) };

        // Run Function
        viewer_old(self.0);
    }
}
