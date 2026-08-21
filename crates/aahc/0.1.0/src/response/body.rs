mod chunked;
mod eof;
mod fixed;

use futures_io::AsyncRead;
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

/// An in-progress HTTP response which is currently receiving a response body.
///
/// The `'socket` lifetime parameter is the lifetime of the transport socket. The `Socket` type
/// parameter is the type of the transport-layer socket over which the HTTP response is received.
#[derive(Debug)]
enum Impl<'socket, Socket: AsyncRead + ?Sized> {
	Chunked(chunked::Receive<'socket, Socket>),
	EOF(eof::Receive<'socket, Socket>),
	Fixed(fixed::Receive<'socket, Socket>),
}

/// An in-progress HTTP response which is currently receiving a response body.
///
/// After the headers are received, an instance of this type is obtained. It implements
/// [`AsyncRead`](futures_io::AsyncRead), which allows the application to read the response body.
/// When the response body is finished, `finish` should be called to tidy up.
///
/// The `'socket` lifetime parameter is the lifetime of the transport socket. The `Socket` type
/// parameter is the type of the transport-layer socket over which the HTTP response is received.
#[derive(Debug)]
pub struct Receive<'socket, Socket: AsyncRead + ?Sized> {
	/// The body receiving implementation.
	body_impl: Impl<'socket, Socket>,

	/// Whether the request permitted the socket to persist after the body has been received.
	persistent: bool,
}

impl<'socket, Socket: AsyncRead + ?Sized> Receive<'socket, Socket> {
	/// Constructs a new `Receive` using chunked encoding.
	///
	/// The `socket` parameter is the underlying socket to read from. The `persistent` parameter
	/// indicates whether the connection can persist after the response is read.
	pub(crate) fn new_chunked(socket: Pin<&'socket mut Socket>, persistent: bool) -> Self {
		Self {
			body_impl: Impl::Chunked(chunked::Receive::new(socket)),
			persistent,
		}
	}

	/// Constructs a new `Receive` to read a response terminated by socket closure.
	///
	/// The `socket` parameter is the underlying socket to read from.
	pub(crate) fn new_eof(socket: Pin<&'socket mut Socket>) -> Self {
		Self {
			body_impl: Impl::EOF(eof::Receive::new(socket)),
			persistent: false,
		}
	}

	/// Constructs a new `Receive` with a fixed body length.
	///
	/// The `socket` parameter is the underlying socket to read from. The `length` parameter is the
	/// length of the response body. The `persistent` parameter indicates whether the connection
	/// can persist after the response is read.
	pub(crate) fn new_fixed(
		socket: Pin<&'socket mut Socket>,
		length: u64,
		persistent: bool,
	) -> Self {
		Self {
			body_impl: Impl::Fixed(fixed::Receive::new(socket, length)),
			persistent,
		}
	}

	/// Destroys a `Receive`.
	///
	/// This function returns `true` if the transport socket can be reused for another HTTP request
	/// to the same host, or `false` if the transport socket must be closed.
	#[allow(clippy::must_use_candidate)] // Return value can be ignored, the purpose of this function is mainly to drop self.
	pub fn finish(self) -> bool {
		self.persistent
			&& match self.body_impl {
				Impl::Chunked(chunked) => chunked.finish(),
				Impl::EOF(_) => false,
				Impl::Fixed(fixed) => fixed.finish(),
			}
	}
}

impl<'socket, Socket: AsyncRead + ?Sized> AsyncRead for Receive<'socket, Socket> {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut [u8],
	) -> Poll<Result<usize>> {
		match self.body_impl {
			Impl::Chunked(ref mut chunked) => Pin::new(chunked).poll_read(cx, buf),
			Impl::EOF(ref mut eof) => Pin::new(eof).poll_read(cx, buf),
			Impl::Fixed(ref mut fixed) => Pin::new(fixed).poll_read(cx, buf),
		}
	}

	fn poll_read_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &mut [std::io::IoSliceMut<'_>],
	) -> Poll<Result<usize>> {
		match self.body_impl {
			Impl::Chunked(ref mut chunked) => Pin::new(chunked).poll_read_vectored(cx, bufs),
			Impl::EOF(ref mut eof) => Pin::new(eof).poll_read_vectored(cx, bufs),
			Impl::Fixed(ref mut fixed) => Pin::new(fixed).poll_read_vectored(cx, bufs),
		}
	}
}
