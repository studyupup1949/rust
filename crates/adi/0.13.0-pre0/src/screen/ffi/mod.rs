// Copyright Jeron A. Lau 2017-2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

pub mod render;

/// Platform-Specific Windowing API
mod native {
    /* Linux / BSD: Use XCB/XKB (Either OpenGL, OpenGLES or Vulkan) */
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "bitrig",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    include!("linux.rs");

    /* Windows (Either OpenGL or Vulkan) */
    #[cfg(target_os = "windows")]
    include!("windows.rs");

    /* Wasm (WebGL) */
    #[cfg(target_arch = "wasm32")]
    include!("wasm.rs");

    /* Android (Either OpenGLES or Vulkan) */
    #[cfg(target_os = "android")]
    include!("android.rs");
}

/// Native Window
pub(crate) use self::native::Window;
pub use ami::*;
