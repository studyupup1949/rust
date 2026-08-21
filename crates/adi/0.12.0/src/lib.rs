// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

//! Create platform-agnostic apps and video games (similar to SDL).

#![warn(missing_docs)]
#![doc(html_logo_url = "https://plopgrizzly.com/images/adi.png",
   html_favicon_url = "https://plopgrizzly.com/images/plopgrizzly-splash.png")]

/// Screen interface API.
#[cfg(feature = "screen")]
pub mod screen { extern crate adi_screen; pub use self::adi_screen::*; }

/// Speaker interface API.
#[cfg(feature = "speaker")]
pub mod speaker { extern crate adi_speaker; pub use self::adi_speaker::*; }
