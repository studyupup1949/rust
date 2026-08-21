use futures_io::AsyncRead;
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A response body that ends when the underlying socket is closed by the server.
#[derive(Debug)]
pub(super) struct Receive<'socket, Socket: AsyncRead + ?Sized> {
	/// The underlying socket.
	socket: Pin<&'socket mut Socket>,
}

impl<'socket, Socket: AsyncRead + ?Sized> Receive<'socket, Socket> {
	/// Constructs a new `Receive`.
	///
	/// The `socket` parameter is the underlying socket to read from.
	pub(super) fn new(socket: Pin<&'socket mut Socket>) -> Self {
		Self { socket }
	}
}

impl<Socket: AsyncRead + ?Sized> AsyncRead for Receive<'_, Socket> {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut [u8],
	) -> Poll<Result<usize>> {
		self.socket.as_mut().poll_read(cx, buf)
	}

	fn poll_read_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &mut [std::io::IoSliceMut<'_>],
	) -> Poll<Result<usize>> {
		self.socket.as_mut().poll_read_vectored(cx, bufs)
	}
}
