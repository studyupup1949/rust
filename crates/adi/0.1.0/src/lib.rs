/**
 * Aldaron's Device Interface - "lib.rs"
 * Copyright 2017 (c) Jeron Lau - Licensed under the GNU GENERAL PUBLIC LICENSE
**/

pub mod base;
pub mod screen;
pub mod input;

pub use screen::Screen;
pub use input::Input;

#[cfg(test)]
mod tests {
//	#[test]
//	fn it_works() {
//	}
}
