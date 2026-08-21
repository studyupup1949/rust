extern crate actix_web;
extern crate serde;

extern crate actix_proc_macros;
pub use actix_proc_macros::*;

#[allow(type_alias_bounds)] // Even if this isn't enforced, I want to express intent explicitly to human readers
pub type ResponderResult<T: serde::Serialize> = Result<Response<T>, Box<dyn std::error::Error>>;

pub enum Response<T: serde::Serialize> {
	Json(T),
	Text(String),
	Builder(actix_web::dev::HttpResponseBuilder)
}

#[macro_export]
macro_rules! code {
    ($code: ident) => { ::actix_helper_macros::Response::Builder(::actix_web::HttpResponse::$code()) }
}

#[macro_export]
macro_rules! text {
	($val: expr) => { ::actix_helper_macros::Response::Text($val.to_string()) };
}

#[macro_export]
macro_rules! json {
    ($val: expr) => { ::actix_helper_macros::Response::Json($val) };
}

#[macro_export]
macro_rules! or_404 {
    ($item: expr) => {
        match $item {
            None => { return Ok(code!(NotFound)) }
            Some(v) => v
        }
    }
}

