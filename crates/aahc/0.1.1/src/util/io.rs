use futures_core::ready;
use futures_io::{AsyncBufRead, AsyncRead, AsyncWrite};
use std::io::{IoSliceMut, Result};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A set of additional utility functions available on any type implementing `AsyncBufRead`.
pub(crate) trait AsyncBufReadExt: AsyncBufRead {
	/// Fills the internal buffer, then invokes a callback which can consume some bytes from that
	/// buffer.
	///
	/// This function adapts the `poll_fill_buf` and `consume` methods into a proper future.
	///
	/// The type parameter `CallbackReturn` is the type returned by the callback, and `Callback` is
	/// the callback itself.
	///
	/// The parameter `callback` is the callback function which can use data from the buffer. It is
	/// called at most once per call to `read_buf` (it may not be called at all if an error
	/// occurs). It is passed the bytes in the buffer. Its return value identifies how many bytes
	/// to consume from the buffer, along with an arbitrary value to pass back to the caller of
	/// `read_buf`.
	fn read_buf<CallbackReturn, Callback: FnOnce(&'_ [u8]) -> (usize, CallbackReturn) + Unpin>(
		self: Pin<&mut Self>,
		callback: Callback,
	) -> ReadBufFuture<'_, Self, CallbackReturn, Callback> {
		ReadBufFuture {
			source: self,
			callback: Some(callback),
		}
	}
}

impl<R: AsyncBufRead + ?Sized> AsyncBufReadExt for R {}

/// A set of additional utility functions available on any type implementing `AsyncRead`.
pub(crate) trait AsyncReadExt: AsyncRead {
	/// Reads data to a caller-provided buffer.
	#[cfg(test)]
	fn read<'buffer>(
		self: Pin<&mut Self>,
		buffer: &'buffer mut [u8],
	) -> ReadFuture<'_, 'buffer, Self> {
		ReadFuture {
			source: self,
			buffer,
		}
	}

	/// Reads data to multiple caller-provided buffers.
	#[cfg(test)]
	fn read_vectored<'buffers>(
		self: Pin<&mut Self>,
		buffers: &'buffers mut [IoSliceMut<'buffers>],
	) -> ReadVectoredFuture<'_, 'buffers, Self> {
		ReadVectoredFuture {
			source: self,
			buffers,
		}
	}

	/// Performs a vectored read with an additional length limit.
	#[cfg(test)]
	fn read_vectored_bounded<'bufs>(
		self: Pin<&mut Self>,
		bufs: &'bufs mut [IoSliceMut<'bufs>],
		limit: u64,
	) -> ReadVectoredBoundedFuture<'_, 'bufs, Self> {
		ReadVectoredBoundedFuture {
			source: self,
			bufs,
			limit,
		}
	}

	/// A wrapper around [`futures_io::AsyncRead::poll_read_vectored`] that limits the amount of
	/// data returned to a specified quantity.
	fn poll_read_vectored_bounded(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &mut [IoSliceMut<'_>],
		limit: u64,
	) -> Poll<Result<usize>> {
		if limit == 0 {
			Ok(0).into()
		} else {
			let limit = std::cmp::min(limit, usize::MAX as u64) as usize;
			let first_buffer = &mut bufs[0];
			if first_buffer.len() >= limit {
				// The first IoSlice alone covers at least limit bytes. Our read will only touch that
				// slice and no more.
				self.poll_read(cx, &mut first_buffer[..limit])
			} else {
				// The first IoSlice alone is smaller than limit bytes. Do a vectored read. To avoid
				// modifying any of the IoSlices, choose only enough IoSlices to add up to ≤limit
				// bytes. This might even mean just one IoSlice (if the first slice is smaller than
				// limit but the first two added are larger), but in general it could be more than one.
				let buf_count: usize = bufs
					.iter()
					.scan(0_usize, |size_so_far, elt| {
						*size_so_far += elt.len();
						Some(*size_so_far > limit)
					})
					.enumerate()
					.find(|elt| elt.1)
					.unwrap_or((bufs.len(), false))
					.0;
				self.poll_read_vectored(cx, &mut bufs[..buf_count])
			}
		}
	}
}

impl<R: AsyncRead + ?Sized> AsyncReadExt for R {}

/// A set of additional utility functions available on any type implementing `AsyncWrite`.
pub(crate) trait AsyncWriteExt: AsyncWrite {
	/// Writes a block of bytes to the writeable.
	///
	/// This function performs repeated writes into the writeable until the entire requested data
	/// has been written.
	fn write_all<'a>(self: Pin<&'a mut Self>, data: &'a [u8]) -> WriteAllFuture<'a, Self> {
		WriteAllFuture { sink: self, data }
	}
}

impl<W: AsyncWrite + ?Sized> AsyncWriteExt for W {}

/// A future that fills an `AsyncBufRead`’s internal buffer and then invokes a callback to consume
/// some or all of the data.
#[derive(Debug)]
pub(crate) struct ReadBufFuture<
	'source,
	Source: AsyncBufRead + ?Sized,
	CallbackReturn,
	Callback: FnOnce(&[u8]) -> (usize, CallbackReturn) + Unpin,
> {
	source: Pin<&'source mut Source>,
	callback: Option<Callback>,
}

impl<
	Source: AsyncBufRead + ?Sized,
	CallbackReturn,
	Callback: FnOnce(&[u8]) -> (usize, CallbackReturn) + Unpin,
> Future for ReadBufFuture<'_, Source, CallbackReturn, Callback>
{
	type Output = Result<CallbackReturn>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.get_mut();
		let data = ready!(this.source.as_mut().poll_fill_buf(cx))?;
		let (consumed, ret) = (this.callback.take().unwrap())(data);
		this.source.as_mut().consume(consumed);
		Ok(ret).into()
	}
}

/// A future that reads from an `AsyncRead` into a single caller-provided buffer.
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct ReadFuture<'source, 'buffer, Source: AsyncRead + ?Sized> {
	source: Pin<&'source mut Source>,
	buffer: &'buffer mut [u8],
}

#[cfg(test)]
impl<Source: AsyncRead + ?Sized> Future for ReadFuture<'_, '_, Source> {
	type Output = Result<usize>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.get_mut();
		this.source.as_mut().poll_read(cx, this.buffer)
	}
}

/// A future that reads from an `AsyncRead` into a collection of caller-provided buffers.
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct ReadVectoredFuture<'source, 'buffers, Source: AsyncRead + ?Sized> {
	source: Pin<&'source mut Source>,
	buffers: &'buffers mut [IoSliceMut<'buffers>],
}

#[cfg(test)]
impl<Source: AsyncRead + ?Sized> Future for ReadVectoredFuture<'_, '_, Source> {
	type Output = Result<usize>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.get_mut();
		this.source.as_mut().poll_read_vectored(cx, this.buffers)
	}
}

/// A future that reads from an `AsyncRead` into a collection of caller-provided buffers, with an
/// additional length limit.
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct ReadVectoredBoundedFuture<'source, 'bufs, Source: AsyncRead + ?Sized> {
	source: Pin<&'source mut Source>,
	bufs: &'bufs mut [IoSliceMut<'bufs>],
	limit: u64,
}

#[cfg(test)]
impl<Source: AsyncRead + ?Sized> Future for ReadVectoredBoundedFuture<'_, '_, Source> {
	type Output = Result<usize>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.get_mut();
		this.source
			.as_mut()
			.poll_read_vectored_bounded(cx, this.bufs, this.limit)
	}
}

/// A future that writes all of an array to an `AsyncWrite`.
#[derive(Debug)]
pub(crate) struct WriteAllFuture<'a, T: AsyncWrite + ?Sized> {
	sink: Pin<&'a mut T>,
	data: &'a [u8],
}

impl<T: AsyncWrite + ?Sized> Future for WriteAllFuture<'_, T> {
	type Output = Result<()>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		while !self.data.is_empty() {
			let data = self.data;
			let bytes_written = ready!(self.sink.as_mut().poll_write(cx, data))?;
			self.data = &self.data[bytes_written..];
		}
		Ok(()).into()
	}
}

/// Issues repeated reads until the caller-provided buffer is full.
#[cfg(test)]
pub(crate) async fn read_all<Source: AsyncRead + ?Sized>(
	mut src: Pin<&mut Source>,
	mut buffer: &mut [u8],
) -> Result<()> {
	while !buffer.is_empty() {
		let bytes_read = src.as_mut().read(buffer).await?;
		if bytes_read == 0 {
			return Err(std::io::ErrorKind::UnexpectedEof.into());
		} else {
			buffer = &mut buffer[bytes_read..];
		}
	}
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;
	use futures_executor::block_on;

	/// Tests calling `read` on a source.
	#[test]
	fn read() {
		block_on(async {
			let mut src: &[u8] = &b"abcdefgh"[..];
			let mut buffer = [0_u8; 4];
			let bytes_read = Pin::new(&mut src).read(&mut buffer[..]).await.unwrap();
			assert_eq!(bytes_read, 4);
			assert_eq!(&buffer, b"abcd");
		});
	}

	/// Tests calling `read_vectored` on a source.
	#[test]
	fn read_vectored() {
		block_on(async {
			let mut src: &[u8] = &b"abcdefgh"[..];
			let mut buf1 = [0_u8; 4];
			let mut buf2 = [0_u8; 2];
			let mut slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
			let bytes_read = Pin::new(&mut src).read_vectored(&mut slices).await.unwrap();
			assert_eq!(bytes_read, 6);
			assert_eq!(&buf1, b"abcd");
			assert_eq!(&buf2, b"ef");
		});
	}

	/// Tests calling `poll_read_vectored_bounded` with a limit small enough to fill less than one
	/// buffer.
	#[test]
	fn poll_read_vectored_bounded_one_partial() {
		block_on(async {
			let mut src: &[u8] = &b"abcdefgh"[..];
			let mut buf1 = [0_u8; 4];
			let mut buf2 = [0_u8; 4];
			let mut slices = [
				IoSliceMut::new(&mut buf1[..]),
				IoSliceMut::new(&mut buf2[..]),
			];
			let bytes_read = Pin::new(&mut src)
				.read_vectored_bounded(&mut slices, 3)
				.await
				.unwrap();
			assert_eq!(bytes_read, 3);
			assert_eq!(&buf1, b"abc\0");
			assert_eq!(&buf2, b"\0\0\0\0");
		});
	}

	/// Tests calling `poll_read_vectored_bounded` with a limit large enough to fill the first
	/// buffer, but not the first two.
	#[test]
	fn poll_read_vectored_bounded_one_full() {
		block_on(async {
			let mut src: &[u8] = &b"abcdefgh"[..];
			let mut buf1 = [0_u8; 4];
			let mut buf2 = [0_u8; 4];
			let mut slices = [
				IoSliceMut::new(&mut buf1[..]),
				IoSliceMut::new(&mut buf2[..]),
			];
			let bytes_read = Pin::new(&mut src)
				.read_vectored_bounded(&mut slices, 5)
				.await
				.unwrap();
			assert_eq!(bytes_read, 4);
			assert_eq!(&buf1, b"abcd");
			assert_eq!(&buf2, b"\0\0\0\0");
		});
	}

	/// Tests calling `poll_read_vectored_bounded` with a limit large enough to fill the first two
	/// buffers.
	#[test]
	fn poll_read_vectored_bounded_two_full() {
		block_on(async {
			let mut src: &[u8] = &b"abcdefghij"[..];
			let mut buf1 = [0_u8; 4];
			let mut buf2 = [0_u8; 4];
			let mut slices = [
				IoSliceMut::new(&mut buf1[..]),
				IoSliceMut::new(&mut buf2[..]),
			];
			let bytes_read = Pin::new(&mut src)
				.read_vectored_bounded(&mut slices, 10)
				.await
				.unwrap();
			assert_eq!(bytes_read, 8);
			assert_eq!(&buf1, b"abcd");
			assert_eq!(&buf2, b"efgh");
		});
	}

	/// Tests calling `write_all` on a sink that accepts unlimited data at a time.
	#[test]
	fn write_all_fast() {
		struct Test {
			v: Vec<u8>,
		}
		impl AsyncWrite for Test {
			fn poll_write(
				mut self: Pin<&mut Self>,
				_cx: &mut Context<'_>,
				buf: &[u8],
			) -> Poll<Result<usize>> {
				self.v.extend_from_slice(buf);
				Ok(buf.len()).into()
			}

			fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
				panic!("Should not be called");
			}

			fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
				panic!("Should not be called");
			}
		}
		let mut t = Test { v: vec![] };
		block_on(async { Pin::new(&mut t).write_all(b"abcdefgh").await }).unwrap();
		assert_eq!(t.v.as_slice(), b"abcdefgh");
	}

	/// Tests calling `write_all` on a sink that accepts data only one byte at a time.
	#[test]
	fn write_all_slow() {
		struct Test {
			v: Vec<u8>,
		}
		impl AsyncWrite for Test {
			fn poll_write(
				mut self: Pin<&mut Self>,
				_cx: &mut Context<'_>,
				buf: &[u8],
			) -> Poll<Result<usize>> {
				match buf.first() {
					None => Ok(0).into(),
					Some(&b) => {
						self.v.push(b);
						Ok(1).into()
					}
				}
			}

			fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
				panic!("Should not be called");
			}

			fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
				panic!("Should not be called");
			}
		}
		let mut t = Test { v: vec![] };
		block_on(async { Pin::new(&mut t).write_all(b"abcdefgh").await }).unwrap();
		assert_eq!(t.v.as_slice(), b"abcdefgh");
	}

	/// Tests calling `write_all` on a sink that returns an error.
	#[test]
	fn write_all_error() {
		struct Test {
			already_called: bool,
		}
		impl AsyncWrite for Test {
			fn poll_write(
				mut self: Pin<&mut Self>,
				_cx: &mut Context<'_>,
				_data: &[u8],
			) -> Poll<Result<usize>> {
				assert!(!self.already_called);
				self.already_called = true;
				Err(std::io::Error::other("Test error message")).into()
			}

			fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
				panic!("Should not be called");
			}

			fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
				panic!("Should not be called");
			}
		}
		let mut t = Test {
			already_called: false,
		};
		let e = block_on(async { Pin::new(&mut t).write_all(b"abcdefgh").await }).unwrap_err();
		assert_eq!(e.kind(), std::io::ErrorKind::Other);
		assert_eq!(format!("{e}"), "Test error message");
	}
}
