use crate::request::body::Send;
use std::pin::Pin;

/// Sends an HTTP request line and request headers.
///
/// The `method` parameter holds the method for the request (e.g. `GET`, `POST`, or `DELETE`). The
/// `request_target` parameter holds the “request target”, which is typically the path and query
/// string parts of the URL, though it may include the scheme, host, and port parts if a proxy
/// request is being made. The `headers` parameter holds the request headers to send. The `socket`
/// parameter is the transport-layer socket over which the HTTP request will be sent, which must
/// already be connected to the remote host; it is recommended that the socket provide write
/// buffering (e.g. that the raw transport socket be wrapped in a `BufWriter`) for good
/// performance.
///
/// The `'socket` lifetime parameter is the lifetime of `socket`, which, once this function
/// returns, must continue to live in the returned [`Send`](Send) instance. The `Socket` type
/// parameter is the type of `socket`.
///
/// This function returns once the headers have been written to `socket`. The return value is a
/// [`Send`](Send) instance that can be used to write the request body, if any.
///
/// *Important*: This function does not flush the socket. Normally the application will proceed to
/// sending the request body, so no flush is necessary; however, if the socket is a buffered
/// wrapper around an underlying socket and the application intends to unwrap the wrapper (either
/// temporarily, creating a new wrapper while sending the request body, or permanently), it must
/// flush the socket before unwrapping, otherwise data may be lost.
///
/// # Errors
/// This function returns an error if writing to `socket` fails.
///
/// # Panics
/// This function panics under any of the following conditions, in a debug build:
/// * if the request method is `CONNECT` (this method is not supported) or not a valid token
/// * if the path is not a valid path (only the absence of characters outside the range 0x21 to
///   0x7F is checked, not the full `request-target` production)
/// * if the `Transfer-Encoding` header is present and is set to any value other than `chunked`
///   (other transfer encodings are not supported)
/// * if the `Content-Length` header is present and is set to something other than a non-negative
///   64-bit integer
/// * if more than one of `Transfer-Encoding` and `Content-Length` headers is present (either both
///   headers or multiple instances of the same header)
/// * if the `Upgrade` header is present (protocol switching is not supported)
/// * if the `TE` header is present (neither transfer encodings other than `chunked` nor trailers
///   are supported)
/// * if any header name is not a valid token
/// * if any header value is invalid (a valid header value is one that does not contain any bytes
///   0x00 through 0x08 or 0x0A through 0x1F, and which does not start or end with a space or tab)
///
/// The word “token” refers to the `token` production in the HTTP RFC, namely a string that
/// comprises only digits, letters, and the characters ```!#$%&'*+-.^_`|~```, and is at least one
/// character long.
///
/// These are debug-build panics, not errors, because it is expected that the application will not
/// construct invalid or unsupported requests; thus, such requests represent bugs in the
/// application.
pub async fn send<'socket, Socket: futures_io::AsyncWrite + ?Sized>(
	method: &str,
	request_target: &str,
	headers: &[crate::Header<'_>],
	mut socket: Pin<&'socket mut Socket>,
) -> std::io::Result<Send<'socket, Socket>> {
	use crate::util::io::AsyncWriteExt as _;
	use crate::util::is_token;

	// Sanity check method.
	debug_assert!(is_token(method), "Request method {} is not a token", method);
	debug_assert!(
		method != "CONNECT",
		"Request method CONNECT is not supported"
	);

	// Sanity check request_target.
	debug_assert!(
		crate::util::is_request_target(request_target),
		"Request target contains invalid characters"
	);

	// Sanity check headers.
	for header in headers.iter() {
		// Check header name/value for validity.
		debug_assert!(
			is_token(header.name),
			"Request header {} is not a token",
			header.name
		);
		debug_assert!(
			crate::util::is_field_value(header.value),
			"Request header value {:?} is not a valid field value",
			header.value
		);

		// Check for specific headers that need special handling.
		debug_assert!(
			!header.name.eq_ignore_ascii_case("TE"),
			"Request header TE is not supported"
		);
		if header.name.eq_ignore_ascii_case("Transfer-Encoding") {
			debug_assert!(
				header.value == b"chunked",
				"Request Transfer-Encoding is {:?}, but only chunked is supported",
				header.value
			);
		} else if header.name.eq_ignore_ascii_case("Content-Length") {
			debug_assert!(
				std::str::from_utf8(header.value)
					.ok()
					.and_then(|v| u64::from_str_radix(v, 10).ok())
					.is_some(),
				"Request Content-Length {:?} is not a non-negative integer",
				header.value
			);
		}
	}
	debug_assert!(
		headers
			.iter()
			.filter(|h| h.name.eq_ignore_ascii_case("content-length")
				|| h.name.eq_ignore_ascii_case("transfer-encoding"))
			.count() <= 1,
		"Request must contain at most one of Content-Length and Transfer-Encoding"
	);

	// Send the request line.
	socket.as_mut().write_all(method.as_bytes()).await?;
	socket.as_mut().write_all(b" ").await?;
	socket.as_mut().write_all(request_target.as_bytes()).await?;
	socket.as_mut().write_all(b" HTTP/1.1\r\n").await?;

	// Send the headers.
	for &header in headers.iter() {
		socket.as_mut().write_all(header.name.as_bytes()).await?;
		socket.as_mut().write_all(b":").await?;
		socket.as_mut().write_all(header.value).await?;
		socket.as_mut().write_all(b"\r\n").await?;
	}

	// Send the blank line.
	socket.as_mut().write_all(b"\r\n").await?;

	// Done!
	let metadata = super::Metadata {
		head: method == "HEAD",
		connection_close: crate::util::is_connection_close(headers),
	};
	if headers
		.iter()
		.any(|h| h.name.eq_ignore_ascii_case("Transfer-Encoding"))
	{
		Ok(Send::new_chunked(socket, metadata))
	} else {
		let h = headers
			.iter()
			.find(|h| h.name.eq_ignore_ascii_case("Content-Length"));
		let length = h
			.and_then(|h| std::str::from_utf8(h.value).ok())
			.and_then(|v| u64::from_str_radix(v, 10).ok())
			.unwrap_or(0);
		Ok(Send::new_fixed(socket, metadata, length))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::Header;
	use futures_executor::block_on;

	/// Tests sending some basic headers, with neither Content-Length nor Transfer-Encoding.
	#[test]
	fn test_basic() {
		let headers = [
			Header {
				name: "User-Agent",
				value: b"Thingy/1.0 OtherThing/2.0",
			},
			Header {
				name: "Host",
				value: b"someplace.example.com",
			},
		];
		let mut sink: Vec<u8> = Vec::new();
		let _ = block_on(send("POST", "/abcd/efgh", &headers, Pin::new(&mut sink))).unwrap();
		assert_eq!(sink, &b"POST /abcd/efgh HTTP/1.1\r\nUser-Agent:Thingy/1.0 OtherThing/2.0\r\nHost:someplace.example.com\r\n\r\n"[..]);
	}

	/// Tests sending some basic request headers, including Content-Length.
	#[test]
	fn test_content_length() {
		let headers = [
			Header {
				name: "User-Agent",
				value: b"Thingy/1.0 OtherThing/2.0",
			},
			Header {
				name: "Content-Length",
				value: b"1234",
			},
		];
		let mut sink: Vec<u8> = Vec::new();
		let _ = block_on(send("POST", "/abcd/efgh", &headers, Pin::new(&mut sink))).unwrap();
		assert_eq!(sink, &b"POST /abcd/efgh HTTP/1.1\r\nUser-Agent:Thingy/1.0 OtherThing/2.0\r\nContent-Length:1234\r\n\r\n"[..]);
	}

	/// Tests sending some basic request headers, including Transfer-Encoding.
	#[test]
	fn test_transfer_encoding() {
		let headers = [
			Header {
				name: "User-Agent",
				value: b"Thingy/1.0 OtherThing/2.0",
			},
			Header {
				name: "Transfer-Encoding",
				value: b"chunked",
			},
		];
		let mut sink: Vec<u8> = Vec::new();
		let _ = block_on(send("POST", "/abcd/efgh", &headers, Pin::new(&mut sink))).unwrap();
		assert_eq!(sink, &b"POST /abcd/efgh HTTP/1.1\r\nUser-Agent:Thingy/1.0 OtherThing/2.0\r\nTransfer-Encoding:chunked\r\n\r\n"[..]);
	}
}
