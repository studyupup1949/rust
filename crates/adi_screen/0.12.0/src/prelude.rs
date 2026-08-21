// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

pub use adi_gpu::*;

/// Macro to create multiple sprites in an array.
///
/// # Example
/// ```
/// let mut window = WindowBuilder::new("Window Name", None).finish();
/// 
/// textures!(window, aci_png::decode,
/// 	"res/texture0.png", // 0
/// 	"res/texture1.png", // 1
/// );
/// 
/// models!(window, "res/model.data");
/// 
/// let sprites = sprites!(window,
/// 		(0/*model 0*/, Some(0/*texture 0*/),
/// 	Transform::new().translate(0.0, -0.5, 2.0), false),
/// 		(0/*model 0*/, Some(0/*texture 0*/),
/// 	Transform::new().translate(0.0, -4.5, 2.0), false));
/// ```
#[macro_export] macro_rules! sprites {
	($window:expr, $( $x:expr ),*) => { {
		let window = $window;
		[ $( $crate::Sprite::new(window, $x.0, $x.1, $x.2, $x.3, false,
			true) ),* ]
	} }
}

/// Macro to create multiple fog-affected sprites in an array.
/// # Example
/// See [`sprites!()`](macro.sprites.html)
#[macro_export] macro_rules! sprites_fog {
	($window:expr, $( $x:expr ),*) => { {
		let window = $window;
		[ $( $crate::Sprite::new(window, $x.0, $x.1, $x.2, $x.3, true,
			true) ),* ]
	} }
}

/// Macro to create multiple non-camera affected sprites in an array.
/// # Example
/// See [`sprites!()`](macro.sprites.html)
#[macro_export] macro_rules! sprites_gui {
	($window:expr, $( $x:expr ),*) => { {
		let window = $window;
		[ $( $crate::Sprite::new(window, $x.0, $x.1, $x.2, $x.3, false,
			false) ),* ]
	} }
}

/// Macro to load textures from files for the window.
///
/// The first texture file listed is indexed 0, and each subsequent texture
/// increases by 1.  See: [`sprites!()`](macro.sprites.html) for example.
#[macro_export] macro_rules! textures {
	($window:expr, $decode:expr, $( $x:expr ),*) => { {
		let a = vec![ $( {
			let mut video = $decode(include_bytes!($x), afi::Rgba)
				.unwrap();
			let wh = video.wh();
			let px = video.pop().unwrap();

			$crate::Texture::new($window, wh, &px)
		} ),* ];

		$window.textures(a);
	} }
}
