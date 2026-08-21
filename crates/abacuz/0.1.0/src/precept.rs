use actix_web::error::ErrorForbidden;
use actix_web::{HttpRequest, HttpResponse};
use futures::executor;

use std::time::Duration;

use super::guard;
use super::SrvData;
use super::SrvResult;

pub fn stop(req: &HttpRequest, srv_data: &SrvData) -> SrvResult {
	if let Some(user) = guard::get_user(req, srv_data) {
		if user.master {
			let data_server = srv_data.server.read().unwrap();
			if let Some(server) = &*data_server {
				let result = String::from("Abacuz is stopping...");
				println!("{}", result);
				executor::block_on(server.stop(false));
				return Ok(HttpResponse::Ok().body(result));
			}
		}
	}
	Err(ErrorForbidden(
		"You don't have access to call this resource.",
	))
}

static SLEEP_TO_SHUTDOWN: Duration = Duration::from_millis(1000);

pub fn shut(req: &HttpRequest, srv_data: &SrvData) -> SrvResult {
	if let Some(user) = guard::get_user(req, srv_data) {
		if user.master {
			let result = String::from("Abacuz is shutting...");
			println!("{}", result);
			std::thread::spawn(|| {
				std::thread::sleep(SLEEP_TO_SHUTDOWN);
				std::process::exit(0);
			});
			return Ok(HttpResponse::Ok().body(result));
		}
	}
	Err(ErrorForbidden(
		"You don't have access to call this resource.",
	))
}
