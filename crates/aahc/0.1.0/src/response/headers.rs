use crate::error::InvalidData;
use crate::response::body::Receive;
use crate::Header;
use futures_io::AsyncBufRead;
use std::io::Result;
use std::pin::Pin;

/// An HTTP status line and response headers.
///
/// `'headers` is the lifetime of the array of `Header` instances as well as the byte buffer into
/// which they (and the `reason` field) point.
#[derive(Debug, PartialEq)]
pub struct Response<'headers> {
	/// The HTTP minor version number.
	///
	/// For example, for HTTP/1.0, this is zero; for HTTP/1.1, this is 1.
	pub minor_version: u8,

	/// The status code.
	pub status: u16,

	/// The reason phrase.
	pub reason: &'headers str,

	/// The response headers.
	pub headers: &'headers [Header<'headers>],
}

/// Receives an HTTP status line and response headers.
///
/// The `socket` parameter is the transport-layer socket over which the HTTP response will be
/// received. The `buffer` parameter is an application-provided buffer that will hold the response
/// headers. The `headers` parameter is an application-provided array that will be filled with the
/// decoded headers. The `metadata` parameter is the instance returned at the end of sending the
/// request body.
///
/// The `'socket` lifetime parameter is the lifetime of `socket`, which, once this function
/// returns, must continue to live in the returned [`Receive`](Receive). The `'headers` lifetime
/// parameter is the lifetime of `buffer` and `headers`, which need only live as long as the
/// application wishes to examine the response headers. The `Socket` type parameter is the type of
/// `socket`.
///
/// On success, the headers along with the next stage of the operation, reading the response body,
/// are returned.
///
/// # Errors
/// This function returns an error if reading from `socket` fails.
///
/// This function returns an error of kind [`InvalidData`](std::io::ErrorKind::InvalidData) under
/// the following conditions:
/// * if the response status line or headers are malformed
/// * if the response status line and headers are too large to fit in `buffer`
/// * if there are too many response headers to fit in `headers`
/// * if the server sent a status code 101 Switching Protocols
/// * if multiple `Content-Length` headers are present
/// * if the value of the `Content-Length` header is not a nonnegative integer
/// * if the value of the `Content-Length` header is too large to represent in a `u64`
/// * if the `Content-Length` and `Transfer-Encoding` headers are both present
/// * if the `Content-Length` header is present in a 204 No Content response
/// * if multiple `Transfer-Encoding` headers are present
/// * if the `Transfer-Encoding` header indicates an encoding other than chunked
/// * if the `Transfer-Encoding` header is present in a 204 No Content response
pub async fn receive<'socket, 'headers, Socket: AsyncBufRead + ?Sized>(
	mut socket: Pin<&'socket mut Socket>,
	buffer: &'headers mut [u8],
	headers: &'headers mut [Header<'headers>],
	metadata: crate::request::Metadata,
) -> Result<(Response<'headers>, Receive<'socket, Socket>)> {
	let mut buffer_used: usize = 0;
	loop {
		// Read data into the buffer until it ends with two consecutive [CR]LFs.
		//
		// It would be nice if we could just use httparse to try and parse a Response instance here
		// and break out of the loop when the attempt succeeds. Unfortunately, Response::parse()
		// borrows the buffer for the lifetime of the headers, leaving said buffer immutable. It’s
		// not enough just to drop the Response instance; the mere creation of the Response
		// instance, *ever*, binds the lifetime of the buffer (via Response::parse) to the lifetime
		// of the headers (via Response::new).
		//
		// So just do this hack instead.
		loop {
			let headers_done = {
				use crate::util::io::AsyncBufReadExt;
				socket
					.as_mut()
					.read_buf(|bytes: &[u8]| -> (usize, Result<bool>) {
						// If the bytes slice is empty, we have reached EOF.
						if bytes.is_empty() {
							return (0, Err(std::io::ErrorKind::UnexpectedEof.into()));
						}

						// Copy as many bytes as fit into the outer buffer. This might be more
						// than we need, but that’s fine.
						let bytes_copied_to_buffer = {
							let buffer_left = &mut buffer[buffer_used..];
							let bytes_to_copy = std::cmp::min(buffer_left.len(), bytes.len());
							let copy_target = &mut buffer_left[..bytes_to_copy];
							let copy_source = &bytes[..bytes_to_copy];
							copy_target.copy_from_slice(copy_source);
							bytes_to_copy
						};

						// See if the buffer contains a complete set of headers yet, including
						// the bytes just added.
						if let Some(n) =
							headers_length(&buffer[..buffer_used + bytes_copied_to_buffer])
						{
							// The first n bytes of self.buffer contain a complete set of headers.
							// However, we might have copied some of the body from the AsyncBufRead
							// into self.buffer as well, meaning self.buffer_used +
							// bytes_copied_to_buffer could be greater than n. What we will do is
							// update self.buffer_used to cover only the headers (not any possible
							// body parts copied in), and set a return value for read_buf that
							// likewise consumes only the headers, not any of the body, thus
							// leaving all the body in the AsyncBufWrite for later.
							let bytes_to_consume = n - buffer_used;
							buffer_used = n;
							(bytes_to_consume, Ok(true))
						} else {
							// All the bytes copied into the buffer so far don’t have a complete
							// set of headers yet. Keep all of the bytes we copied, consume them
							// from the AsyncBufRead, and indicate that we aren’t done yet to the
							// outer scope.
							buffer_used += bytes_copied_to_buffer;
							(bytes_copied_to_buffer, Ok(false))
						}
					})
					.await?
			}?;
			if headers_done {
				break;
			}
			if buffer_used == buffer.len() {
				// The buffer is full and we haven’t gotten a full set of headers yet. It’s not
				// happening!
				return Err(InvalidData::ResponseHeadersTooLong.into());
			}
		}

		// Get the status code.
		let status_code = parse_status_code(buffer)?;
		if status_code == 101 {
			// The server sent Switching Protocols, which we do not support.
			return Err(InvalidData::SwitchingProtocols.into());
		}
		if 100 <= status_code && status_code <= 199 {
			// This is an informational header. Discard it and wait for the real header.
			buffer_used = 0;
			continue;
		}

		// We have the real, non-100-series response. Split the buffer and proceed.
		break;
	}

	// Parse the headers.
	let mut resp = httparse::Response::new(headers);
	match resp
		.parse(&buffer[..buffer_used])
		.map_err(<InvalidData as From<httparse::Error>>::from)?
	{
		httparse::Status::Partial => {
			// We thought it was complete, but httparse thinks it isn’t. That should mean it’s
			// malformed. This is almost certainly due to incorrect newlines (we try our best to
			// work exactly the same way as httparse itself, but may not be perfect).
			return Err(Into::<InvalidData>::into(httparse::Error::NewLine).into());
		}
		httparse::Status::Complete(n) if n != buffer_used => {
			// httparse thought the headers ended earlier than we did. This is almost certainly due
			// to incorrect newlines (we try our best to work exactly the same way as httparse
			// itself, but may not be perfect).
			return Err(Into::<InvalidData>::into(httparse::Error::NewLine).into());
		}
		httparse::Status::Complete(_) => (),
	}
	let resp = Response {
		minor_version: resp.version.unwrap(),
		status: resp.code.unwrap(),
		reason: resp.reason.unwrap(),
		headers: resp.headers,
	};

	// See if we have content-length and/or transfer-encoding headers.
	let content_length = get_content_length(&resp)?;
	let chunked = is_chunked(&resp)?;

	// A server MUST NOT send a Content-Length header field in any message that contains a
	// Transfer-Encoding header field.
	if content_length.is_some() && chunked {
		return Err(InvalidData::ContentLengthAndTransferEncoding.into());
	}

	// A server MUST NOT send a Content-Length header field in any response with a status code
	// of 1xx (Informational) or 204 (No Content).
	if resp.status == 204 {
		if chunked {
			return Err(InvalidData::TransferEncodingWithNoContent.into());
		}
		if content_length.is_some() {
			return Err(InvalidData::ContentLengthWithNoContent.into());
		}
	}

	// See whether the connection can persist. This logic excludes the case of a response body
	// with indeterminate length because that is handled further down.
	let persistent = (resp.minor_version == 1)
		&& !metadata.connection_close
		&& !crate::util::is_connection_close(resp.headers);

	// Follow the rules to determine response body length.
	if metadata.head || resp.status == 204 || resp.status == 304 {
		// 1. Any response to a HEAD request and any response with a 1xx (Informational), 204
		// (No Content), or 304 (Not Modified) status code is always terminated by the first
		// empty line after the header fields, regardless of the header fields present in the
		// message, and thus cannot contain a message body.
		Ok((resp, Receive::new_fixed(socket, 0, persistent)))
	// Rule 2 is not applicable: CONNECT is not supported.
	} else if chunked {
		// 3. If a Transfer-Encoding header field is present and the chunked transfer coding
		// (Section 4.1) is the final encoding, the message body length is determined by
		// reading and decoding the chunked data until the transfer coding indicates the data
		// is complete.
		Ok((resp, Receive::new_chunked(socket, persistent)))
	// If a Transfer-Encoding header field is present in a response and the chunked transfer
	// coding is not the final encoding → not applicable, other transfer-encodings are not
	// supported.
	//
	// If a message is received with both a Transfer-Encoding and a Content-Length header field
	// → not applicable, this is rejected above.
	//
	// Rule 4 is handled in get_content_length().
	} else if let Some(n) = content_length {
		// 5. If a valid Content-Length header field is present without Transfer-Encoding, its
		// decimal value defines the expected message body length in octets. If the sender
		// closes the connection or the recipient times out before the indicated number of
		// octets are received, the recipient MUST consider the message to be incomplete and
		// close the connection.
		Ok((resp, Receive::new_fixed(socket, n, persistent)))
	// Rule 6 is not applicable: this is not a request message.
	} else {
		// 7. Otherwise, this is a response message without a declared message body length, so
		// the message body length is determined by the number of octets received prior to the
		// server closing the connection.
		Ok((resp, Receive::new_eof(socket)))
	}
}

/// Scans the headers and decode the `Content-Length` header, if any.
///
/// If the `Content-Length` header appears and contains a valid number, returns `Ok(Some(n))` where
/// `n` is that number. If there is no `Content-Length` header, returns `Ok(None)`.
///
/// # Errors
/// This function returns an error of kind [`InvalidData`](std::io::ErrorKind::InvalidData) under
/// the following conditions:
/// * if multiple `Content-Length` headers are present
/// * if the value of the `Content-Length` header is not a nonnegative integer
/// * if the value of the `Content-Length` header is too large to represent in a `u64`
fn get_content_length(resp: &Response<'_>) -> Result<Option<u64>> {
	let mut ret = None;
	for header in resp.headers.iter() {
		if header.name.eq_ignore_ascii_case("content-length") {
			use crate::error::BadContentLength;
			if ret.is_some() {
				return Err(InvalidData::MultipleContentLengths.into());
			}
			let value = std::str::from_utf8(header.value).map_err(BadContentLength::NotUtf8)?;
			let value = value.parse::<u64>().map_err(BadContentLength::NotU64)?;
			ret = Some(value);
		}
	}
	Ok(ret)
}

/// Scans the buffer and determine the length of the HTTP headers.
///
/// If all headers have been received, returns `Some(n)` where `n` is the length of the headers
/// in bytes, including the final [CR]LF[CR]LF. If not all headers have been received yet,
/// returns `None`.
fn headers_length(buffer: &[u8]) -> Option<usize> {
	// Search for [CR]LF[CR]LF. The position() function returns how many elements it consumed
	// in *that particular call* prior to the found element; thus, if called more than once on
	// the same iterator (i.e. to find subsequent occurrences of the item), it returns
	// incremental, not cumulative, distances.
	let mut start_pos = 0;
	let mut iter = buffer.iter();
	while let Some(dist) = iter.position(|&b| b == b'\n' || b == b'\r') {
		let eol_pos = start_pos + dist;
		for &candidate in &[
			&b"\r\n\r\n"[..],
			&b"\r\n\n"[..],
			&b"\n\r\n"[..],
			&b"\n\n"[..],
		] {
			if buffer.len() >= eol_pos + candidate.len()
				&& &buffer[eol_pos..eol_pos + candidate.len()] == candidate
			{
				return Some(eol_pos + candidate.len());
			}
		}
		// This one was not it. We have consumed eol_pos + 1 elements.
		start_pos = eol_pos + 1;
	}
	None
}

/// Scans the headers and determine whether the `transfer-encoding` header is present and indicates
/// that chunked encoding is in use for the response body.
///
/// If the `transfer-encoding` header appears and indicates chunked encoding, returns `true`. If
/// the `transfer-encoding` header does not appear, returns `false`.
///
/// # Errors
/// This function returns an error of kind [`InvalidData`](std::io::ErrorKind::InvalidData) under
/// the following conditions:
/// * if multiple `Transfer-Encoding` headers are present
/// * if the `Transfer-Encoding` header indicates an encoding other than chunked
fn is_chunked(resp: &Response<'_>) -> Result<bool> {
	let mut ret = false;
	for header in resp.headers.iter() {
		if header.name.eq_ignore_ascii_case("transfer-encoding") {
			if ret {
				return Err(InvalidData::MultipleTransferEncodings.into());
			}
			if header.value == b"chunked" {
				ret = true;
			} else {
				return Err(InvalidData::NotChunked.into());
			}
		}
	}
	Ok(ret)
}

/// Scans the buffer and extracts the HTTP status code.
///
/// The `buf` parameter is the HTTP response headers to examine.
///
/// # Errors
/// This function returns an error of kind [`InvalidData`](std::io::ErrorKind::InvalidData) if the
/// response status line or headers are malfoemd.
fn parse_status_code(buf: &[u8]) -> Result<u16> {
	// Use a local headers array of length zero. This will almost certainly result in
	// TooManyHeaders, but that’s fine because httparse will populate the code member before
	// returning such an error, and that’s all we need (and it avoids wasting time having httparse
	// parse the entire response headers only to throw them away). The use of the local array
	// avoids problems with tying up lifetimes of more important arrays provided by the caller.
	let mut headers = [];
	let mut resp = httparse::Response::new(&mut headers);
	match resp.parse(buf) {
		Ok(httparse::Status::Partial) => {
			// We thought it was complete, but httparse thinks it isn’t. That should mean it’s
			// malformed. This is almost certainly due to incorrect newlines (we try our best to
			// work exactly the same way as httparse itself, but may not be perfect).
			Err(Into::<InvalidData>::into(httparse::Error::NewLine).into())
		}
		Ok(httparse::Status::Complete(_)) | Err(httparse::Error::TooManyHeaders) => {
			// Ok(Complete(_)) is unusual (it should only happen if the response has no headers),
			// but acceptable.
			//
			// Err(TooManyHeaders) is the normal case.
			//
			// The unwrap here cannot panic because httparse does not return Complete or
			// TooManyHeaders prior to parsing the status code.
			Ok(resp.code.unwrap())
		}
		Err(e) => {
			// Something is wrong with the response.
			Err(Into::<InvalidData>::into(e).into())
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::request::Metadata;
	use futures_executor::block_on;
	use std::io::ErrorKind;

	fn expect_error<T: std::fmt::Debug>(kind: ErrorKind, x: &Result<T>) {
		match x {
			Err(e) if e.kind() == kind => (),
			_ => panic!("Expected error of kind {:?}, got {:?}", kind, x),
		}
	}

	fn expect_invalid_data_cb<T: std::fmt::Debug>(
		_cb: impl FnOnce(&InvalidData) -> bool, // unused if detailed-errors is off
		x: &Result<T>,
	) {
		expect_error(ErrorKind::InvalidData, x);
		#[cfg(feature = "detailed-errors")]
		{
			let source = x
				.as_ref()
				.unwrap_err()
				.get_ref()
				.expect("Expected error source, got None");
			let downcasted = source.downcast_ref::<InvalidData>();
			let downcasted = match downcasted {
				Some(x) => x,
				None => panic!(
					"Expected error source to be an InvalidData instance, got {:?}",
					source
				),
			};
			assert!(
				_cb(downcasted),
				"Expected error source to be something else, got {:?}",
				downcasted
			);
		}
	}

	fn expect_invalid_data<T: std::fmt::Debug>(variant: InvalidData, x: &Result<T>) {
		expect_invalid_data_cb(|v| *v == variant, x);
	}

	/// Tests the get_content_length function.
	#[test]
	fn test_get_content_length() {
		use crate::error::BadContentLength;

		// Basic headers including content-length.
		assert_eq!(
			get_content_length(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[Header {
					name: "content-length",
					value: b"1234"
				}]
			})
			.unwrap(),
			Some(1234)
		);

		// Basic headers without content-length.
		assert_eq!(
			get_content_length(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[Header {
					name: "something-else",
					value: b"1234"
				}]
			})
			.unwrap(),
			None
		);

		// Various kinds of invalid content length strings.
		for content_length in &[&b"-10"[..], &b"abcd"[..], &b"36893488147419103232"[..]] {
			// Invalid headers with negative content length.
			expect_invalid_data_cb(
				|v| {
					if let InvalidData::BadContentLength(BadContentLength::NotU64(_)) = v {
						true
					} else {
						false
					}
				},
				&get_content_length(&Response {
					minor_version: 1,
					status: 200,
					reason: "OK",
					headers: &[Header {
						name: "content-length",
						value: content_length,
					}],
				}),
			);
		}
		expect_invalid_data_cb(
			|v| {
				if let InvalidData::BadContentLength(BadContentLength::NotUtf8(_)) = v {
					true
				} else {
					false
				}
			},
			&get_content_length(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[Header {
					name: "content-length",
					value: &b"\xFFabcd"[..],
				}],
			}),
		);

		// Invalid: multiple content-length headers.
		expect_invalid_data(
			InvalidData::MultipleContentLengths,
			&get_content_length(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[
					Header {
						name: "content-length",
						value: b"1234",
					},
					Header {
						name: "content-length",
						value: b"1235",
					},
				],
			}),
		);
	}

	/// Tests the header_length function.
	#[test]
	fn test_headers_length() {
		// Basic complete set of headers ending with CR LF CR LF.
		assert_eq!(headers_length(b"H:V\r\nH:V\r\n\r\n"), Some(12));

		// Complete sets of headers ending with various non-CRLF line terminators, including a mix
		// thereof.
		assert_eq!(headers_length(b"H:V\nH:V\n\n"), Some(9));
		assert_eq!(headers_length(b"H:V\r\nH:V\n\n"), Some(10));
		assert_eq!(headers_length(b"H:V\nH:V\r\n\n"), Some(10));
		assert_eq!(headers_length(b"H:V\nH:V\n\r\n"), Some(10));
		assert_eq!(headers_length(b"H:V\nH:V\r\n\r\n"), Some(11));

		// Basic incomplete headers.
		assert_eq!(headers_length(b"H:V\nH:V\n"), None);
		assert_eq!(headers_length(b"H:V\nH:V"), None);
		assert_eq!(headers_length(b""), None);

		// Headers that are incomplete because they end with a CR but without its required matching
		// LF.
		assert_eq!(headers_length(b"H:V\nH:V\n\r"), None);
	}

	/// Tests the is_chunked function.
	#[test]
	fn test_is_chunked() {
		// Basic headers including transfer-encoding: chunked.
		assert_eq!(
			is_chunked(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[Header {
					name: "transfer-encoding",
					value: b"chunked"
				}]
			})
			.unwrap(),
			true
		);

		// Basic headers without transfer-encoding.
		assert_eq!(
			is_chunked(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[Header {
					name: "something-else",
					value: b"1234"
				}]
			})
			.unwrap(),
			false
		);

		// An unsupported transfer-encoding.
		expect_invalid_data(
			InvalidData::NotChunked,
			&is_chunked(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[Header {
					name: "transfer-encoding",
					value: b"gzip",
				}],
			}),
		);

		// Invalid: multiple transfer-encoding headers.
		expect_invalid_data(
			InvalidData::MultipleTransferEncodings,
			&is_chunked(&Response {
				minor_version: 1,
				status: 200,
				reason: "OK",
				headers: &[
					Header {
						name: "transfer-encoding",
						value: b"chunked",
					},
					Header {
						name: "transfer-encoding",
						value: b"chunked",
					},
				],
			}),
		);
	}

	/// Tests the parse_status_code function.
	#[test]
	fn test_parse_status_code() {
		// Basic headers.
		assert_eq!(
			parse_status_code(b"HTTP/1.1 200 OK\r\nH: V\r\n\r\n").unwrap(),
			200
		);

		// No headers.
		assert_eq!(parse_status_code(b"HTTP/1.1 200 OK\r\n\r\n").unwrap(), 200);

		// Invalid: not HTTP.
		expect_invalid_data(
			InvalidData::ParseHeaders(httparse::Error::Version),
			&parse_status_code(b"ABCD/1.1 200 OK\r\n\r\n"),
		);
	}

	/// Tests receiving a full set of headers.
	#[test]
	fn test_receive_basic() {
		block_on(async {
			let mut data = &b"HTTP/1.1 200 OK\r\nH1: V1\r\nH2: V2\r\n\r\n"[..];
			let mut buffer = [0u8; 256];
			let mut headers = [httparse::EMPTY_HEADER; 2];
			let metadata = Metadata {
				head: false,
				connection_close: false,
			};
			let (resp, _) = receive(Pin::new(&mut data), &mut buffer, &mut headers, metadata)
				.await
				.unwrap();
			assert_eq!(resp.minor_version, 1);
			assert_eq!(resp.status, 200);
			assert_eq!(resp.reason, "OK");
			assert_eq!(resp.headers.len(), 2);
			assert_eq!(resp.headers[0].name, "H1");
			assert_eq!(resp.headers[0].value, b"V1");
			assert_eq!(resp.headers[1].name, "H2");
			assert_eq!(resp.headers[1].value, b"V2");
		});
	}

	/// Tests receiving a 100 Continue status followed by a proper response.
	#[test]
	fn test_receive_100_continue() {
		block_on(async {
			let mut data = &b"HTTP/1.1 100 Continue\r\nH0: V0\r\n\r\nHTTP/1.1 200 OK\r\nH1: V1\r\nH2: V2\r\n\r\n"[..];
			let mut buffer = [0u8; 256];
			let mut headers = [httparse::EMPTY_HEADER; 2];
			let metadata = Metadata {
				head: false,
				connection_close: false,
			};
			let (resp, _) = receive(Pin::new(&mut data), &mut buffer, &mut headers, metadata)
				.await
				.unwrap();
			assert_eq!(resp.minor_version, 1);
			assert_eq!(resp.status, 200);
			assert_eq!(resp.reason, "OK");
			assert_eq!(resp.headers.len(), 2);
			assert_eq!(resp.headers[0].name, "H1");
			assert_eq!(resp.headers[0].value, b"V1");
			assert_eq!(resp.headers[1].name, "H2");
			assert_eq!(resp.headers[1].value, b"V2");
		});
	}

	/// Tests a truncated response.
	#[test]
	fn test_receive_truncated() {
		block_on(async {
			let mut data = &b"HTTP/1.1 200 OK\r\nH1: V1\r\nH2: V2\r\n"[..];
			let mut buffer = [0u8; 256];
			let mut headers = [httparse::EMPTY_HEADER; 2];
			let metadata = Metadata {
				head: false,
				connection_close: false,
			};
			expect_error(
				ErrorKind::UnexpectedEof,
				&receive(Pin::new(&mut data), &mut buffer, &mut headers, metadata).await,
			);
		});
	}

	/// Tests response headers being too long to fit in the application-provided buffer.
	#[test]
	fn test_receive_too_long() {
		block_on(async {
			let mut data = &b"HTTP/1.1 200 OK\r\nH1: V1\r\nH2: V2\r\n\r\n"[..];
			let mut buffer = [0u8; 34];
			let mut headers = [httparse::EMPTY_HEADER; 2];
			let metadata = Metadata {
				head: false,
				connection_close: false,
			};
			expect_invalid_data(
				InvalidData::ResponseHeadersTooLong,
				&receive(Pin::new(&mut data), &mut buffer, &mut headers, metadata).await,
			);
		});
	}

	/// Tests receiving a Switching Protocols status.
	#[test]
	fn test_receive_switching_protocols() {
		block_on(async {
			let mut data = &b"HTTP/1.1 101 Switching Protocols\r\nH1: V1\r\nH2: V2\r\n\r\n"[..];
			let mut buffer = [0u8; 256];
			let mut headers = [httparse::EMPTY_HEADER; 2];
			let metadata = Metadata {
				head: false,
				connection_close: false,
			};
			expect_invalid_data(
				InvalidData::SwitchingProtocols,
				&receive(Pin::new(&mut data), &mut buffer, &mut headers, metadata).await,
			);
		});
	}

	/// Tests receiving an invalid `Content-Length` header.
	#[test]
	fn test_receive_bad_length() {
		block_on(async {
			let mut data = &b"HTTP/1.1 200 OK\r\nContent-Length: -1\r\n\r\n"[..];
			let mut buffer = [0u8; 256];
			let mut headers = [httparse::EMPTY_HEADER; 2];
			let metadata = Metadata {
				head: false,
				connection_close: false,
			};
			expect_invalid_data_cb(
				|v| {
					use crate::error::BadContentLength;
					if let InvalidData::BadContentLength(BadContentLength::NotU64(_)) = v {
						true
					} else {
						false
					}
				},
				&receive(Pin::new(&mut data), &mut buffer, &mut headers, metadata).await,
			);
		});
	}

	/// Tests receiving an unacceptable `Transfer-Encoding` header.
	#[test]
	fn test_receive_bad_transfer_encoding() {
		block_on(async {
			let mut data = &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n"[..];
			let mut buffer = [0u8; 256];
			let mut headers = [httparse::EMPTY_HEADER; 2];
			let metadata = Metadata {
				head: false,
				connection_close: false,
			};
			expect_invalid_data(
				InvalidData::NotChunked,
				&receive(Pin::new(&mut data), &mut buffer, &mut headers, metadata).await,
			);
		});
	}
}
