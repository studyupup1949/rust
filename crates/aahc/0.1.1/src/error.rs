//! Errors that originate inside `aahc` or `httparse`.
//!
//! `aahc` API functions report three kinds of errors: errors passed up from the underlying socket,
//! errors caused by the server sending an invalid or unsupported HTTP response, and errors caused
//! by the server closing its socket prematurely (before the entire response is transferred).
//! `aahc` API functions always report errors in the form of [`std::io::Error`]. Errors passed up
//! from the underlying socket pass through `aahc` completely unmodified. Errors caused by the
//! server closing its socket prematurely are reported as [`std::io::ErrorKind::UnexpectedEof`].
//! Errors caused by the server sending an invalid or unsupported HTTP response are reported as
//! [`std::io::ErrorKind::InvalidData`]; if the `detailed-errors` feature is enabled, then the
//! inner error of the [`std::io::Error`] is an [`InvalidData`] instance, otherwise there is no
//! source and this module is not exported.

use std::fmt::{Display, Formatter};

/// The ways in which a received `Content-Type` header can be invalid.
#[derive(Debug, PartialEq)]
pub enum BadContentLength {
	/// The header is not valid UTF-8.
	NotUtf8(std::str::Utf8Error),

	/// The header is not a nonnegative integer or does not fit into a `u64`.
	NotU64(<u64 as std::str::FromStr>::Err),
}

impl Display for BadContentLength {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::NotUtf8(inner) => inner.fmt(f),
			Self::NotU64(inner) => inner.fmt(f),
		}
	}
}

impl std::error::Error for BadContentLength {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::NotUtf8(inner) => Some(inner),
			Self::NotU64(inner) => Some(inner),
		}
	}
}

/// The ways in which a chunk header can be invalid.
#[derive(Debug, PartialEq)]
pub enum BadChunkHeader {
	/// A byte in the chunk size is not a hex digit.
	SizeNotHex,

	/// The size does not fit in a `u64`.
	SizeNotU64,

	/// A character in the chunk header extensions section was not permitted to appear there.
	ExtChar,

	/// A newline character (CR or LF) was not present where required, either after the chunk
	/// header, after a chunk’s data, or after the blank line following the end marker.
	Newline,
}

impl Display for BadChunkHeader {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::SizeNotHex => write!(f, "Chunk size is not a hex number"),
			Self::SizeNotU64 => write!(f, "Chunk size is too large"),
			Self::ExtChar => write!(f, "Chunk extensions contains invalid character"),
			Self::Newline => write!(f, "Chunk framing contains incorrect newlines"),
		}
	}
}

impl std::error::Error for BadChunkHeader {}

/// The type of nested error included in any error of kind
/// [`InvalidData`](std::io::ErrorKind::InvalidData) that originates within this `aahc` itself.
///
/// Errors that pass through `aahc` but do not originate there—such as errors returned by the
/// underlying socket—may be of kind [`InvalidData`](std::io::ErrorKind::InvalidData) but not
/// contain a nested error object of this type.
///
/// The nested error is included only if the `detailed-errors` feature is enabled, because it
/// requires heap allocation. If the `detailed-errors` feature is disabled, this type does not
/// exist.
#[derive(Debug, PartialEq)]
pub enum InvalidData {
	/// An error occurred during parsing headers.
	ParseHeaders(httparse::Error),

	/// The response headers are too long.
	ResponseHeadersTooLong,

	/// The server decided to switch protocols. This is not supported.
	SwitchingProtocols,

	/// The server sent both a `Content-Length` header and a `Transfer-Encoding` header.
	ContentLengthAndTransferEncoding,

	/// The server sent a `Content-Length` header with a 204 No Content status code.
	ContentLengthWithNoContent,

	/// The server sent multiple `Content-Length` headers.
	MultipleContentLengths,

	/// The server sent an invalid `Content-Length` header.
	BadContentLength(BadContentLength),

	/// The server sent a `Transfer-Encoding` header with a 204 No Content status code.
	TransferEncodingWithNoContent,

	/// The server sent multiple `Transfer-Encoding` headers.
	MultipleTransferEncodings,

	/// The server sent a `Transfer-Encoding` header with an encoding other than `chunked`.
	NotChunked,

	/// The server sent an invalid chunk header.
	BadChunkHeader(BadChunkHeader),
}

impl Display for InvalidData {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::ParseHeaders(inner) => inner.fmt(f),
			Self::ResponseHeadersTooLong => write!(f, "Response headers too long"),
			Self::SwitchingProtocols => write!(f, "Unsupported 101 Switching Protocols received"),
			Self::ContentLengthAndTransferEncoding => {
				write!(f, "Content-Length and Transfer-Encoding both received")
			}
			Self::ContentLengthWithNoContent => {
				write!(f, "Content-Length received in 204 No Content response")
			}
			Self::MultipleContentLengths => write!(f, "Multiple Content-Length headers received"),
			Self::BadContentLength(inner) => {
				write!(f, "Invalid Content-Length header received: {inner}")
			}
			Self::TransferEncodingWithNoContent => {
				write!(f, "Transfer-Encoding received in 204 No Content response")
			}
			Self::MultipleTransferEncodings => {
				write!(f, "Multiple Transfer-Encoding headers received")
			}
			Self::NotChunked => write!(f, "Unsupported Transfer-Encoding received"),
			Self::BadChunkHeader(inner) => write!(f, "Invalid chunk header received: {inner}"),
		}
	}
}

impl std::error::Error for InvalidData {
	fn source<'a>(&'a self) -> Option<&'a (dyn std::error::Error + 'static)> {
		match self {
			Self::ParseHeaders(inner) => Some(inner),
			Self::BadContentLength(inner) => Some(inner),
			Self::BadChunkHeader(inner) => Some(inner),
			Self::ResponseHeadersTooLong
			| Self::SwitchingProtocols
			| Self::ContentLengthAndTransferEncoding
			| Self::ContentLengthWithNoContent
			| Self::MultipleContentLengths
			| Self::TransferEncodingWithNoContent
			| Self::MultipleTransferEncodings
			| Self::NotChunked => None,
		}
	}
}

impl From<httparse::Error> for InvalidData {
	fn from(inner: httparse::Error) -> Self {
		Self::ParseHeaders(inner)
	}
}

impl From<BadContentLength> for InvalidData {
	fn from(inner: BadContentLength) -> Self {
		Self::BadContentLength(inner)
	}
}

impl From<BadChunkHeader> for InvalidData {
	fn from(inner: BadChunkHeader) -> Self {
		Self::BadChunkHeader(inner)
	}
}

impl From<InvalidData> for std::io::Error {
	#[cfg(feature = "detailed-errors")]
	fn from(inner: InvalidData) -> Self {
		Self::new(std::io::ErrorKind::InvalidData, inner)
	}

	#[cfg(not(feature = "detailed-errors"))]
	fn from(_: InvalidData) -> Self {
		std::io::ErrorKind::InvalidData.into()
	}
}

impl From<BadContentLength> for std::io::Error {
	fn from(inner: BadContentLength) -> Self {
		Into::<InvalidData>::into(inner).into()
	}
}

impl From<BadChunkHeader> for std::io::Error {
	fn from(inner: BadChunkHeader) -> Self {
		Into::<InvalidData>::into(inner).into()
	}
}
