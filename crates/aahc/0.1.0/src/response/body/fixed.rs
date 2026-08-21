use futures_core::ready;
use futures_io::AsyncRead;
use std::io::{ErrorKind, Result};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A response body that is a fixed length known a priori from a `Content-Length` header.
#[derive(Debug)]
pub struct Receive<'socket, Socket: AsyncRead + ?Sized> {
	/// The underlying socket.
	socket: Pin<&'socket mut Socket>,

	/// The amount of body left for the caller to receive.
	remaining: u64,
}

impl<'socket, Socket: AsyncRead + ?Sized> Receive<'socket, Socket> {
	/// Constructs a new `Receive`.
	///
	/// The `socket` parameter is the underlying socket to read from. The `length` parameter is the
	/// length of the response body.
	pub fn new(socket: Pin<&'socket mut Socket>, length: u64) -> Self {
		Self {
			socket,
			remaining: length,
		}
	}

	/// Destroys a `Receive`.
	///
	/// This function returns `true` if the entire response body has been received, or `false` if
	/// not.
	pub fn finish(self) -> bool {
		self.remaining == 0
	}
}

impl<'socket, Socket: AsyncRead + ?Sized> AsyncRead for Receive<'socket, Socket> {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut [u8],
	) -> Poll<Result<usize>> {
		if self.remaining == 0 {
			Ok(0).into()
		} else {
			let to_read = std::cmp::min(buf.len() as u64, self.remaining) as usize;
			let bytes_read = ready!(self.socket.as_mut().poll_read(cx, &mut buf[..to_read]))?;
			if bytes_read == 0 {
				Err(ErrorKind::UnexpectedEof.into()).into()
			} else {
				self.remaining -= bytes_read as u64;
				Ok(bytes_read).into()
			}
		}
	}

	fn poll_read_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &mut [std::io::IoSliceMut<'_>],
	) -> Poll<Result<usize>> {
		if self.remaining == 0 {
			Ok(0).into()
		} else {
			use crate::util::io::AsyncReadExt as _;
			let remaining = self.remaining;
			let bytes_read = ready!(self
				.socket
				.as_mut()
				.poll_read_vectored_bounded(cx, bufs, remaining))?;
			if bytes_read == 0 {
				return Err(ErrorKind::UnexpectedEof.into()).into();
			}
			self.remaining -= bytes_read as u64;
			Ok(bytes_read).into()
		}
	}
}

#[cfg(test)]
mod test {
	use super::Receive;
	use crate::util::io::AsyncReadExt as _;
	use futures_executor::block_on;
	use std::io::IoSliceMut;
	use std::pin::Pin;

	/// Tests receiving all the body via a single poll_read.
	#[test]
	fn test_read_all() {
		block_on(async {
			let mut data = &b"abcdefgh"[..];
			let mut rx = Receive::new(Pin::new(&mut data), 6);
			let mut buffer = [0_u8; 8];
			let bytes_read = Pin::new(&mut rx).read(&mut buffer).await.unwrap();
			assert_eq!(bytes_read, 6);
			assert_eq!(&buffer, b"abcdef\0\0");
			assert!(rx.finish());
		});
	}

	/// Tests receiving all the body via multiple poll_reads.
	#[test]
	fn test_read_all_multi() {
		block_on(async {
			let mut data = &b"abcdefgh"[..];
			let mut rx = Receive::new(Pin::new(&mut data), 6);
			let mut buffer = [0_u8; 4];
			let bytes_read = Pin::new(&mut rx).read(&mut buffer).await.unwrap();
			assert_eq!(bytes_read, 4);
			assert_eq!(&buffer, b"abcd");
			let bytes_read = Pin::new(&mut rx).read(&mut buffer).await.unwrap();
			assert_eq!(bytes_read, 2);
			assert_eq!(&buffer, b"efcd");
			assert!(rx.finish());
		});
	}

	/// Tests receiving only part of the body via poll_read before finishing.
	#[test]
	fn test_read_some() {
		block_on(async {
			let mut data = &b"abcdefgh"[..];
			let mut rx = Receive::new(Pin::new(&mut data), 6);
			let mut buffer = [0_u8; 4];
			let bytes_read = Pin::new(&mut rx).read(&mut buffer).await.unwrap();
			assert_eq!(bytes_read, 4);
			assert_eq!(&buffer, b"abcd");
			assert!(!rx.finish());
		});
	}

	/// Tests receiving all the body via a single poll_read_vectored.
	#[test]
	fn test_read_vectored_all() {
		block_on(async {
			let mut data = &b"abcdefgh"[..];
			let mut rx = Receive::new(Pin::new(&mut data), 6);
			let mut buf1 = [0_u8; 4];
			let mut buf2 = [0_u8; 2];
			let mut slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
			let bytes_read = Pin::new(&mut rx).read_vectored(&mut slices).await.unwrap();
			assert_eq!(bytes_read, 6);
			assert_eq!(&buf1, b"abcd");
			assert_eq!(&buf2, b"ef");
			assert!(rx.finish());
		});
	}

	/// Tests receiving all the body via multiple poll_read_vectoreds.
	#[test]
	fn test_read_vectored_all_multi() {
		block_on(async {
			let mut data = &b"abcdefgh"[..];
			let mut rx = Receive::new(Pin::new(&mut data), 6);
			let mut buf1 = [0_u8; 2];
			let mut buf2 = [0_u8; 1];
			let mut slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
			let bytes_read = Pin::new(&mut rx).read_vectored(&mut slices).await.unwrap();
			assert_eq!(bytes_read, 3);
			assert_eq!(&buf1, b"ab");
			assert_eq!(&buf2, b"c");
			let mut slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
			let bytes_read = Pin::new(&mut rx).read_vectored(&mut slices).await.unwrap();
			assert_eq!(bytes_read, 3);
			assert_eq!(&buf1, b"de");
			assert_eq!(&buf2, b"f");
			assert!(rx.finish());
		});
	}

	/// Tests receiving only part of the body via poll_read_vectored before finishing.
	#[test]
	fn test_read_vectored_some() {
		block_on(async {
			let mut data = &b"abcdefgh"[..];
			let mut rx = Receive::new(Pin::new(&mut data), 6);
			let mut buf1 = [0_u8; 3];
			let mut buf2 = [0_u8; 2];
			let mut slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
			let bytes_read = Pin::new(&mut rx).read_vectored(&mut slices).await.unwrap();
			assert_eq!(bytes_read, 5);
			assert_eq!(&buf1, b"abc");
			assert_eq!(&buf2, b"de");
			assert!(!rx.finish());
		});
	}
}
