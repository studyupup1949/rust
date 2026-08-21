#![forbid(unsafe_code)]
#![warn(
	future_incompatible,
	nonstandard_style,
	rust_2018_idioms,
	rustdoc,
	unused
)]
#![warn(
	deprecated_in_future,
	missing_crate_level_docs,
	missing_debug_implementations,
	missing_docs,
	missing_doc_code_examples,
	// Disabled due to <https://github.com/rust-lang/rust/issues/69952> and
	// <https://github.com/rust-lang/rust/issues/77217>.
	// single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unused_crate_dependencies,
	unused_import_braces,
	// Disabled due to <https://github.com/rust-lang/rust/issues/77217>.
	// unused_lifetimes,
	unused_qualifications,
	unused_results
)]
#![warn(clippy::pedantic, clippy::cargo)]
// Disabled because we frequently mix u64 and usize, where the former is used for message body
// lengths and the latter for in-memory buffer sizes.
#![allow(clippy::cast_possible_truncation)]

//! Agnostic Asynchronous HTTP Client
//!
//! This crate is an HTTP/1.1 client library that does no dynamic memory allocation, is not tied to
//! any specific specific asynchronous executor or executors, and does not spawn additional
//! threads.
//!
//! The only requirement to use `aahc` is that the application provides a socket connected to the
//! HTTP server, which must implement [`AsyncBufRead`](futures_io::AsyncBufRead) and
//! [`AsyncWrite`](futures_io::AsyncWrite). This allows `aahc` to be used in any asynchronous
//! executor/reactor environment which provides TCP sockets.
//!
//! # Example
//! ```
//! let mut runtime = tokio::runtime::Builder::new()
//!		.basic_scheduler()
//!		.enable_io()
//!		.build()
//!		.unwrap();
//! runtime.block_on(async {
//!		use async_compat::CompatExt as _;
//!
//!		// Connect to the server.
//!		let mut socket = tokio::net::TcpStream::connect("example.com:80").await.unwrap();
//!
//!		// Split the socket into read and write halves and buffer the read half.
//!		let (mut read_socket, mut write_socket) = socket.split();
//!		let mut read_socket = tokio::io::BufReader::new(read_socket);
//!
//!		// Make the sockets into Futures-compatible objects.
//!		let mut read_socket = read_socket.compat();
//!		let mut write_socket = write_socket.compat();
//!
//!		// Send the request headers.
//!		let request_headers = [
//!			aahc::Header{name: "user-agent", value: b"aahc"},
//!			aahc::Header{name: "host", value: b"example.com"},
//!		];
//!		let mut send_body = aahc::send_headers(
//!			"GET",
//!			"/",
//!			&request_headers,
//!			std::pin::Pin::new(&mut write_socket),
//!		).await.unwrap();
//!
//!		// There is no body for a GET request, so finish it immediately.
//!		let metadata = send_body.finish().await.unwrap();
//!
//!		// Receive the response headers.
//!		let mut response_headers_buffer = [0u8; 1024];
//!		let mut response_headers_storage = [aahc::Header{name: "", value: b""}; 128];
//!		let (response_headers, mut receive_body) = aahc::receive_headers(
//!			std::pin::Pin::new(&mut read_socket),
//!			&mut response_headers_buffer,
//!			&mut response_headers_storage,
//!			metadata,
//!		).await.unwrap();
//!		assert_eq!(response_headers.minor_version, 1);
//!		assert_eq!(response_headers.status, 200);
//!
//!		// Receive the response body.
//!		let mut response_body = vec![];
//!		tokio::io::copy(
//!			&mut receive_body.compat(),
//!			&mut response_body,
//!		).await.unwrap();
//! })
//! ```

#[cfg(feature = "detailed-errors")]
pub mod error;
#[cfg(not(feature = "detailed-errors"))]
mod error;
mod request;
mod response;
mod util;

pub use httparse::Header;
pub use request::body::Send as SendBody;
pub use request::headers::send as send_headers;
pub use request::Metadata;
pub use response::body::Receive as ReceiveBody;
pub use response::headers::receive as receive_headers;
pub use response::headers::Response;
