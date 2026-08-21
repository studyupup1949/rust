// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

//! Interface with the GPU to render graphics or do fast calculations.

mod base;

pub use self::base::*;

#[cfg(any(
    target_os = "macos",
    target_os = "android",
    target_os = "linux",
    target_os = "windows",
    target_os = "nintendo_switch"
))]
pub(crate) mod vulkan;

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "windows",
    target_os = "web"
))]
pub(crate) mod opengl;

/// Create a new Vulkan / OpenGL Display.
pub(crate) fn new_display() -> Result<Box<Display>, String> {
    let mut err = "".to_string();

    // Try Vulkan first.
    #[cfg(any(
        target_os = "macos",
        target_os = "android",
        target_os = "linux",
        target_os = "windows",
        target_os = "nintendo_switch"
    ))]
    {
        match vulkan::new() {
            Ok(vulkan) => {
                unsafe {
                    // for use of mutable statics.
                    super::super::RENDERER = super::super::Renderer::Vulkan;
                    super::super::CONTEXT = super::super::vulkan::load_functions();
                }

                return Ok(vulkan);
            }
            Err(vulkan) => err.push_str(&vulkan),
        }
        err.push('\n');
    }

    // Fallback on OpenGL/OpenGLES
    #[cfg(any(target_os = "android", target_os = "linux", target_os = "windows",))]
    {
        match opengl::new() {
            Ok(opengl) => {
                unsafe {
                    // for use of mutable statics.
                    super::super::RENDERER = super::super::Renderer::OpenGL;
                    super::super::CONTEXT = super::super::opengl::load_functions();
                }

                return Ok(opengl);
            }
            Err(opengl) => err.push_str(opengl),
        }
        err.push('\n');
    }

    // If neither Vulkan nor OpenGL are available, use CPU graphics.
    unsafe {
        // for use of mutable statics.
        super::super::RENDERER = super::super::Renderer::Barg;
        super::super::CONTEXT = super::super::barg::load_functions();
    }

    // Give up
    err.push_str("No more backend options");
    Err(err)
}
