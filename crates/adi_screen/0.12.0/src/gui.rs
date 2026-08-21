// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

/// A font that's built into the library.
pub const DEFAULT_FONT: &'static [u8] = include_bytes!("res/font/DejaVuSansMono.ttf");

pub use fonterator::{ Font };
