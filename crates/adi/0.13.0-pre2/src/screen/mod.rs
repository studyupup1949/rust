// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

//! # Screen Feature
//! Render graphics to a computer monitor, laptop display, or phone screen.
//!
//! In order to do this we need to create an `App`.  To do this we:
//!
//! ```
//! #[macro_use]
//! extern crate adi;
//!
//! use adi::{App, screen, hid};
//!
//! main!(
//!     Ctx,
//!     struct Ctx {
//!         mode: fn(app: &mut Ctx),
//!     }
//! );
//!
//! impl App for Ctx {
//!     fn new() -> Ctx {
//!         Ctx {
//!             mode: mode,
//!         }
//!     }
//!
//!     fn run(&mut self) {
//!         (self.mode)(self)
//!     }
//! }
//!
//! fn mode(app: &mut App, _ctx: &mut Ctx, _runner: &mut Runner<Ctx>, event: Event, _dt: f32) {
//!     // Check for exit request
//!     if hid::Key::Back.pressed(0) {
//!         adi::old();
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![doc(html_logo_url = "http://plopgrizzly.com/adi_screen/icon.png",
    html_favicon_url = "http://plopgrizzly.com/adi_screen/icon.ico",
    html_root_url = "http://plopgrizzly.com/adi_screen/")]

#[cfg(any(
    target_os = "macos",
    target_os = "android",
    target_os = "linux",
    target_os = "windows",
    target_os = "nintendo_switch"
))]
mod vulkan;

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "windows",
    target_os = "web"
))]
mod opengl;

mod barg;

mod gpu_data;
pub(crate) mod screen;
mod shared;
mod viewer;

mod ffi;

/// Prelude module.
pub mod prelude;

use std::os::raw::c_void;

pub use self::viewer::Viewer;

pub use crate::screen::ffi::render::afi::*;
pub use crate::screen::ffi::render::Matrix;
pub use crate::screen::screen::{App, Gradient, Model, Shape, TexCoords, Texture};

/// Get the pitch of the window's surface.
pub fn pitch() -> usize {
    crate::shared::screen().pitch()
}

/// Get the width and height of the window.
pub fn wh() -> (u16, u16) {
    crate::shared::screen().wh()
}

/// Update window's surface.
pub fn draw(writer: &mut FnMut(*mut u8) -> ()) {
    crate::shared::screen().draw(writer)
}

/// Get the delta timestep (amount of time since previous frame).
pub fn dt() -> f32 {
    crate::shared::screen().dt
}

/// Create texture coordinate object.
pub fn texcoords(texcoords: &[(f32, f32)]) -> TexCoords {
    crate::shared::screen().texcoords(texcoords)
}

/// Update the clear color of the Window.
pub fn clear(color: (u8, u8, u8)) {
    crate::shared::screen().clear(color)
}

/// Upload a texture to the GPU.
pub fn texture(wh: (u16, u16), graphic: &VFrame) -> Texture {
    crate::shared::screen().texture(wh, graphic)
}

/// A Platform Dependant Struct of Rendering Functions.
pub(crate) struct PlatformDependant {
    // Native Window Projection Matrix (based on aspect ratio).
    projection: fn() -> Matrix,

    // Put the linked Matrix into storage (location is implementation dependant)
    viewer_new: fn(mat: [f32; 16]) -> *mut c_void,
    // Set the linked Matrix.
    viewer_set: fn(viewer: &mut *mut c_void, mat: [f32; 16]),
    // Free Matrix from storage.
    viewer_old: fn(viewer: *mut c_void),
    // Put the Texture into storage  (location is implementation dependant)
    //    texture_new: fn(wh: (u16, u16), tex: &[u8]) -> *mut c_void,
    // Set the Texture.
    //    texture_set: fn(texture: &mut *mut c_void, writer: &mut FnMut(*mut u8) -> ()),
    // Free Texture from storage.
    //    texture_old: fn(texture: *mut c_void),
}

/// Type of renderer that is available and in use.
#[derive(Copy, Clone)]
pub enum Renderer {
    /// OpenGL / OpenGLES / WebGL
    OpenGL,
    /// Vulkan
    Vulkan,
    /// Barg
    Barg,
}

impl Renderer {
    /// Get the renderer in use.
    pub fn get() -> Renderer {
        unsafe { RENDERER }
    }
}

/// Try Vulkan by default, OpenGL / Metal backup.
static mut RENDERER: Renderer = Renderer::Vulkan;

static mut CONTEXT: PlatformDependant = PlatformDependant {
    projection: self::barg::projection,
    viewer_new: self::barg::viewer_new,
    viewer_set: self::barg::viewer_set,
    viewer_old: self::barg::viewer_old,
    //    texture_new: self::barg::texture_new,
    //    texture_set: self::barg::texture_set,
    //    texture_old: self::barg::texture_old,
};
