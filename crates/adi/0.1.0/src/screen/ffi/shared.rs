/*
 * Aldaron's Device Interface - "screen/ffi/shared.rs"
 * Copyright 2017 (c) Jeron Lau - Licensed under the GNU GENERAL PUBLIC LICENSE
**/

use Screen;

pub const MWW : u32 = 640;
pub const MWH : u32 = 360;

pub fn convert_mouse_pos(screen: &Screen, c: (i16, i16)) -> (f32,f32) {
	let x = ((c.0 as f32) / (screen.size.0 as f32) * 2.0) - 1.0;
	let y = ((c.1 as f32) / (screen.size.1 as f32) * 2.0) - 1.0;
	(x, y)
}