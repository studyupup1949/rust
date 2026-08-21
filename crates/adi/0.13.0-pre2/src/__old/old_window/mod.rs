// Copyright Jeron A. Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use Event;
use awi::render::afi::{VFrame, PathOp};
pub use awi::screen::{Screen as AwiScreen, ScreenError};
use aci_png;
use Texture;
use Model;
use Vec3;
use Sprite;
use Transform;

/// Activity
pub type Activity = fn(screen: Screen, event: Event, dt: f32);

/// A GUI Widget for the Window
pub enum Widget<'a> {
	/// No widget - or a spacer widget. Grows to the largest size it can be.
	Empty,
	/// A vector graphic.
	Graphic(&'a Iterator<Item = PathOp>),
	/// A pixel image.
	Image(&'a VFrame),
	/// A text block, formatted in markdown.
	Text(&'a str, [u8;4]),
	/// A background wrapper for another widget.
	Color(&'a Widget<'a>, [u8;4]),
	/// A listen wrapper for another widget.
	Listen(&'a Widget<'a>, Box<FnMut(Event)>),
}

// TODO: make into tree-generating macros.
impl<'a> Widget<'a> {
	/// Get widget for a button widget.
	pub fn button(widget: &'a Widget<'a>, _run: &'a mut FnMut())
		-> Widget<'a>
	{
		Widget::Listen(widget, Box::new(move |input| {
			match input {
				_ => { /*not pressed, do nothing*/ }
			}
		}))
	}

	/// Get widget for a mutable text field.
	pub fn field(text: &'a mut String, widget: &'a mut Widget<'a>,
		rgba: [u8;4]) -> Widget<'a>
	{
		*widget = Widget::Text(text, rgba);
		Widget::Listen(widget, Box::new(move |input| {
			match input {
				_ => { /*not pressed, do nothing*/ }
			}
		}))
	}

/*	/// Get widget for window titlebar with window name, allows moving
	/// window by dragging with the left mouse button.  Right-click or
	/// Ctrl-click or tap on titlebar sends a menu button event
	pub fn titlebar(widget: &'a mut [Widget<'a>; 2]) -> Widget<'a> {
		widget[0] = Widget::Text("TITLE TODO");
		widget[1] = Widget::Color(&widget[0], 0x44_00_FF_BB);

		Widget::Listen(&widget[1], Box::new(move |input| {
			match input {
				_ => { /*not pressed, do nothing*/ }
			}
		}))
	}*/
}

/// A builder for `Window`.
struct WindowBuilder {
	fog: Option<(f32, f32)>,
	rgb: (u8, u8, u8),
}

fn init_a(screen: Screen, event: Event, dt: f32) {
	let mut img = aci_png::decode(
		include_bytes!("../res/button.png"),
		::awi::render::afi::Srgba
	).unwrap();
	let wh = img.wh();
	let px = img.pop().unwrap();
	let _button = Texture(native.texture(wh, &px), wh.0, wh.1);

	native.clear(self.rgb);
	native.fog(self.fog);

	let overlay_texture = Texture(
		native.texture((1,1), &VFrame(vec![255, 0, 0, 200])),
		1, 1);
	let overlay_model = native.model(&[
		-1.0, -1.0, 0.0, 1.0,
		-1.0, 1.0, 0.0, 1.0,
		1.0, 1.0, 0.0, 1.0,
		1.0, -1.0, 0.0, 1.0
	], vec![(0, 4)]);
	let texcoords = native.texcoords(&[
		0.0, 0.0, 1.0, 1.0,
		0.0, 1.0, 1.0, 1.0,
		1.0, 1.0, 1.0, 1.0,
		1.0, 0.0, 1.0, 1.0
	]);
	let overlay = Sprite(native.shape_texture(&overlay_model,
		Transform::IDENTITY, &overlay_texture.0, texcoords,
		true, false, false));

	Window {
		since_frame: 0.0,
		minsize: 40, aspect: 0.0, _button,
		seconds: 0.0, fps_counter: 0, fps: 0,
		font: ::gui::Font::new(::gui::DEFAULT_FONT).unwrap(),
		textures: vec![], models: vec![], overlay_texture,
		overlay, layout: Grid { grid: vec![], widget: 0 },
		widgets: vec![], overlay_update: false,
	}	
}

/*impl WindowBuilder {
	/// A new `WindowBuilder`.
	pub fn new() -> Self {
		WindowBuilder {
			fog: None,
			rgb: (0, 127, 0),
		}
	}

	/// Set fog distance and fog depth, off by default.
	pub fn fog(mut self, fog_distance: f32, fog_depth: f32) -> Self {
		self.fog = Some((fog_distance, fog_depth));
		self
	}

	/// Set background color, white by default.
	pub fn background(mut self, rgb: (u8, u8, u8)) -> Self {
		self.rgb = rgb;
		self
	}

	/// Finish building the `Window`.
	pub fn start<'a>(self) -> Result<(), ScreenError> {
		

		AwiScreen::start(&mut |screen, event, dt| {
			let screen = Screen(screen);

			
		})?;

		Ok(())
	}
}*/

/// Connection to a computer/phone screen.
pub struct Screen<Ctx>(&'static mut AwiScreen<Window<'static, Ctx>>)
	where Ctx: Default;

impl<T> Screen<T> {
	pub fn start() -> Result<(), ScreenError> {
		let run: Activity = init_a;

		AwiScreen::<Window>::start(&mut |screen, event, dt| {
			let screen = Screen(screen);

			
		})
	}
}

#[derive(Clone, Default)]
struct Grid {
	grid: Vec<Vec<Grid>>,
	widget: u16,
}

/// Window represents a connection to a display that can also recieve input.
#[derive(Default)]
pub struct Window<'a, Ctx> where ={
	since_frame: f32,
	minsize: u16,
	aspect: f32,
	// Frame Rate Counting
	seconds: f32,
	fps_counter: u16,
	fps: u16,
	// Button Texture
	pub(crate) _button: Texture,
	// Default Font
	pub(crate) font: ::gui::Font<'static>,
	// 
	pub(crate) textures: Vec<Texture>,
	pub(crate) models: Vec<Model>,
	// 2 dimensional overlay.
	overlay: Sprite,
	overlay_texture: Texture,
	overlay_update: bool,
	layout: Grid,
	widgets: Vec<Widget<'a>>,
	// Context
	pub ctx: Ctx,
}

pub trait WindowFunctions {
	fn unit_ratio(&self) -> f32;
	fn wh(&self) -> (u16, u16);
}

impl<'a> WindowFunctions for Window<'a> {
	fn unit_ratio(&self) -> f32 {
		self.aspect
	}

	fn wh(&self) -> (u16, u16) {
		self.window.wh()
	}
}

impl<'a, Ctx> Window<'a, Ctx> where Ctx: Default {
/*	/// Start rendering to the screen.
	pub fn start(fog: [f32; 2], bg: [u8; 3]) -> Result<(), ScreenError> {
		WindowBuilder::new()
			.fog(fog[0], fog[1])
			.background((bg[0], bg[1], bg[2]))
			.start()
	}*/

	/// Update fog distance.
	pub fn fog(&mut self, fog: Option<(f32, f32)>) {
		self.window.fog(fog);
	}

	/// Set the background color of the window.
	pub fn background(&mut self, rgb: (u8, u8, u8)) -> () {
		self.window.clear(rgb);
	}

	/// Adjust the location and direction of the camera.
	pub fn camera(&mut self, xyz: Vec3, rotate_xyz: Vec3) {
		self.window.camera(xyz, rotate_xyz);
	}

//	/// Get the minimal x and y dimension for a widget.
//	pub fn unit_size(&self) -> (f32, f32) {
//		self.minsize.1
//	}

	/// Update the window and return the user input.  This should run in a
	/// loop.  Returns `None` when done looping through input.  After `None`
	/// is returned, the next call will update the screen.
	pub fn update(&mut self) -> Option<Event> {
		let mut input = self.window.input();

		if input == None {
			// Update Screen
			self.since_frame = self.window.update();
			self.seconds += self.since_frame;

			// Count FPS
			self.fps_counter += 1;

			if self.seconds >= 1.0 {
				self.seconds -= 1.0;
				self.fps = self.fps_counter;
				self.fps_counter = 0;
				println!("{}", self.fps);
			}
		}

		if input == None && self.aspect == 0.0 {
			input = Some(Event::Resize);
		}

		if input == Some(Event::Resize) {
			let wh = self.wh();
			let (w, h) = (wh.0 as f32, wh.1 as f32);

			self.window.resize(wh);
			self.aspect = h / w;
			self.overlay_update = true;
		}

		if self.overlay_update {
			let wh = self.wh();
			let mut dimensions: Vec<(u16, Vec<u16>)> = vec![];

			// Calculate widths heights of each one.
			for row in self.layout.grid.iter() {
				let mut rowd = (self.minsize, vec![]);

				for widget in row.iter() {
					match &self.widgets[widget.widget as usize] {
						Widget::Empty => println!("Empty"),
						Widget::Graphic(_vg) => println!("Graphic"),
						Widget::Image(_vframe) => println!("Image"),
						Widget::Text(string, color) => {
							let mut x = 0.0;

							// Loop through the glyphs in the text.
							for g in self.font.glyphs(string, (self.minsize as f32, self.minsize as f32)) {
								// Draw the glyph
								self.window.draw(g.0.draw(x, 0.0), *color);

								// Position next glyph
								x += g.1;
							}
						},
						Widget::Color(_widget, _color) => println!("Color"),
						Widget::Listen(_widget, _closure) => println!("Listen"),
					}
					rowd.1.push(self.minsize);
				}

				dimensions.push(rowd);
			}

			let overlay_model = self.window.model(&[
				-1.0, -1.0, 0.0, 1.0,
				-1.0, 1.0, 0.0, 1.0,
				1.0, 1.0, 0.0, 1.0,
				1.0, -1.0, 0.0, 1.0
			], vec![(0, 4)]);
			let texcoords = self.window.texcoords(&[
				0.0, 0.0, 1.0, 1.0,
				0.0, 1.0, 1.0, 1.0,
				1.0, 1.0, 1.0, 1.0,
				1.0, 0.0, 1.0, 1.0
			]);
			let overlay_texture = /*self.window.draw_update*/();
			self.window.set_texture(&mut self.overlay_texture.0, wh,
				&overlay_texture
			);
			{
				self.window.drop_shape(&self.overlay.0);
			}
			self.overlay = Sprite(self.window.shape_texture(
				&overlay_model, Transform::IDENTITY,
				&self.overlay_texture.0, texcoords, true, false,
				false
			));
			
			self.overlay_update = false;
		}

		input
	}

/*	/// Returns a number between 0-1. This function is used for animations.
	/// It will take rate_spr seconds to go from 0 to 1. 
	pub fn pulse_half_linear(&self, rate_spr: f32) -> f32 {
		self.time.pulse_half_linear(rate_spr)
	}

	/// Returns a number between 0-1. This function is used for animations.
	/// It will take rate_spr seconds to go from 0 to 1 and back to 0.
	pub fn pulse_full_linear(&self, rate_spr: f32) -> f32 {
		self.time.pulse_full_linear(rate_spr)

	}

	/// Returns a number between 0-1. This function is used for animations.
	/// It will take rate_spr seconds to go from 0 to 1. It uses cosine
	/// underneath to make the animation look smooth, by making the
	/// beginning and end of the animation slower than the middle.
	pub fn pulse_half_smooth(&self, rate_spr: f32) -> f32 {
		self.time.pulse_half_smooth(rate_spr)
	}

	/// Returns a number between 0-1. This function is used for animations.
	/// It will take rate_spr seconds to go from 0 to 1 and back to 0. It
	/// uses cosine underneath to make the animation look smooth, by making
	/// the beginning and end of the animation slower than the middle.
	pub fn pulse_full_smooth(&self, rate_spr: f32) -> f32 {
		self.time.pulse_full_smooth(rate_spr)
	}*/

	/// Get the time passed since the previous window frame.
	pub fn since(&self) -> f32 {
		self.since_frame
	}

	/// Return the current number of FPS the window is running at.
	pub fn fps(&self) -> (u16, bool) {
		(self.fps, self.fps_counter == 0)
	}

	/// Set the GUI for this `Window`.  Position is (column/x, row/y).
	pub fn widget(&mut self, position: &[(u8, u8)], widget: Widget<'a>) {
		// Add widget to collection.
		let id = self.widgets.len() as u16;
		self.widgets.push(widget);

		// Add widget to layout tree.
		let mut grid = &mut self.layout;

		for i in position.iter() {
			let temp = grid;
			let (x, y) = i;

			if *y as usize >= temp.grid.len() {
				temp.grid.resize(*y as usize + 1, vec![]);
			}

			if *x as usize >= temp.grid[*y as usize].len() {
				temp.grid[*y as usize].resize(*x as usize + 1, Grid {
					grid: vec![],
					widget: ::std::u16::MAX,
				});
			}

			grid = &mut temp.grid[*y as usize][*x as usize];
		}

		grid.widget = id;

		self.overlay_update = true;
	}

	#[doc(hidden)]
	pub fn models(&mut self, models: Vec<Model>) {
		self.models = models;
	}

	#[doc(hidden)]
	pub fn textures(&mut self, textures: Vec<Texture>) {
		self.textures = textures;
	}
}
