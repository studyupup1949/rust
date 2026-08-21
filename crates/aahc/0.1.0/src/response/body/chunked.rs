use futures_core::ready;
use futures_io::AsyncRead;
use std::io::Result;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Checks whether a character can legally appear in the chunk extensions section.
fn is_chunk_ext_char(b: u8) -> bool {
	b == b'\t' || !b.is_ascii_control()
}

/// The different states that the decoder can be in.
#[derive(Debug, Eq, PartialEq)]
enum State {
	/// The size part of a chunk header is being read, and no characters have been seen yet.
	///
	/// Once a hex digit is seen, state transitions to [`ChunkSizeRest`].
	SizeFirst,

	/// The size part of a chunk header is being read, and at least one hex digit has been seen.
	///
	/// The contained value is the decoded chunk size read so far.
	///
	/// Once a semicolon is seen, state transitions to [`ChunkExt`]. Once a CR is seen, state
	/// transitions to [`HeaderLF`].
	SizeRest(u64),

	/// The chunk extensions, are being read.
	///
	/// The contained value is the chunk size.
	///
	/// Once a CR is seen, state transitions to [`HeaderLF`].
	Ext(u64),

	/// The LF at the end of a chunk header is being read.
	///
	/// The contained value is the chunk size.
	///
	/// Once an LF is seen, state transitions to [`Data`] or [`FinalCR`].
	HeaderLF(u64),

	/// The chunk data is being read.
	///
	/// The contained value is the remaining chunk size.
	///
	/// Once all the bytes have been read, state transitions to [`DataCR`].
	Data(NonZeroU64),

	/// The CR following the chunk data is being read.
	///
	/// Once a CR is seen, state transitions to [`DataLF`].
	DataCR,

	/// The LF following the chunk data is being read.
	///
	/// Once an LF is seen, state transitions to [`SizeFirst`].
	DataLF,

	/// The CR following the terminal chunk header is being read.
	///
	/// Once a CR is seen, state transitions to [`FinalLF`].
	FinalCR,

	/// The LF following the terminal chunk header is being read.
	///
	/// Once an LF is seen, state transitions to [`Done`].
	FinalLF,

	/// Everything has been read.
	Done,
}

/// A response body that is encoded using chunked transfer coding.
#[derive(Debug)]
pub struct Receive<'socket, Socket: AsyncRead + ?Sized> {
	/// The underlying socket.
	socket: Pin<&'socket mut Socket>,

	/// The current state.
	state: State,
}

impl<'socket, Socket: AsyncRead + ?Sized> Receive<'socket, Socket> {
	/// Constructs a new `Receive`.
	///
	/// The `socket` parameter is the underlying socket to read from.
	pub fn new(socket: Pin<&'socket mut Socket>) -> Self {
		Self {
			socket,
			state: State::SizeFirst,
		}
	}

	/// Destroys a `Receive`.
	///
	/// This function returns `true` if the entire response body has been received, or `false` if
	/// not.
	pub fn finish(self) -> bool {
		self.state == State::Done
	}

	/// Reads and returns one byte from the socket.
	fn poll_read_byte(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<u8>> {
		let mut byte = [0_u8; 1];
		if ready!(self.socket.as_mut().poll_read(cx, &mut byte))? == 1 {
			Ok(byte[0]).into()
		} else {
			Err(std::io::ErrorKind::UnexpectedEof.into()).into()
		}
	}

	/// Advances the state to either [`State::Data`] or [`State::Done`].
	///
	/// This function returns `Some(n)` in the case of [`State::Data`], where `n` is the number of
	/// bytes of body data that can be fetched; `None` in the case of [`State::Done`]; or
	/// [`Poll::Pending`] in case it is not possible to advance to either of those states because
	/// not enough data has been received yet.
	fn poll_advance_to_data(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<Option<NonZeroU64>>> {
		loop {
			use crate::error::BadChunkHeader;
			match self.state {
				State::SizeFirst => {
					let b = ready!(self.as_mut().poll_read_byte(cx))?;
					if b.is_ascii_hexdigit() {
						// Unwrap is safe because b.is_ascii_hexdigit().
						let nybble: u64 =
							u64::from_str_radix(std::str::from_utf8(&[b]).unwrap(), 16).unwrap();
						self.state = State::SizeRest(nybble);
					} else {
						break Err(BadChunkHeader::SizeNotHex.into()).into();
					}
				}

				State::SizeRest(size_so_far) => match ready!(self.as_mut().poll_read_byte(cx))? {
					b';' => self.state = State::Ext(size_so_far),
					b'\r' => self.state = State::HeaderLF(size_so_far),
					b if b.is_ascii_hexdigit() => {
						if size_so_far >= 0x1000_0000_0000_0000_u64 {
							// Adding another digit would overflow.
							break Err(BadChunkHeader::SizeNotU64.into()).into();
						} else {
							// Unwrap is safe because b.is_ascii_hexdigit().
							let nybble: u64 =
								u64::from_str_radix(std::str::from_utf8(&[b]).unwrap(), 16)
									.unwrap();
							self.state = State::SizeRest((size_so_far << 4) | nybble);
						}
					}
					_ => break Err(BadChunkHeader::SizeNotHex.into()).into(),
				},

				State::Ext(chunk_size) => match ready!(self.as_mut().poll_read_byte(cx))? {
					b'\r' => self.state = State::HeaderLF(chunk_size),
					b if is_chunk_ext_char(b) => (),
					_ => break Err(BadChunkHeader::ExtChar.into()).into(),
				},

				State::HeaderLF(chunk_size) => {
					if ready!(self.as_mut().poll_read_byte(cx))? == b'\n' {
						self.state = if let Some(n) = NonZeroU64::new(chunk_size) {
							State::Data(n)
						} else {
							State::FinalCR
						};
					} else {
						break Err(BadChunkHeader::Newline.into()).into();
					}
				}

				State::Data(chunk_remaining) => break Ok(Some(chunk_remaining)).into(),

				State::DataCR => {
					if ready!(self.as_mut().poll_read_byte(cx))? == b'\r' {
						self.state = State::DataLF;
					} else {
						break Err(BadChunkHeader::Newline.into()).into();
					}
				}

				State::DataLF => {
					if ready!(self.as_mut().poll_read_byte(cx))? == b'\n' {
						self.state = State::SizeFirst;
					} else {
						break Err(BadChunkHeader::Newline.into()).into();
					}
				}

				State::FinalCR => {
					if ready!(self.as_mut().poll_read_byte(cx))? == b'\r' {
						self.state = State::FinalLF;
					} else {
						break Err(BadChunkHeader::Newline.into()).into();
					}
				}

				State::FinalLF => {
					if ready!(self.as_mut().poll_read_byte(cx))? == b'\n' {
						self.state = State::Done;
					} else {
						break Err(BadChunkHeader::Newline.into()).into();
					}
				}

				State::Done => break Ok(None).into(),
			}
		}
	}
}

impl<'socket, Socket: AsyncRead + ?Sized> AsyncRead for Receive<'socket, Socket> {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut [u8],
	) -> Poll<Result<usize>> {
		match ready!(self.as_mut().poll_advance_to_data(cx))? {
			None => Ok(0).into(),
			Some(bytes_available) => {
				let to_read = std::cmp::min(buf.len() as u64, bytes_available.get()) as usize;
				let buf = &mut buf[..to_read];
				let bytes_read = ready!(self.socket.as_mut().poll_read(cx, buf))?;
				let bytes_remaining = bytes_available.get() - bytes_read as u64;
				self.state = match NonZeroU64::new(bytes_remaining) {
					Some(bytes_remaining) => State::Data(bytes_remaining),
					None => State::DataCR,
				};
				Ok(bytes_read).into()
			}
		}
	}

	fn poll_read_vectored(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		bufs: &mut [std::io::IoSliceMut<'_>],
	) -> Poll<Result<usize>> {
		match ready!(self.as_mut().poll_advance_to_data(cx))? {
			None => Ok(0).into(),
			Some(bytes_available) => {
				use crate::util::io::AsyncReadExt as _;
				let bytes_read = ready!(self.socket.as_mut().poll_read_vectored_bounded(
					cx,
					bufs,
					bytes_available.get()
				))?;
				let bytes_remaining = bytes_available.get() - bytes_read as u64;
				self.state = match NonZeroU64::new(bytes_remaining) {
					Some(bytes_remaining) => State::Data(bytes_remaining),
					None => State::DataCR,
				};
				Ok(bytes_read).into()
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::Receive;
	use crate::util::io::AsyncReadExt as _;
	use futures_executor::block_on;
	use std::pin::Pin;

	/// Tests that a given raw input can be read to a given output, and that doing so consumes the
	/// entire input.
	async fn test_reads_all_input(mut input: &[u8], expected_output: &[u8]) {
		// Read and verify the output.
		let mut rx = Receive::new(Pin::new(&mut input));
		let mut actual_output = vec![0_u8; expected_output.len()].into_boxed_slice();
		crate::util::io::read_all(Pin::new(&mut rx), &mut actual_output[..])
			.await
			.unwrap();
		assert_eq!(*actual_output, *expected_output);

		// Make sure reading another byte doesn’t work.
		let mut another_byte = [0_u8; 1];
		assert_eq!(
			Pin::new(&mut rx).read(&mut another_byte[..]).await.unwrap(),
			0
		);

		// Now that we have *tried* to read another byte, we should be at end of input.
		assert!(input.is_empty());
	}

	/// Tests reading some chunked data via `poll_read`.
	#[test]
	fn test_poll_read() {
		block_on(async {
			test_reads_all_input(
				&b"006\r\nHello \r\n006\r\nWorld!\r\n0\r\n\r\n"[..],
				b"Hello World!",
			)
			.await;
		});
	}

	/// Tests reading some chunked data with chunk extensions.
	#[test]
	fn test_poll_read_exts() {
		block_on(async {
			test_reads_all_input(&b"006; cext-name=cext-value\r\nHello \r\n006; cext-name=\"quoted-cext-value-with\ttabs-in-it\"\r\nWorld!\r\n0\r\n\r\n"[..], b"Hello World!").await;
		});
	}

	/// Tests doing a vectored read.
	#[test]
	fn test_poll_read_vectored() {
		block_on(async {
			use std::io::IoSliceMut;

			// Read and verify the output.
			let mut input = &b"C\r\nHello World!\r\n0\r\n\r\n"[..];
			let mut rx = Receive::new(Pin::new(&mut input));
			let mut output1 = [0_u8; 6];
			let mut output2 = [0_u8; 6];
			let mut slices = [IoSliceMut::new(&mut output1), IoSliceMut::new(&mut output2)];
			let bytes_read = Pin::new(&mut rx).read_vectored(&mut slices).await.unwrap();
			assert_eq!(bytes_read, 12);
			assert_eq!(output1, *b"Hello ");
			assert_eq!(output2, *b"World!");

			// Make sure reading another byte doesn’t work.
			let mut another_byte = [0_u8; 1];
			assert_eq!(
				Pin::new(&mut rx).read(&mut another_byte[..]).await.unwrap(),
				0
			);

			// Now that we have *tried* to read another byte, we should be at end of input.
			assert!(input.is_empty());
		});
	}
}
