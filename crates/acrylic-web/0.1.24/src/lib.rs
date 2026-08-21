use acrylic::app::Application;
use acrylic::node::NodePath;
use acrylic::Spot;
use acrylic::Size;
use acrylic::Point;

use std::collections::HashMap;

extern "C" {
	fn raw_log(s: *const u8, l: usize);
	fn raw_set_request_url(s: *const u8, l: usize);
	fn raw_set_request_url_prefix(s: *const u8, l: usize);
	fn raw_update_blit(x: isize, y: isize, w: usize, h: usize, px: *const u8, d: usize, p: *const u8, l: usize);
	fn raw_is_request_pending() -> usize;
}

pub fn log(s: &str) {
	unsafe { raw_log(s.as_ptr(), s.len()) };
}

pub fn update_blit(p: Point, s: Size, px: &[u8], d: usize, m: &str) {
	unsafe {
		raw_update_blit(p.x, p.y, s.w, s.h, px.as_ptr(), d, m.as_ptr(), m.len())
	};
}

pub fn set_request_url(s: &str) {
	unsafe { raw_set_request_url(s.as_ptr(), s.len()) };
}

pub fn set_request_url_prefix(s: &str) {
	unsafe { raw_set_request_url_prefix(s.as_ptr(), s.len()) };
}

pub fn is_request_pending() -> bool {
	unsafe { raw_is_request_pending() != 0 }
}

pub fn ensure_pending_request(app: &Application) {
	if !is_request_pending() {
		if let Some(data_request) = app.data_requests.last() {
			set_request_url(&data_request.name);
		}
	}
}

#[allow(dead_code)]
pub static mut APPLICATION: Option<Application> = None;
pub static mut RESPONSE_BYTES: Option<Vec<u8>> = None;
pub static mut BLITS_PIXELS: Option<HashMap<NodePath, (Spot, Vec<u8>)>> = None;

pub fn blit<'a>(spot: &'a Spot, path: &'a NodePath) -> (&'a mut [u8], usize, bool) {
	let depth = path.len();
	let (position, size) = *spot;
	let total_pixels = size.w * size.h * 4;
	let (saved_spot, slice) = unsafe {
		let blits = BLITS_PIXELS.as_mut().unwrap();
		if let None = blits.get_mut(path) {
			let pixels = vec![0; total_pixels];
			let spot = (Point::zero(), Size::zero());
			blits.insert(path.clone(), (spot, pixels));
		}
		let (spot, vec) = blits.get_mut(path).unwrap();
		vec.resize(total_pixels, 0);
		(spot, &mut *vec)
	};
	if *saved_spot != *spot {
		*saved_spot = *spot;
		let mut name = String::new();
		for i in path {
			if name.len() > 0 {
				name.push('-');
			}
			name += &format!("{}", i);
		}
		update_blit(position, size, slice, depth, &name);
	}
	(slice, 0, true)
}

#[export_name = "alloc_response_bytes"]
pub extern fn alloc_response_bytes(len: usize) -> *const u8 {
	let mut vec = Vec::with_capacity(len);
	unsafe { vec.set_len(len) };
	let ptr = vec.as_ptr();
	unsafe { RESPONSE_BYTES = Some(vec) };
	ptr
}

#[export_name = "process_response"]
pub extern fn process_response(app: &mut Application) {
	let request = app.data_requests.pop().unwrap();
	let node = app.get_node(&request.node).unwrap();
	let mut node = node.lock().unwrap();
	let data = unsafe {
		RESPONSE_BYTES.as_ref().unwrap()
	};
	node.loaded(app, &request.node, &request.name, 0, data);
}

#[export_name = "drop_response_bytes"]
pub extern fn drop_response_bytes() {
	unsafe {
		RESPONSE_BYTES = None;
	}
}

#[export_name = "discard_request"]
pub extern fn discard_request(app: &mut Application) {
	app.data_requests.pop().unwrap();
}

#[export_name = "set_output_size"]
pub extern fn set_output_size(app: &mut Application, w: usize, h: usize) {
	app.update_spot((Point::zero(), Size::new(w, h)));
}

#[export_name = "frame"]
pub extern fn frame(app: &mut Application) {
	app.render();
	ensure_pending_request(app);
}

pub fn wasm_init(assets: &str, app: Application) -> &'static Application {
	unsafe {
		set_request_url_prefix(&String::from(assets));
		BLITS_PIXELS = Some(HashMap::new());
		APPLICATION = Some(app);
		&APPLICATION.as_ref().unwrap()
	}
}

#[macro_export]
macro_rules! app {
	($path: literal, $init: block) => {
		#[export_name = "init"]
		pub extern fn init() -> &'static Application {
			std::panic::set_hook(Box::new(|panic_info| {
				let dbg = format!("{}", panic_info);
				log(&dbg);
			}));
			platform::wasm_init($path, $init)
		}
	}
}
