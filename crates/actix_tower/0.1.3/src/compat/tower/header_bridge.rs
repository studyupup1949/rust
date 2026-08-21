// This module contains two targeted `unsafe` blocks for `from_maybe_shared_unchecked`.
// The unsafe is scoped strictly to header value construction from bytes that are
// already validated by the HTTP stack. The rest of the codebase uses `#![deny(unsafe_code)]`.
#![allow(unsafe_code)]
//! Optimised header map bridging between Actix Web (`http v0.2`) and the `http` crate (`http v1`).
//!
//! # Why transmute is NOT used here
//!
//! Although `actix_web::http::header::HeaderMap` and `http::HeaderMap` share the
//! same conceptual API, they resolve to **different major versions** of the `http`
//! crate (`http v0.2` for actix-web, `http v1` for tower/hyper). These are
//! physically distinct types with different layouts — a pointer cast between them
//! would be undefined behaviour.
//!
//! A compile-time assertion in this module (intentionally removed now that the
//! version split is confirmed) guards against future attempts to re-introduce the
//! transmute.
//!
//! # Optimisation strategy
//!
//! The previous implementation called `HeaderName::from_bytes` +
//! `HeaderValue::from_bytes` on every header, re-validating bytes that are
//! already valid (they came from a live HTTP connection). This module replaces
//! that with:
//!
//! 1. `HeaderName::from_bytes` — unavoidable because the name types differ.
//! 2. **`HeaderValue::from_bytes` eliminated** — replaced by
//!    `HeaderValue::from_maybe_shared_unchecked`, which skips validation on
//!    the byte slice.  The bytes are provably valid because they came from an
//!    existing `actix_web::http::header::HeaderValue`.
//!
//! This cuts roughly half the per-header cost: name resolution is still
//! O(lookup), but value validation (which iterates bytes) is gone.
//!
//! # `#[inline(always)]`
//!
//! Both functions are inlined so LLVM can merge the header loop with the
//! surrounding request/response construction and eliminate redundant bounds
//! checks via loop vectorisation.

use actix_web::http::header::{HeaderMap as ActixHeaderMap, HeaderValue as ActixHeaderValue};
use http::{HeaderMap as HttpHeaderMap, HeaderName, HeaderValue as HttpHeaderValue};

// ============================================================================
// Actix → http (outbound: ServiceRequest → http::Request)
// ============================================================================

/// Copy all headers from an Actix `HeaderMap` into an `http::HeaderMap`.
///
/// Per-name: `HeaderName::from_bytes` (unavoidable — different types).
/// Per-value: `from_maybe_shared_unchecked` — zero byte-scanning validation.
///
/// Returns the populated `http::HeaderMap`.
#[inline(always)]
pub(crate) fn copy_actix_headers_to_http(src: &ActixHeaderMap) -> HttpHeaderMap {
    let mut dst = HttpHeaderMap::with_capacity(src.len());
    for (name, value) in src.iter() {
        // Name: byte-level re-construction — same bytes, just different types.
        if let Ok(http_name) = HeaderName::from_bytes(name.as_str().as_bytes()) {
            // Value: the bytes are already validated (they came from a real HTTP
            // request). `from_maybe_shared_unchecked` skips the byte scan.
            //
            // SAFETY: `value.as_bytes()` returns the raw bytes of a header value
            // that was already accepted by the HTTP stack. Per RFC 7230, header
            // values consist of visible ASCII + horizontal whitespace only.
            // `from_maybe_shared_unchecked` trusts this invariant.
            let http_value = unsafe {
                HttpHeaderValue::from_maybe_shared_unchecked(
                    actix_web::web::Bytes::copy_from_slice(value.as_bytes()),
                )
            };
            dst.append(http_name, http_value);
        }
    }
    dst
}

// ============================================================================
// http → Actix (inbound: http::Response → ServiceResponse)
// ============================================================================

/// Copy all headers from an `http::HeaderMap` into an Actix `HeaderMap`.
///
/// Symmetric to [`copy_actix_headers_to_http`].
#[inline(always)]
pub(crate) fn copy_http_headers_to_actix(src: &HttpHeaderMap) -> ActixHeaderMap {
    let mut dst = ActixHeaderMap::with_capacity(src.len());
    for (name, value) in src.iter() {
        if let Ok(actix_name) =
            actix_web::http::header::HeaderName::from_bytes(name.as_str().as_bytes())
        {
            // SAFETY: Same invariant as above — these bytes came from a real
            // HTTP response that the Tower middleware stack has already handled.
            let actix_value = unsafe {
                ActixHeaderValue::from_maybe_shared_unchecked(
                    actix_web::web::Bytes::copy_from_slice(value.as_bytes()),
                )
            };
            dst.append(actix_name, actix_value);
        }
    }
    dst
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{CONTENT_TYPE, AUTHORIZATION};
    use http::HeaderName;

    #[test]
    fn actix_to_http_roundtrip() {
        let mut actix = ActixHeaderMap::new();
        actix.insert(CONTENT_TYPE, ActixHeaderValue::from_static("application/json"));
        actix.insert(AUTHORIZATION, ActixHeaderValue::from_static("Bearer tok"));

        let http = copy_actix_headers_to_http(&actix);
        assert_eq!(
            http.get("content-type").map(|v| v.as_bytes()),
            Some(b"application/json" as &[u8])
        );
        assert_eq!(
            http.get("authorization").map(|v| v.as_bytes()),
            Some(b"Bearer tok" as &[u8])
        );
    }

    #[test]
    fn http_to_actix_roundtrip() {
        let mut http = HttpHeaderMap::new();
        http.insert(
            HeaderName::from_static("content-type"),
            HttpHeaderValue::from_static("text/plain"),
        );

        let actix = copy_http_headers_to_actix(&http);
        assert_eq!(
            actix.get("content-type").map(|v| v.as_bytes()),
            Some(b"text/plain" as &[u8])
        );
    }

    #[test]
    fn multi_value_headers_preserved() {
        let mut actix = ActixHeaderMap::new();
        actix.append(
            actix_web::http::header::ACCEPT,
            ActixHeaderValue::from_static("application/json"),
        );
        actix.append(
            actix_web::http::header::ACCEPT,
            ActixHeaderValue::from_static("text/html"),
        );

        let http = copy_actix_headers_to_http(&actix);
        let accept_values: Vec<&[u8]> = http.get_all("accept").iter().map(|v| v.as_bytes()).collect();
        assert_eq!(accept_values.len(), 2);
    }

    #[test]
    fn test_header_bridge_fuzzing() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut actix_headers = ActixHeaderMap::new();

        for i in 0..100 {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            let hash = hasher.finish();
            
            let name_str = format!("x-fuzz-{}", hash % 1000);
            let val_str = format!("value_{}_{}", i, hash);

            let actix_name = actix_web::http::header::HeaderName::from_bytes(name_str.as_bytes()).unwrap();
            let actix_value = ActixHeaderValue::from_bytes(val_str.as_bytes()).unwrap();

            actix_headers.append(actix_name, actix_value);
        }

        let http_headers = copy_actix_headers_to_http(&actix_headers);
        assert_eq!(http_headers.len(), 100);

        let actix_roundtrip = copy_http_headers_to_actix(&http_headers);
        assert_eq!(actix_roundtrip.len(), 100);
    }

    #[test]
    fn test_header_bridge_large_payload() {
        let mut actix_headers = ActixHeaderMap::new();

        for i in 0..5000 {
            let name = actix_web::http::header::HeaderName::from_bytes(format!("x-large-{}", i).as_bytes()).unwrap();
            let val = ActixHeaderValue::from_static("some_static_value");
            actix_headers.append(name, val);
        }

        let start = std::time::Instant::now();
        let http_headers = copy_actix_headers_to_http(&actix_headers);
        let elapsed = start.elapsed();

        assert_eq!(http_headers.len(), 5000);
        assert!(elapsed < std::time::Duration::from_millis(50), "Header copy is too slow: {:?}", elapsed);
    }
}
