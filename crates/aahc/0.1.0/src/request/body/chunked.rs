use futures_core::ready;
use futures_io::AsyncWrite;
use std::cmp::max;
use std::io::Result;
use std::num::{NonZeroU64, NonZeroUsize};
use std::pin::Pin;
use std::task::{Context, Poll};

/// An in-progress HTTP request which is currently sending a chunk-encoded request body.
///
/// The `'socket` lifetime parameter is the lifetime of the transport socket. The `Socket` type
/// parameter is the type of transport-layer socket over which the HTTP request will be sent.
#[derive(Debug, Eq, PartialEq)]
pub struct Send<'socket, Socket: AsyncWrite + ?Sized> {
	/// The underlying socket.
	socket: Pin<&'socket mut Socket>,

	/// A buffer to hold a chunk header (size plus CRLF).
	header_footer_buffer: [u8; 22],

	/// The number of bytes of `header_footer_buffer` that are filled.
	header_footer_buffer_used: usize,

	/// The number of bytes of `header_footer_buffer` that have been sent over the socket.
	header_footer_buffer_sent: usize,

	/// The number of bytes left to send in the current chunk.
	chunk_bytes_left: u64,

	/// The length of the next chunk after the current one, if known.
	next_chunk_size: Option<NonZeroU64>,
}

impl<'socket, Socket: AsyncWrite + ?Sized> Send<'socket, Socket> {
	/// Constructs a new `Send`.
	///
	/// The `socket` parameter is the transport-layer socket over which the HTTP request will be
	/// sent.
	pub fn new(socket: Pin<&'socket mut Socket>) -> Self {
		Self {
			socket,
			header_footer_buffer: Default::default(),
			header_footer_buffer_used: 0,
			header_footer_buffer_sent: 0,
			chunk_bytes_left: 0,
			next_chunk_size: None,
		}
	}

	/// Gives a hint about how many bytes of body remain to be sent.
	///
	/// The `length` parameter is the minimum number of bytes of body that will be sent from this
	/// point forward.
	///
	/// The application must send at least `length` bytes before finishing the request. However, it
	/// is permitted to send *more* than `length` bytes.
	///
	/// For a fixed-length request (e.g. one whose length was set by a `content-length` header),
	/// this function does nothing.
	///
	/// For a chunked request, this function sets the size of the next chunk; this allows a large
	/// chunk size to be sent without the entire contents of the chunk needing to be available at
	/// once. For example, if the application knows that the body will contain *at least* another
	/// 100,000 bytes, it could call this function passing a `length` parameter of 100,000, but
	/// then write just 10,000 bytes at a time, ten times over. Without calling this function, that
	/// would result in ten 10,000-byte chunks being sent; by calling this function, instead a
	/// single 100,000-byte chunk is sent, reducing the overhead due to chunk headers without
	/// requiring that the application load all 100,000 bytes into memory at once.
	pub fn hint_length(&mut self, length: u64) {
		// The length of the next chunk is not always equal to the hint value. If we’re currently
		// in the middle of a chunk, we must finish that chunk first, which means the next chunk
		// will be the hint value minus however much is left in the current chunk.
		if length > self.chunk_bytes_left {
			let new_hint = NonZeroU64::new(length - self.chunk_bytes_left).unwrap();
			// Don’t always use the new hint. Use the largest next chunk size we’ve seen so far.
			self.next_chunk_size =
				Some(self.next_chunk_size.map_or(new_hint, |v| max(v, new_hint)));
		}
	}

	/// Polls sending the chunk header to the socket.
	///
	/// If the most recent chunk header has been fully sent, this function does nothing and
	/// indicates `Ready`. Otherwise, it tries to send the remainder of the chunk header. It
	/// returns `Ready` if sending the chunk header succeeded, or `Pending` if the socket refused
	/// to accept all the data.
	fn poll_send_header_footer(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
		while self.header_footer_buffer_sent != self.header_footer_buffer_used {
			// The chunk header isn’t finished sending yet. Try to make some progress there.
			let bytes_written = ready!(self.socket.as_mut().poll_write(
				cx,
				&self.header_footer_buffer
					[self.header_footer_buffer_sent..self.header_footer_buffer_used],
			))?;
			self.header_footer_buffer_sent += bytes_written;
		}
		Ok(()).into()
	}

	/// Prepares for a write of some data bytes.
	///
	/// The `cx` parameter is the asynchronous context. The `length` parameter is the number of
	/// bytes that are about to be written.
	///
	/// This function returns `Poll::Ready(Ok(n))` if the socket is in a condition where `n` bytes
	/// of body data can be sent. `n` is always less than or equal to `length`.
	fn pre_write(
		&mut self,
		cx: &mut Context<'_>,
		length: NonZeroUsize,
	) -> Poll<Result<NonZeroUsize>> {
		use std::convert::TryInto as _;

		// Start a chunk if we’re not currently inside one.
		if self.chunk_bytes_left == 0 {
			use std::io::Write as _;

			// We might have some footer in the buffer from a chunk we previously finished. If so,
			// send it out so the buffer is empty and we can fill it with chunk header.
			ready!(self.poll_send_header_footer(cx))?;

			// If we have a hint, use the maximum of the write size and the hint; if we don’t have
			// a hint, just use the write size. Destroy (take) the hint, since it only applies to
			// this chunk, not any future chunks.
			let length64 = NonZeroU64::new(length.get().try_into().unwrap_or(u64::MAX)).unwrap();
			let chunk_size = self
				.next_chunk_size
				.take()
				.map_or(length64, |v| max(v, length64));
			let mut cursor = std::io::Cursor::new(&mut self.header_footer_buffer[..]);
			write!(&mut cursor, "{:X}\r\n", chunk_size).unwrap();
			self.header_footer_buffer_used = cursor.position() as usize;
			self.header_footer_buffer_sent = 0;
			self.chunk_bytes_left = chunk_size.get();
		}

		// There might be some header in the buffer, either because we just put it there or because
		// we put it there on a previous call and didn’t get to send all of it. Drain it before
		// sending any body data.
		ready!(self.poll_send_header_footer(cx))?;

		// If we get here, we’re inside a chunk, we have at least one byte of chunk body left, and
		// the header/footer buffer is empty. Inform the caller that they can send some bytes,
		// either as many as they want or as many as fit in the rest of the current chunk,
		// whichever is smaller.
		Ok(std::cmp::min(
			length,
			NonZeroUsize::new(self.chunk_bytes_left.try_into().unwrap_or(usize::MAX)).unwrap(),
		))
		.into()
	}

	/// Does closing work after a write has completed.
	///
	/// The `bytes_written` parameter is the number of bytes written to the underlying socket.
	fn post_write(&mut self, bytes_written: usize) {
		self.chunk_bytes_left -= bytes_written as u64;
		if self.chunk_bytes_left == 0 {
			// At the end of a chunk, we must send a CRLF chunk footer.
			self.header_footer_buffer[0..2].copy_from_slice(b"\r\n");
			self.header_footer_buffer_sent = 0;
			self.header_footer_buffer_used = 2;
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
	/// This function panics in a debug build if the most recent chunk was not fully sent.
	///
	/// # Errors
	/// This function returns an error if writing to the underlying socket fails.
	pub async fn finish(mut self) -> Result<()> {
		use crate::util::io::AsyncWriteExt as _;

		struct SendHeaderFooterDataFuture<'socket, 'body, Socket: AsyncWrite + ?Sized> {
			body: &'body mut Send<'socket, Socket>,
		}
		impl<Socket: AsyncWrite + ?Sized> std::future::Future
			for SendHeaderFooterDataFuture<'_, '_, Socket>
		{
			type Output = Result<()>;
			fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
				self.body.poll_send_header_footer(cx)
			}
		}

		// Sanity check that the previous chunk, if any, was fully sent.
		debug_assert!(self.chunk_bytes_left == 0);

		// The most recent chunk’s footer would still be in the buffer. Drain it.
		SendHeaderFooterDataFuture { body: &mut self }.await?;

		// Send the end-of-body chunk header and footer.
		self.socket.as_mut().write_all(b"0\r\n\r\n").await?;

		Ok(())
	}
}

impl<Socket: AsyncWrite + ?Sized> AsyncWrite for Send<'_, Socket> {
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
		if let Some(buf_len) = NonZeroUsize::new(buf.len()) {
			let this = Pin::into_inner(self);

			// Prepare for the write.
			let to_write: NonZeroUsize = ready!(this.pre_write(cx, buf_len))?;

			// Send the number of bytes that pre_write said we could.
			let bytes_written = ready!(this
				.socket
				.as_mut()
				.poll_write(cx, &buf[..(to_write.get())]))?;

			// Close up the write.
			this.post_write(bytes_written);
			Ok(bytes_written).into()
		} else {
			// The caller asked to write zero bytes.
			Ok(0).into()
		}
	}

	fn poll_write_vectored(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &[std::io::IoSlice<'_>],
	) -> Poll<Result<usize>> {
		// Add up the total number of bytes to send.
		let total_length = bufs
			.iter()
			.map(|elt| elt.len())
			.fold(0_usize, usize::saturating_add);
		if let Some(total_length) = NonZeroUsize::new(total_length) {
			let this = Pin::into_inner(self);

			// Prepare for the write.
			let to_write: NonZeroUsize = ready!(this.pre_write(cx, total_length))?;

			// Perform the write.
			let first_buffer = &bufs[0];
			let bytes_written = if first_buffer.len() >= to_write.get() {
				// The first IoSlice alone covers at least to_write bytes. Write only that slice,
				// or part of it if it’s larger than to_write.
				ready!(this
					.socket
					.as_mut()
					.poll_write(cx, &first_buffer[..to_write.get()]))?
			} else {
				// The first IoSlice alone is smaller than to_write. Do a vectored write. To avoid
				// modifying any of the IoSlices, choose only enough IoSlices to add up to
				// ≤to_write bytes. This might even mean just one IoSlice (if the first slice is
				// smaller than to_write but the first two added are larger), but in general it
				// could be more than one.
				let buf_count: usize = bufs
					.iter()
					.scan(0_usize, |size_so_far, elt| {
						*size_so_far += elt.len();
						Some(*size_so_far > to_write.get())
					})
					.enumerate()
					.find(|elt| elt.1)
					.unwrap_or((bufs.len(), false))
					.0;
				ready!(this
					.socket
					.as_mut()
					.poll_write_vectored(cx, &bufs[..buf_count]))?
			};

			// Close up the write.
			this.post_write(bytes_written);
			Ok(bytes_written).into()
		} else {
			// The caller asked to write zero bytes.
			Ok(0).into()
		}
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
	use crate::request::body::test::AsyncWriteExt;
	use futures_executor::block_on;

	/// Tests sending a body with one single chunk.
	#[test]
	fn test_basic() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink));
			assert_eq!(
				Pin::new(&mut body).write(b"Hello World!").await.unwrap(),
				12
			);
			body.finish().await.unwrap();
			assert_eq!(sink, b"C\r\nHello World!\r\n0\r\n\r\n");
		});
	}

	/// Tests sending a body as two chunks, one after another.
	#[test]
	fn test_two_chunks() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink));
			assert_eq!(Pin::new(&mut body).write(b"Hello ").await.unwrap(), 6);
			assert_eq!(Pin::new(&mut body).write(b"World!").await.unwrap(), 6);
			body.finish().await.unwrap();
			assert_eq!(sink, b"6\r\nHello \r\n6\r\nWorld!\r\n0\r\n\r\n");
		});
	}

	/// Tests sending a body as one chunk, using a size hint to break it into two blocks but not
	/// two chunks.
	#[test]
	fn test_hint() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink));
			body.hint_length(12);
			assert_eq!(Pin::new(&mut body).write(b"Hello ").await.unwrap(), 6);
			assert_eq!(Pin::new(&mut body).write(b"World!").await.unwrap(), 6);
			body.finish().await.unwrap();
			assert_eq!(sink, b"C\r\nHello World!\r\n0\r\n\r\n");
		});
	}

	/// Tests use of a size hint where a single write overlaps the end of the hinted chunk.
	#[test]
	fn test_hint_overlap() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink));
			body.hint_length(10);
			assert_eq!(Pin::new(&mut body).write(b"Hello ").await.unwrap(), 6);
			assert_eq!(Pin::new(&mut body).write(b"World!").await.unwrap(), 4);
			assert_eq!(Pin::new(&mut body).write(b"d!").await.unwrap(), 2);
			body.finish().await.unwrap();
			assert_eq!(sink, b"A\r\nHello Worl\r\n2\r\nd!\r\n0\r\n\r\n");
		});
	}

	/// Tests that trying to finish while inside a chunk panics.
	#[test]
	#[should_panic]
	fn test_truncate() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink));
			body.hint_length(12);
			let _ = Pin::new(&mut body).write(b"Hello ").await;
			let _ = body.finish().await;
		});
	}

	/// Tests that writing zero-byte blocks in various places doesn’t create spurious empty chunks.
	#[test]
	fn test_zero_bytes() {
		block_on(async {
			let mut sink = Vec::new();
			let mut body = Send::new(Pin::new(&mut sink));
			assert_eq!(Pin::new(&mut body).write(b"").await.unwrap(), 0);
			body.hint_length(12);
			assert_eq!(Pin::new(&mut body).write(b"Hello ").await.unwrap(), 6);
			assert_eq!(Pin::new(&mut body).write(b"").await.unwrap(), 0);
			assert_eq!(Pin::new(&mut body).write(b"World!").await.unwrap(), 6);
			assert_eq!(Pin::new(&mut body).write(b"").await.unwrap(), 0);
			body.finish().await.unwrap();
			assert_eq!(sink, b"C\r\nHello World!\r\n0\r\n\r\n");
		});
	}
}
