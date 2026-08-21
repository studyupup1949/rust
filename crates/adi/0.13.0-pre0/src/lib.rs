// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

//! Lightweight platform-agnostic device interface abstraction layer for
//! creating apps and video games.
//!
//! Aldaron's Device Interface aims to be the new [SDL](https://www.libsdl.org/)
//! in some aspects, but isn't specifically targeting video games.  This library
//! also aims to replace cross-platform GUI toolkits like
//! [GTK](https://www.gtk.org/).  Ultimately, though, it's cooler than both
//! combined, and way smaller!  Since those libraries are 3-letter acronyms,
//! this library must also be: [ADI](https://aldaron.tk/crates/)!
//!
//! # Getting Started
//! Rather than having a bunch of separately maintained projects like SDL,
//! everything is together in ADI.  By default, everything is built (all of the
//! device interfaces).  This usually is not what you want, so this is how you
//! should specify ADI in your `Cargo.toml` if you only want the `screen` API:
//! ```TOML
//! [dependencies.adi]
//! version = "0.13"
//! default-features = false
//! features = ["screen"]
//! path = "../adi"
//! ```
//! Thanks to Cargo, that's a lot easier than finding all of the SDL libraries!
//! Conveniently, the features have the same name as the modules in this
//! documentation, to make it even easier!  Have fun!

#![warn(missing_docs)]
#![doc(
    html_logo_url = "https://plopgrizzly.com/images/adi.png",
    html_favicon_url = "https://plopgrizzly.com/images/plopgrizzly-splash.png"
)]

extern crate libc;

/// Screen interface API.
#[cfg(feature = "screen")]
pub mod screen;

/// Speaker interface API.
#[cfg(feature = "speaker")]
pub mod speaker;

/// Microphone interface API.
#[cfg(feature = "mic")]
pub mod mic;

/// Human Interface Device API (input API).
#[cfg(feature = "hid")]
pub mod hid;

/*
/// Camera / Webcam Interface API.
#[cfg(feature = "cam")]
pub mod cam;
*/

// Shared connections between devices.
mod shared;

#[cfg(feature = "ami")]
#[macro_use]
extern crate ami;
#[cfg(feature = "afi")]
pub(crate) extern crate afi;
#[cfg(feature = "barg")]
pub extern crate barg;
#[cfg(feature = "stick")]
extern crate stick;
#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate dl_api;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb_derive;

pub use shared::{new, old, App};
