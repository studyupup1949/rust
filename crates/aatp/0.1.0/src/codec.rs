//! Ergonomic, bounds-checked cursors for AATP payloads — the safe alternative to
//! hand-indexing bytes. [`Writer`] appends typed fields into a caller-owned buffer;
//! [`Reader`] pulls them back, borrowing strings and byte slices **zero-copy**. Both
//! fail closed (never panic, never over-read). The [`aatp_message!`](crate::aatp_message)
//! macro composes these into a whole-struct codec, so a message type gets `encode` /
//! `decode` for free — the derive-style ergonomics of the field, with zero dependencies.

use core::str;

/// The largest a single length-prefixed `bytes`/`str` field may be — the guard against a
/// hostile length asking us to address gigabytes. Keeps every offset inside a 32-bit `usize`.
pub const MAX_FIELD: usize = 16 * 1024 * 1024;

/// The single length guard for a `bytes`/`str` field, shared by [`Writer::bytes`] and
/// [`Reader::bytes`] so the boundary is pinned once — in a pure function a test can hit
/// without allocating [`MAX_FIELD`] bytes. `len == MAX_FIELD` is allowed; one more is refused.
fn check_field_len(len: usize) -> Result<(), CodecError> {
    if len > MAX_FIELD {
        Err(CodecError::TooLong(len))
    } else {
        Ok(())
    }
}

/// What can go wrong on a cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    /// A [`Writer`] ran out of room in its buffer.
    OutOfSpace,
    /// A [`Reader`] ran past the end of its input.
    Truncated,
    /// A `str` field was not valid UTF-8.
    BadUtf8,
    /// A length-prefixed field declared more than [`MAX_FIELD`] bytes.
    TooLong(usize),
}

/// Appends typed fields into a caller-owned buffer, little-endian, tracking the write
/// position. Allocation-free.
pub struct Writer<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Writer<'a> {
    /// A writer over `buf`, positioned at the start.
    pub fn new(buf: &'a mut [u8]) -> Writer<'a> {
        Writer { buf, pos: 0 }
    }

    /// Bytes written so far — the frame length once every field is in.
    pub fn position(&self) -> usize {
        self.pos
    }

    fn raw(&mut self, bytes: &[u8]) -> Result<(), CodecError> {
        let end = self.pos + bytes.len();
        let slot = self
            .buf
            .get_mut(self.pos..end)
            .ok_or(CodecError::OutOfSpace)?;
        slot.copy_from_slice(bytes);
        self.pos = end;
        Ok(())
    }

    /// Write a `u8`.
    pub fn u8(&mut self, v: u8) -> Result<(), CodecError> {
        self.raw(&[v])
    }
    /// Write a `u16` (little-endian).
    pub fn u16(&mut self, v: u16) -> Result<(), CodecError> {
        self.raw(&v.to_le_bytes())
    }
    /// Write a `u32` (little-endian).
    pub fn u32(&mut self, v: u32) -> Result<(), CodecError> {
        self.raw(&v.to_le_bytes())
    }
    /// Write a `u64` (little-endian).
    pub fn u64(&mut self, v: u64) -> Result<(), CodecError> {
        self.raw(&v.to_le_bytes())
    }
    /// Write a length-prefixed byte slice (`u32` length + bytes). Rejects a field over
    /// [`MAX_FIELD`], so the `as u32` length cast can never truncate.
    pub fn bytes(&mut self, b: &[u8]) -> Result<(), CodecError> {
        check_field_len(b.len())?;
        self.u32(b.len() as u32)?;
        self.raw(b)
    }
    /// Write a length-prefixed string (as its UTF-8 bytes).
    pub fn str(&mut self, s: &str) -> Result<(), CodecError> {
        self.bytes(s.as_bytes())
    }
}

/// Pulls typed fields back out of a buffer, borrowing `str`/`bytes` **zero-copy** from it.
pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// A reader over `buf`, positioned at the start.
    pub fn new(buf: &'a [u8]) -> Reader<'a> {
        Reader { buf, pos: 0 }
    }

    /// Bytes consumed so far.
    pub fn position(&self) -> usize {
        self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], CodecError> {
        let end = self.pos + n;
        let slice = self.buf.get(self.pos..end).ok_or(CodecError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    /// Read a `u8`.
    pub fn u8(&mut self) -> Result<u8, CodecError> {
        Ok(self.take(1)?[0])
    }
    /// Read a `u16` (little-endian).
    pub fn u16(&mut self) -> Result<u16, CodecError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    /// Read a `u32` (little-endian).
    pub fn u32(&mut self) -> Result<u32, CodecError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    /// Read a `u64` (little-endian).
    pub fn u64(&mut self) -> Result<u64, CodecError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    /// Read a length-prefixed byte slice, borrowed from the input (zero-copy).
    pub fn bytes(&mut self) -> Result<&'a [u8], CodecError> {
        let len = self.u32()? as usize;
        check_field_len(len)?;
        self.take(len)
    }
    /// Read a length-prefixed string, borrowed and UTF-8-validated in place.
    pub fn str(&mut self) -> Result<&'a str, CodecError> {
        str::from_utf8(self.bytes()?).map_err(|_| CodecError::BadUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_field_cap_and_its_guard_are_pinned() {
        assert_eq!(MAX_FIELD, 16_777_216); // pins the constant's value (16 MiB)
        assert_eq!(check_field_len(MAX_FIELD), Ok(())); // the cap itself is allowed
        assert_eq!(
            check_field_len(MAX_FIELD + 1),
            Err(CodecError::TooLong(MAX_FIELD + 1))
        );
    }

    #[test]
    fn writer_then_reader_round_trips_every_scalar_and_field_type() {
        let mut buf = [0u8; 128];
        let mut w = Writer::new(&mut buf);
        w.u8(0xAB).unwrap();
        w.u16(0x1234).unwrap();
        w.u32(0xDEAD_BEEF).unwrap();
        w.u64(0x0102_0304_0506_0708).unwrap();
        w.bytes(&[1, 2, 3]).unwrap();
        w.str("héllo").unwrap();
        let n = w.position();

        let mut r = Reader::new(&buf[..n]);
        assert_eq!(r.u8().unwrap(), 0xAB);
        assert_eq!(r.u16().unwrap(), 0x1234);
        assert_eq!(r.u32().unwrap(), 0xDEAD_BEEF);
        assert_eq!(r.u64().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(r.bytes().unwrap(), &[1, 2, 3]);
        assert_eq!(r.str().unwrap(), "héllo");
        assert_eq!(r.position(), n); // consumed exactly what was written
    }

    #[test]
    fn writer_lays_down_the_exact_little_endian_bytes() {
        let mut buf = [0u8; 16];
        let mut w = Writer::new(&mut buf);
        w.u16(0x0102).unwrap();
        w.u32(0x0304_0506).unwrap();
        assert_eq!(w.position(), 6);
        assert_eq!(&buf[..2], &[0x02, 0x01]); // u16 LE
        assert_eq!(&buf[2..6], &[0x06, 0x05, 0x04, 0x03]); // u32 LE
    }

    #[test]
    fn bytes_field_is_length_prefixed() {
        let mut buf = [0u8; 16];
        let mut w = Writer::new(&mut buf);
        w.bytes(&[0xAA, 0xBB]).unwrap();
        let n = w.position(); // last use of the writer — releases the &mut buf borrow
        assert_eq!(n, 6);
        assert_eq!(&buf[..4], &2u32.to_le_bytes()); // length prefix
        assert_eq!(&buf[4..6], &[0xAA, 0xBB]); // the bytes
    }

    #[test]
    fn writer_rejects_a_field_that_does_not_fit_at_the_boundary() {
        let mut four = [0u8; 4];
        let mut w = Writer::new(&mut four);
        w.u32(1).unwrap(); // exactly fills it
        assert_eq!(w.u8(0), Err(CodecError::OutOfSpace)); // one more overflows
        // a u64 into a 7-byte buffer fails; into 8 succeeds.
        let mut seven = [0u8; 7];
        assert_eq!(Writer::new(&mut seven).u64(0), Err(CodecError::OutOfSpace));
        let mut eight = [0u8; 8];
        assert_eq!(Writer::new(&mut eight).u64(0), Ok(()));
    }

    #[test]
    fn reader_rejects_a_truncated_field_at_the_boundary() {
        assert_eq!(Reader::new(&[]).u8(), Err(CodecError::Truncated));
        assert_eq!(Reader::new(&[0]).u16(), Err(CodecError::Truncated)); // needs 2
        assert_eq!(Reader::new(&[0, 0, 0]).u32(), Err(CodecError::Truncated)); // needs 4
        assert_eq!(Reader::new(&[0; 7]).u64(), Err(CodecError::Truncated)); // needs 8
        // a bytes field whose length prefix overruns the buffer
        let mut framed = [0u8; 6];
        Writer::new(&mut framed).bytes(&[1, 2]).unwrap();
        assert_eq!(
            Reader::new(&framed[..5]).bytes(),
            Err(CodecError::Truncated)
        );
    }

    #[test]
    fn reader_rejects_a_str_field_that_is_not_utf8() {
        let mut buf = [0u8; 8];
        let mut w = Writer::new(&mut buf);
        w.bytes(&[0xFF]).unwrap(); // 0xFF is not valid UTF-8
        let n = w.position();
        assert_eq!(Reader::new(&buf[..n]).str(), Err(CodecError::BadUtf8));
    }

    #[test]
    fn over_long_length_prefix_is_refused_before_addressing() {
        // craft a bytes field whose u32 length claims MAX_FIELD+1
        let mut buf = [0u8; 8];
        buf[..4].copy_from_slice(&((MAX_FIELD as u32) + 1).to_le_bytes());
        assert_eq!(
            Reader::new(&buf).bytes(),
            Err(CodecError::TooLong(MAX_FIELD + 1))
        );
    }

    #[test]
    fn empty_bytes_and_str_round_trip() {
        let mut buf = [0u8; 16];
        let mut w = Writer::new(&mut buf);
        w.bytes(&[]).unwrap();
        w.str("").unwrap();
        let n = w.position();
        let mut r = Reader::new(&buf[..n]);
        assert_eq!(r.bytes().unwrap(), &[] as &[u8]);
        assert_eq!(r.str().unwrap(), "");
    }
}
