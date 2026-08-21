//! Contains an implementation of accumulating reader.
//!
//! See `AccReader` documentation for more information and examples.

use std::io::{self, BufRead, Read, Write, Seek, SeekFrom};
use std::cmp;
use std::ptr;

/// An accumulating reader which provides `Seek` for any `Read`.
///
/// An accumulating reader wraps an instance of `std::io::Read` trait and provides 
/// implementations of `Read` and `std::io::Seek` which use the wrapped `Read` as a source.
/// If the underlying stream implements `std::io::BufRead`, then the wrapping `AccReader`
/// will also implement `BufRead`.
///
/// This struct keeps an internal buffer which contains everything read so far from the
/// wrapped stream and allows "revisiting" the previously read data through the `Seek` interface.
/// When the user needs to seek beyond what was read from the stream, the accumulating reader
/// will automatically read the necessary number of bytes from the wrapped stream to fulfill
/// the request, if possible.
///
/// Seeking to beyond the end of the underlying stream is not possible and will result in 
/// an error. Seeking using `SeekFrom::End`, naturally, involves buffering the whole underlying 
/// stream, therefore it will either hang with blocking infinite streams like sockets or will fill
/// up all of the available memory with truly infinite streams.
///
/// This struct will buffer all of the underlying stream in order to provide seeking, therefore
/// you should discard it as soon as you don't need it if you are working with large streams
/// of data.
///
/// # Examples
///
/// ```no_run
/// use std::io::{self, Read, Seek, SeekFrom};
///
/// use acc_reader::AccReader;
///
/// let mut ar = AccReader::new(io::stdin());
///
/// // read everything starting from the 12th byte
/// // will panic if the input contains less than 12 bytes
/// ar.seek(SeekFrom::Start(12)).unwrap();
/// let mut input = Vec::new();
/// ar.read_to_end(&mut input).unwrap();
/// ```
///
/// It is also possible to seek starting from the end of stream, but this requires reading
/// the whole stream to the end:
///
/// ```no_run
/// use std::io::{self, Read, Seek, SeekFrom};
///
/// use acc_reader::AccReader;
///
/// let mut ar = AccReader::new(io::stdin());
///
/// // read last 12 bytes
/// // will panic if the input contains less than 12 bytes
/// ar.seek(SeekFrom::End(-12)).unwrap();
/// let mut input = Vec::new();
/// ar.read_to_end(&mut input).unwrap();
/// ```
pub struct AccReader<R: Read> {
    source: R,
    buf: Vec<u8>,
    // invariant: pos <= buf.len()
    pos: usize,
}

impl<R: Read> AccReader<R> {
    /// Creates a new accumulating reader from the provided `BufRead` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// 
    /// use acc_reader::AccReader;
    ///
    /// let input = io::stdin();
    /// let mut ar = AccReader::new(input);
    /// ```
    #[inline]
    pub fn new(source: R) -> AccReader<R> {
        AccReader {
            source: source,
            buf: Vec::new(),
            pos: 0
        }
    }

    /// Unwraps this accumulating reader, returning the underlying `BufRead` instance.
    ///
    /// Note that any accumulated data will be lost.
    /// 
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// use acc_reader::AccReader;
    ///
    /// let input = io::stdin();
    /// let mut ar = AccReader::new(input);
    ///
    /// let input2 = ar.into_inner();
    /// ```
    #[inline]
    pub fn into_inner(self) -> R {
        self.source
    }

    // Read from the stream into the internal buffer as much as possible,
    // but no more than the provided number of bytes.
    // Updates the buffer length to the actual number of bytes read, even
    // in case of errors.
    fn read_up_to(&mut self, n: u64) -> io::Result<()> {
        let old_len = self.buf.len();
        self.buf.reserve(n as usize);
        unsafe { self.buf.set_len(old_len + n as usize); }

        let mut error = None;
        let mut read = 0;
        {
            let mut target = &mut self.buf[old_len..];
            while !target.is_empty() {
                match self.source.read(target) {
                    Ok(0) => break,
                    Ok(n) => { read += n; let tmp = target; target = &mut tmp[n..]; }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => { error = Some(e); break; },
                }
            }
        }
        unsafe { self.buf.set_len(old_len + read as usize); }

        if let Some(e) = error {
            Err(e)
        } else {
            Ok(())
        }
    }
}

impl<R: Read> Read for AccReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let need_to_read = cmp::min(self.buf.len() - self.pos, buf.len());
        if need_to_read > 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.buf.as_ptr().offset(self.pos as isize), 
                    buf.as_mut_ptr(), 
                    need_to_read
                );
            }
            self.pos += need_to_read;
            Ok(need_to_read)
        } else {
            let read = try!(self.source.read(buf));
            let _ = self.buf.write_all(&buf[..read]);
            self.pos += read;
            Ok(read)
        }
    }
}

impl<R: BufRead> BufRead for AccReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let available = self.buf.len() - self.pos;
        if available == 0 {
            let len_to_consume = {
                let bytes = try!(self.source.fill_buf());
                let _ = self.buf.write_all(bytes);
                bytes.len()
            };
            self.source.consume(len_to_consume);
        }
        Ok(&self.buf[self.pos..])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.buf.len());
    }
}

impl<R: Read> Seek for AccReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::End(n) => {
                if n > 0 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "seeking beyond end of stream"))
                } else {
                    // just read everything that's left and seek from that
                    try!(self.source.read_to_end(&mut self.buf));

                    let d = (-n) as u64;
                    if d > self.buf.len() as u64 {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "seeking before the begining of stream"))
                    } else {
                        self.pos = (self.buf.len() as u64 - d) as usize;
                        Ok(self.pos as u64)
                    }
                }
            }
            SeekFrom::Start(n) if n <= self.buf.len() as u64 => {
                self.pos = n as usize;
                Ok(self.pos as u64)
            }
            SeekFrom::Start(n) => { // n > self.buf.len()
                let need_to_read = n - self.buf.len() as u64;
                try!(self.read_up_to(need_to_read));
                if n > self.buf.len() as u64 {  // still not enough
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "seeking beyond end of stream"))
                } else {
                    self.pos = n as usize;
                    Ok(n)
                }
            }
            SeekFrom::Current(0) => { Ok(self.pos as u64) }
            SeekFrom::Current(n) if n < 0 => {
                let d = (-n) as u64;
                if d > self.pos as u64 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "seeking before the beginning of stream"))
                } else {
                    self.pos = (self.pos as u64 - d) as usize;
                    Ok(self.pos as u64)
                }
            }
            SeekFrom::Current(n) => {  // n > 0
                let new_pos = self.pos as u64 + n as u64;
                if new_pos > self.buf.len() as u64 {
                    let need_to_read = new_pos - self.buf.len() as u64;
                    try!(self.read_up_to(need_to_read));
                    if new_pos > self.buf.len() as u64 {  // still not enough
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "seeking beyond end of stream"))
                    } else {
                        self.pos = new_pos as usize;
                        Ok(new_pos)
                    }
                } else {
                    self.pos = new_pos as usize;
                    Ok(self.pos as u64)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufRead, Read, Seek, BufReader, SeekFrom};

    use super::*;

    #[test]
    fn test_acc_reader_read() {
        let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3];
        let mut reader = AccReader::new(inner);

        let mut buf = [0, 0];
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(buf, [5, 6]);
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(buf, [7, 0]);
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(buf, [1, 2]);
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(buf[0], 3);
    }

    #[test]
    fn test_acc_reader_buf_read() {
        let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
        let mut reader = AccReader::new(BufReader::with_capacity(3, inner));

        assert_eq!(reader.fill_buf().ok(), Some(&[5, 6, 7][..]));
        reader.consume(3);
        assert_eq!(reader.fill_buf().ok(), Some(&[0, 1, 2][..]));
        reader.consume(3);
        assert_eq!(reader.fill_buf().ok(), Some(&[3, 4][..]));
        reader.consume(2);
        assert_eq!(reader.fill_buf().ok(), Some(&[][..]));
    }

    #[test]
    fn test_acc_reader_seek() {
        let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
        let mut reader = AccReader::new(inner);

        let mut buf = [0, 0];

        assert_eq!(reader.seek(SeekFrom::Start(2)).unwrap(), 2);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [7, 0]);

        assert_eq!(reader.seek(SeekFrom::Current(-1)).unwrap(), 3);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [0, 1]);

        assert_eq!(reader.seek(SeekFrom::End(-3)).unwrap(), 5);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [2, 3]);

        // seek to beyond the end and to before the start
        assert_eq!(reader.seek(SeekFrom::End(3)).err().unwrap().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(reader.seek(SeekFrom::Current(-128)).err().unwrap().kind(), io::ErrorKind::InvalidInput);

        // seek to the end from the start
        let mut reader = AccReader::new(inner);
        assert_eq!(reader.seek(SeekFrom::Start(inner.len() as u64)).unwrap(), inner.len() as u64);
        assert_eq!(reader.read(&mut buf).unwrap(), 0);

        // seek to beyond the end from the start
        let mut reader = AccReader::new(inner);
        assert_eq!(reader.seek(SeekFrom::Start(128)).err().unwrap().kind(), io::ErrorKind::InvalidInput);
    }
}
