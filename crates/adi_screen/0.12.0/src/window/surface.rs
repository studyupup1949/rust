// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use footile::{PathBuilder, Plotter, Raster, FillRule};

/// A surface to render on
pub struct Surface {
	path: Option<PathBuilder>,
	plotter: Plotter,
	raster: Raster,
	sum: f32,
	last: (f32, f32),
}

impl Surface {
	pub fn new(w: u32, h: u32) -> Self {
		Surface {
			path: None,
			plotter: Plotter::new(w, h),
			raster: Raster::new(w, h),
			sum: 0.0,
			last: (0.0, 0.0),
		}
	}

	/// Start drawing at x and y position.
	pub fn move_to(&mut self, x: f32, y: f32) {
		self.plotter.clear();
		self.plotter.reset();
		self.path = Some(PathBuilder::new().absolute().move_to(x, y));
		self.sum = 0.0;
		self.last = (x, y);
	}

	pub fn line_to(&mut self, x: f32, y: f32) {
		let mut path = None;
		::std::mem::swap(&mut path, &mut self.path);
		path = Some(path.unwrap().line_to(x, y));
		::std::mem::swap(&mut path, &mut self.path);
		self.sum += (x - self.last.0) * (y + self.last.1);
		self.last = (x, y);
	}

	pub fn quad_to(&mut self, x: f32, y: f32, cx: f32, cy: f32) {
		let mut path = None;
		::std::mem::swap(&mut path, &mut self.path);
		path = Some(path.unwrap().quad_to(cx, cy, x, y));
		::std::mem::swap(&mut path, &mut self.path);
		self.sum += (x - self.last.0) * (y + self.last.1);
		self.last = (x, y);
	}

	pub fn draw(&mut self, rgba: [u8; 4]) {
		if self.path.is_none() { return }
		let mut path = None;
		::std::mem::swap(&mut path, &mut self.path);
		let path = path.unwrap().close().build();
		self.plotter.add_ops(&path);
		self.plotter.fill(FillRule::NonZero);
		if self.sum < 0.0 {
			self.raster.composite(self.plotter.mask(), rgba);
		} else {
			self.raster.cut(self.plotter.mask());
		}
	}

	pub fn to_vec(self) -> Vec<u8> {
		self.raster.get_pixels().2.to_vec()
	}
}
