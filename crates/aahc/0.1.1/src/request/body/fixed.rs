use futures_core::ready;
use futures_io::AsyncWrite;
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

/// An in-progress HTTP request which is currently sending a fixed-length request body.
///
/// The `'socket` lifetime parameter is the lifetime of the transport socket. The `Socket` type
/// parameter is the type of transport-layer socket over which the HTTP request will be sent.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct Send<'socket, Socket: AsyncWrite + ?Sized> {
	/// The underlying socket.
	socket: Pin<&'socket mut Socket>,

	/// How many bytes of body are left to send.
	remaining: u64,
}

impl<'socket, Socket: AsyncWrite + ?Sized> Send<'socket, Socket> {
	/// Constructs a new `Send`.
	///
	/// The `socket` parameter is the transport-layer socket over which the HTTP request will be
	/// sent. The `length` parameter is the number of bytes of body to send.
	pub(super) fn new(socket: Pin<&'socket mut Socket>, length: u64) -> Self {
		Self {
			socket,
			remaining: length,
		}
	}

	/// Finishes the request.
	///
	/// *Important*: This function does not flush the socket. If the application is going to read a
	/// response, or if the socket is a buffered wrapper around an underlying socket and the
	/// application intends to unwrap the wrapper, it must flush the socket before proceeding to
	/// read the response, otherwise a hang might occur due to the server waiting for the remainder
	/// of the request which will never arrive. If the application intends to send another request
	/// instead, using HTTP pipelining, the flush is not necessary.
	///
	/// # Panics
	/// This function panics in a debug build if the full request body has not yet been sent.
	pub(super) fn finish(self) {
		// Sanity check that the full request body has been sent.
		debug_assert!(self.remaining == 0);
	}
}

impl<Socket: AsyncWrite + ?Sized> AsyncWrite for Send<'_, Socket> {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize>> {
		// Sanity check that the body doesn’t overflow.
		debug_assert!(
			buf.len() as u64 <= self.remaining,
			"Attempted to write {} bytes, but Content-Length indicates only {} should be left to send",
			buf.len(),
			self.remaining
		);

		// Write to the underlying socket.
		let bytes_written = ready!(self.socket.as_mut().poll_write(cx, buf))?;
		self.remaining -= bytes_written as u64;
		Ok(bytes_written).into()
	}

	fn poll_write_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &[std::io::IoSlice<'_>],
	) -> Poll<Result<usize>> {
		// Sanity check that the body doesn’t overflow.
		debug_assert!(
			bufs.iter().map(|elt| elt.len() as u64).sum::<u64>() <= self.remaining,
			"Attempted to write {} bytes, but Content-Length indicates only {} should be left to send",
			bufs.iter().map(|elt| elt.len() as u64).sum::<u64>(),
			self.remaining
		);

		// Write to the underlying socket.
		let bytes_written = ready!(self.socket.as_mut().poll_write_vectored(cx, bufs))?;
		self.remaining -= bytes_written as u64;
		Ok(bytes_written).into()
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
		self.socket.as_mut().poll_flush(cx)
	}

	fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
		self.socket.as_mut().poll_close(cx)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::request::body::test::AsyncWriteExt as _;
	use futures_executor::block_on;

	/// Tests sending a request body of the proper length.
	#[test]
	fn basic() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink), 12);
			assert_eq!(
				Pin::new(&mut body).write(b"Hello World!").await.unwrap(),
				12
			);
			body.finish();
			assert_eq!(sink, b"Hello World!");
		});
	}

	/// Tests sending a request body of the proper length in two parts.
	#[test]
	fn two_parts() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink), 12);
			assert_eq!(Pin::new(&mut body).write(b"Hello ").await.unwrap(), 6);
			assert_eq!(Pin::new(&mut body).write(b"World!").await.unwrap(), 6);
			body.finish();
			assert_eq!(sink, b"Hello World!");
		});
	}

	/// Tests sending not enough data before finishing.
	#[test]
	#[should_panic = "remaining == 0"]
	fn truncated() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink), 13);
			let bytes_written = Pin::new(&mut body).write(b"Hello World!").await.unwrap();
			assert_eq!(bytes_written, 12);
			body.finish();
		});
	}

	/// Tests sending too much data.
	#[test]
	#[should_panic = "Attempted to write 12 bytes, but Content-Length indicates only 11 should be left to send"]
	fn overflow() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink), 11);
			let _ = Pin::new(&mut body).write(b"Hello World!").await;
		});
	}
}
