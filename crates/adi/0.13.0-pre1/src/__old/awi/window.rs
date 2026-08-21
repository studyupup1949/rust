// Copyright Jeron A. Lau 2017-2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

// use c_void;

use screen::awi::os;
use screen::awi::Keyboard;

/// A graphics window on a computer, linked to a rendering API.
pub(crate) struct Window {
    os_window: os::Window, /* *mut c_void */
    keyboard: Keyboard,
}

impl Window {
    /// Create a window, using `title` as the title, and `icon` as the
    /// window icon.  The format of icon is as follows:
    /// `(width, height, pixels)`.  You can load icons with aci.  `v` should
    /// be either `None` or `Some(visual_id from EGL)`.
    pub fn new(v: Option<i32>) -> Window {
        let os_window = os::Window::new(v);
        let keyboard = Keyboard::new();

        Window {
            os_window,
            keyboard,
        }
    }

    /// Get the type of connection, plus native window and connection
    /// handles to pass to ffi.  See `WindowConnection` for more details.
    pub fn get_connection(&self) -> ::screen::awi::WindowConnection {
        self.os_window.get_connection()
    }

    /// Get the width and height of the window, as a tuple.
    pub fn wh(&self) -> (u16, u16) {
        self.os_window.wh()
    }

    /// Poll window input, return `None` when finished.  After returning
    /// `None`, the next call will update the window.
    pub fn update(&mut self) {
        // Get window events, and update keyboard state.
        while self
            .os_window
            .poll_event(&mut self.keyboard)
        {}

        // New Frame
        self.update();
    }
}
