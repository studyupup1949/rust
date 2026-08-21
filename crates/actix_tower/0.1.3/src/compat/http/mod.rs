//! `http` crate compatibility helpers.
//!
//! Utilities for converting between Actix's HTTP types and the `http` crate types.

use actix_web::http::{Method, StatusCode, Uri, Version};
use http as http_crate;

/// Convert an Actix `Method` to an `http::Method`.
pub fn method_to_http(m: Method) -> http_crate::Method {
    http_crate::Method::from_bytes(m.as_str().as_bytes()).unwrap_or(http_crate::Method::GET)
}

/// Convert an `http::Method` to an Actix `Method`.
pub fn method_from_http(m: &http_crate::Method) -> Method {
    Method::from_bytes(m.as_str().as_bytes()).unwrap_or(Method::GET)
}

/// Convert an Actix `Uri` to an `http::Uri`.
pub fn uri_to_http(u: &Uri) -> http_crate::Uri {
    u.to_string()
        .parse()
        .unwrap_or(http_crate::Uri::from_static("/"))
}

/// Convert an `http::Uri` to an Actix `Uri`.
pub fn uri_from_http(u: &http_crate::Uri) -> Uri {
    u.to_string().parse().unwrap_or_default()
}

/// Convert an Actix `StatusCode` to an `http::StatusCode`.
pub fn status_to_http(s: StatusCode) -> http_crate::StatusCode {
    http_crate::StatusCode::from_u16(s.as_u16()).unwrap_or(http_crate::StatusCode::OK)
}

/// Convert an `http::StatusCode` to an Actix `StatusCode`.
pub fn status_from_http(s: http_crate::StatusCode) -> StatusCode {
    StatusCode::from_u16(s.as_u16()).unwrap_or(StatusCode::OK)
}

/// Convert an Actix `Version` to an `http::Version`.
pub fn version_to_http(v: Version) -> http_crate::Version {
    match v {
        Version::HTTP_09 => http_crate::Version::HTTP_09,
        Version::HTTP_10 => http_crate::Version::HTTP_10,
        Version::HTTP_11 => http_crate::Version::HTTP_11,
        Version::HTTP_2 => http_crate::Version::HTTP_2,
        Version::HTTP_3 => http_crate::Version::HTTP_3,
        _ => http_crate::Version::HTTP_11,
    }
}

/// Convert an `http::Version` to an Actix `Version`.
pub fn version_from_http(v: http_crate::Version) -> Version {
    match v {
        http_crate::Version::HTTP_09 => Version::HTTP_09,
        http_crate::Version::HTTP_10 => Version::HTTP_10,
        http_crate::Version::HTTP_11 => Version::HTTP_11,
        http_crate::Version::HTTP_2 => Version::HTTP_2,
        http_crate::Version::HTTP_3 => Version::HTTP_3,
        _ => Version::HTTP_11,
    }
}
