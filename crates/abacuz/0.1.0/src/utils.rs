use actix_web::error::{Error, ErrorBadRequest};
use actix_web::{web::Bytes, HttpRequest};

use super::data::User;

use std::fmt::Display;
use std::marker::Sized;

macro_rules! origin {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }}
}

pub(crate) use origin;

pub fn debug<TD: Display + ?Sized>(origin: &'static str, error: &TD) -> String {
    let message = format!("Problem on {} - {}", origin, error);
    eprintln!("{}", message);
    message
}

pub fn get_body(bytes: Bytes) -> Result<String, Error> {
    match String::from_utf8(bytes.to_vec()) {
        Ok(body) => Ok(body),
        Err(utf8_err) => Err(ErrorBadRequest(utf8_err)),
    }
}

pub fn get_lang(req: &HttpRequest) -> String {
    if let Some(lang) = req.headers().get("Accept-Language") {
        if let Ok(lang) = lang.to_str() {
            return String::from(lang);
        }
    }
    String::from("en")
}

pub fn get_absolute(for_user: &User, path: &str) -> String {
    fix_absolute(&for_user.home, path)
}

pub fn fix_absolute(home: &str, path: &str) -> String {
    if path.starts_with(".") {
        join_paths(home, path)
    } else {
        String::from(path)
    }
}

pub fn join_paths(path_a: &str, path_b: &str) -> String {
    let mut result = String::new();
    if starts_with_separator(path_a) {
        result.push(std::path::MAIN_SEPARATOR);
    }
    let mut parts = split_path(path_a);
    let parts_b = split_path(path_b);
    for part_b in parts_b {
        if part_b == "." {
            continue
        } else if part_b == ".." {
            parts.pop();
        } else {
            parts.push(part_b);
        }
    }
    let mut first = true;
    for part in parts {
        if first {
            first = false;
        } else {
            result.push(std::path::MAIN_SEPARATOR);
        }
        result.push_str(part);
    }
    if ends_with_separator(path_b) {
        result.push(std::path::MAIN_SEPARATOR);
    }
    result
}

pub fn split_path(path: &str) -> Vec<&str> {
    let mut result: Vec<&str> = Vec::new();
    let mut start = 0;
    for (i, c) in path.chars().enumerate() {
        if c == '\\' || c == '/'  {
            if i > start {
                result.push(&path[start..i]);
            }
            start = i + 1;
        }
    }
    if start < path.len() {
        result.push(&path[start..]);
    }
    result
}

pub fn starts_with_separator(path: &str) -> bool {
    path.starts_with("/") || path.starts_with("\\")
}

pub fn ends_with_separator(path: &str) -> bool {
    path.ends_with("/") || path.ends_with("\\")
}
