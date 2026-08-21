/**
 * Aldaron's Device Interface - "screen/mod.rs"
 * Copyright 2017 (c) Jeron Lau - Licensed under the GNU GENERAL PUBLIC LICENSE
**/

use base::Time;

pub mod gui;

mod ffi; // Native window module
pub use self::ffi::{ running };

mod image;

mod sprite;
pub use self::sprite::Sprite;

#[link(name = "vulkan-1")]
mod vw;
pub use self::vw::{ Texture, Style, Shader };

pub struct Screen {
	pub vw: vw::Vw,
	pub rqexit: bool,
	window: ffi::NativeWindow,
	pub size: (u32, u32),
//	back_fn: CallbackFn,
	sprites: Vec<sprite::SpriteData>,
	time: (super::base::Time, f32),
	minsize: (u32, (f32, f32)),
	aspect: f32,
	ymultiply: f32,
}

impl Screen {
	pub fn new(name: &str, icon: &'static [u8], shaders: &[vw::Shader])
		-> (Screen, Vec<Style>)
	{
		println!("ADI - Aldaron's Device Interface / Screen");
		println!("Version: 0.1.0, Backend: Vulkan");

		let native = ffi::native_window(name, icon);
		let vw = vw::open(name, &native);
		let size = (640, 360);
		let aspect = (size.1 as f32) / (size.0 as f32);
		let mut screen = Screen { vw: vw, window: native, size: size,
			/*back_fn: default_back_fn,*/ sprites: Vec::new(),
			time: (super::base::Time::now(), 0.0),
			minsize: (64, (2.0 * 64.0 / 640.0, 2.0 * 64.0 / 360.0)),
			aspect: aspect, ymultiply: 1.0 / aspect, rqexit: false };
		let pipelines = vw::make_styles(&mut screen, shaders);
		println!("new window is done!");
		(screen, pipelines)
	}

	pub fn scalex(&self) -> f32 {
		(self.minsize.1).0
	}

	pub fn scaley(&self) -> f32 {
		(self.minsize.1).1 // * self.ymultiply
	}

	pub fn render(&mut self, color: (f32, f32, f32)) -> () {
		self.clear(color.0, color.1, color.2);
		for i in 0..self.sprites.len() {
			if self.sprites[i].enabled {
				self.sprites[i].shape.draw();
			}
		}
		// TODO: Automatically decrease to 30fps if needed.
		self.regulate(60); // 60 fps
		// Update Screen
		vw::draw_update(self);
	}

	pub fn cleanup(&mut self) -> () {
		vw::close(self.vw);
		ffi::cleanup(&mut self.window);
	}

	fn clear(&mut self, r:f32, g:f32, b:f32) -> () {
		vw::draw_clear(self, r, g, b);
	}

	fn regulate(&mut self, fps: u32) -> () {
		let interval = 1.0 / (fps as f32);
		let timed = self.time.0.seconds_since();
		let last = self.time.1;
		let passed = timed - last;
		let free_time = interval - passed;

		if free_time > 0.0 {
			Time::sleep(free_time);
		}

		let timed = self.time.0.seconds_since();

		// reset
		self.time.1 = timed;
	}

	pub fn resize(&mut self, w: u32, h: u32) {
		self.size = (w, h);
		(self.minsize.1).0 = 2.0 * (self.minsize.0 as f32) / (w as f32);
		(self.minsize.1).1 = 2.0 * (self.minsize.0 as f32) / (h as f32);
		self.aspect = (h as f32) / (w as f32);
		self.ymultiply = 1.0 / self.aspect;
		vw::resize(self);
	}

	pub fn toggle_fullscreen(&mut self) {
		ffi::toggle_fullscreen(&mut self.window);
	}

	pub fn stop(&mut self) {
		self.rqexit = true;
	}

	pub fn keep(&mut self) {
		self.rqexit = false;
	}
}
