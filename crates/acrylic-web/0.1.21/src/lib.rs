use acrylic::app::Application;
use acrylic::Size;

extern "C" {
	fn raw_log(s: *const u8, l: usize);
	fn raw_set_request_url(s: *const u8, l: usize);
	fn raw_set_request_url_prefix(s: *const u8, l: usize);
	fn raw_is_request_pending() -> usize;
}

pub fn console_log(s: &str) {
	unsafe {
		raw_log(s.as_ptr(), s.len());
	}
}

pub fn set_request_url(s: &str) {
	unsafe {
		raw_set_request_url(s.as_ptr(), s.len());
	}
}

pub fn set_request_url_prefix(s: &str) {
	unsafe {
		raw_set_request_url_prefix(s.as_ptr(), s.len());
	}
}

pub fn is_request_pending() -> bool {
	unsafe {
		raw_is_request_pending() != 0
	}
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
pub extern fn set_output_size(app: &mut Application, w: usize, h: usize) -> *const u8 {
	{
		let mut root = app.view.lock().expect("could not lock in set_output_size");
		let (p, _) = root.get_spot();
		root.set_spot((p, Size::new(w, h)));
	}
	app.should_recompute = true;
	app.render();
	app.output.pixels.as_ptr()
}

#[export_name = "frame"]
pub extern fn frame(app: &mut Application) {
	app.render();
	ensure_pending_request(app);
}

#[macro_export]
macro_rules! app {
	($path: literal, $init: block) => {
		#[export_name = "init"]
		pub extern fn init() -> &'static Application {
			use std::panic::set_hook;
			set_hook(Box::new(|panic_info| {
				let dbg = format!("{}", panic_info);
				platform::console_log(&dbg);
			}));
			unsafe {
				platform::set_request_url_prefix(&String::from($path));
				platform::APPLICATION = Some($init);
				&platform::APPLICATION.as_ref().unwrap()
			}
		}
	}
}




