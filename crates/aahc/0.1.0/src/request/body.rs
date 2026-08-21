mod chunked;
mod fixed;

use crate::request::Metadata;
use futures_io::AsyncWrite;
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

/// An in-progress HTTP request which is currently sending a request body.
///
/// The `'socket` lifetime parameter is the lifetime of the transport socket. The `Socket` type
/// parameter is the type of transport-layer socket over which the HTTP request will be sent.
#[derive(Debug, Eq, PartialEq)]
enum Mode<'socket, Socket: AsyncWrite + ?Sized> {
	/// The request body has a fixed size given by a `content-length` header or by the inherent
	/// properties of the request (e.g. a `HEAD` request always having a zero-length body).
	Fixed(fixed::Send<'socket, Socket>),

	/// The request body is encoded using chunked transfer encoding.
	Chunked(chunked::Send<'socket, Socket>),
}

/// An in-progress HTTP request which is currently sending a request body.
///
/// After the headers are sent, an instance of this type is obtained. It implements
/// [`AsyncWrite`](futures_io::AsyncWrite), which allows the application to write the request body.
/// When the request body is finished (or immediately, if there is no request body to send),
/// `finish` should be called to proceed to the next step.
///
/// The `'socket` lifetime parameter is the lifetime of the transport socket. The `Socket` type
/// parameter is the type of transport-layer socket over which the HTTP request will be sent.
#[derive(Debug, Eq, PartialEq)]
pub struct Send<'socket, Socket: AsyncWrite + ?Sized> {
	/// The underlying representation.
	inner: Mode<'socket, Socket>,

	/// The request metadata.
	metadata: Metadata,
}

impl<'socket, Socket: AsyncWrite + ?Sized> Send<'socket, Socket> {
	/// Constructs a new `Send` to send a request body of a fixed length.
	///
	/// The `socket` parameter is the transport-layer socket over which the HTTP request will be
	/// sent, and over which the request headers must already have been sent. The `head` parameter
	/// indicates whether the request method was `HEAD`. The `connection_close` parameter indicates
	/// whether the request headers contained `connection: close`. The `length` parameter is the
	/// length of the body to send.
	pub(crate) fn new_fixed(
		socket: Pin<&'socket mut Socket>,
		metadata: Metadata,
		length: u64,
	) -> Self {
		Self {
			inner: Mode::Fixed(fixed::Send::new(socket, length)),
			metadata,
		}
	}

	/// Constructs a new `Send` to send a request body using chunked encoding.
	///
	/// The `socket` parameter is the transport-layer socket over which the HTTP request will be
	/// sent, and over which the request headers must already have been sent. The `head` parameter
	/// indicates whether the request method was `HEAD`. The `connection_close` parameter indicates
	/// whether the request headers contained `connection: close`.
	pub(crate) fn new_chunked(socket: Pin<&'socket mut Socket>, metadata: Metadata) -> Self {
		Self {
			inner: Mode::Chunked(chunked::Send::new(socket)),
			metadata,
		}
	}

	/// Gives a hint about how many bytes of body remain to be sent.
	///
	/// The `length` parameter is the minimum number of bytes of body that will be sent from this
	/// point forward.
	///
	/// The application must send at least `length` bytes before finishing the request. However, it
	/// is permitted to send *more* than `length` bytes (subject to any other constraints, such as
	/// not sending more than specified in the `content-length` header).
	///
	/// For a fixed-length request (e.g. one whose length was set by a `content-length` header),
	/// this function does nothing.
	///
	/// For a chunked request, this function sets the size of the next chunk; this allows a large
	/// chunk size to be sent without the entire contents of the chunk needing to be available at
	/// once. For example, if the application knows that the body will contain *at least* another
	/// 100,000 bytes, it could call this function passing a `length` parameter of 100,000, but
	/// then write just 10,000 bytes at a time, ten times over. Without calling this function, that
	/// would result in ten 10,000-byte chunks being sent; by calling this function, a single
	/// 100,000-byte chunk is sent instead, reducing the overhead due to chunk headers, without
	/// requiring that the application load all 100,000 bytes into memory at once.
	pub fn hint_length(&mut self, length: u64) {
		match &mut self.inner {
			Mode::Fixed(_) => (),
			Mode::Chunked(inner) => inner.hint_length(length),
		}
	}

	/// Finishes the request.
	///
	/// On success, a metadata instance is returned. This instance must be passed in when the
	/// application is ready to receive the response headers.
	///
	/// *Important*: This function does not flush the socket. If the application is going to read a
	/// response, or if the socket is a buffered wrapper around an underlying socket and the
	/// application intends to unwrap the wrapper, it must flush the socket before proceeding to
	/// read the response, otherwise a hang might occur due to the server waiting for the remainder
	/// of the request which will never arrive. If the application intends to send another request
	/// instead, using HTTP pipelining, the flush is not necessary.
	///
	/// # Errors
	/// This function returns an error if writing to the underlying socket fails.
	///
	/// # Panics
	/// This function panics in a debug build:
	/// * if the body has a fixed length and was not fully sent
	/// * if the body is chunked and the most recent chunk was not fully sent
	///
	/// These are debug-build panics, not errors, because it is expected that the application will
	/// provide a request body consistent with the request headers; since both of these things are
	/// under the application’s control, an inconsistency between them represents a bug in the
	/// application.
	pub async fn finish(self) -> Result<Metadata> {
		match self.inner {
			Mode::Fixed(inner) => inner.finish(),
			Mode::Chunked(inner) => inner.finish().await?,
		}
		Ok(self.metadata)
	}
}

impl<Socket: AsyncWrite + ?Sized> AsyncWrite for Send<'_, Socket> {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize>> {
		match self.inner {
			Mode::Fixed(ref mut inner) => Pin::new(inner).poll_write(cx, buf),
			Mode::Chunked(ref mut inner) => Pin::new(inner).poll_write(cx, buf),
		}
	}

	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &[std::io::IoSlice<'_>],
	) -> Poll<Result<usize>> {
		match self.inner {
			Mode::Fixed(ref mut inner) => Pin::new(inner).poll_write_vectored(cx, bufs),
			Mode::Chunked(ref mut inner) => Pin::new(inner).poll_write_vectored(cx, bufs),
		}
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
		match self.inner {
			Mode::Fixed(ref mut inner) => Pin::new(inner).poll_flush(cx),
			Mode::Chunked(ref mut inner) => Pin::new(inner).poll_flush(cx),
		}
	}

	fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
		match self.inner {
			Mode::Fixed(ref mut inner) => Pin::new(inner).poll_close(cx),
			Mode::Chunked(ref mut inner) => Pin::new(inner).poll_close(cx),
		}
	}
}

#[cfg(test)]
mod test {
	use futures_io::AsyncWrite;
	use std::pin::Pin;

	pub trait AsyncWriteExt: AsyncWrite {
		fn write<'a>(self: Pin<&'a mut Self>, data: &'a [u8]) -> WriteFuture<'a, Self> {
			WriteFuture { sink: self, data }
		}
	}

	impl<W: AsyncWrite + ?Sized> AsyncWriteExt for W {}

	/// A future that writes data to an `AsyncWrite`.
	#[derive(Debug)]
	pub struct WriteFuture<'a, T: AsyncWrite + ?Sized> {
		sink: Pin<&'a mut T>,
		data: &'a [u8],
	}

	impl<'a, T: AsyncWrite + ?Sized> std::future::Future for WriteFuture<'a, T> {
		type Output = std::io::Result<usize>;

		fn poll(
			mut self: Pin<&mut Self>,
			cx: &mut std::task::Context<'_>,
		) -> std::task::Poll<Self::Output> {
			let data = self.data;
			self.sink.as_mut().poll_write(cx, data)
		}
	}
}
