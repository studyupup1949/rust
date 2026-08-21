use std::io::{Read, Write};

use crate::error::{AccacheError, Result};

pub const MAGIC: [u8; 4] = *b"ACC1";
pub const FORMAT_VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 4 + 2 + 2 + 4 + 4; // magic + version + flags + count + reserved

pub const FLAG_ZSTD: u16 = 0b0000_0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: u16,
    pub flags: u16,
    pub count: u32,
}

impl Header {
    pub fn new(count: u32, compressed: bool) -> Self {
        let flags = if compressed { FLAG_ZSTD } else { 0 };
        Self {
            magic: MAGIC,
            version: FORMAT_VERSION,
            flags,
            count,
        }
    }

    pub fn is_zstd(&self) -> bool {
        self.flags & FLAG_ZSTD != 0
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_all(&self.magic)?;
        w.write_all(&self.version.to_le_bytes())?;
        w.write_all(&self.flags.to_le_bytes())?;
        w.write_all(&self.count.to_le_bytes())?;
        w.write_all(&[0u8; 4])?; // reserved
        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> Result<Self> {
        let mut buf = [0u8; HEADER_SIZE];
        r.read_exact(&mut buf)
            .map_err(|e| AccacheError::Corrupt(format!("reading header: {e}")))?;
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        if magic != MAGIC {
            return Err(AccacheError::BadMagic { got: magic });
        }
        let version = u16::from_le_bytes(buf[4..6].try_into().unwrap());
        if version != FORMAT_VERSION {
            return Err(AccacheError::UnsupportedFormatVersion(version));
        }
        let flags = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let count = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        Ok(Self {
            magic,
            version,
            flags,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let h = Header::new(42, true);
        let mut buf = Vec::new();
        h.write_to(&mut buf).unwrap();
        assert_eq!(buf.len(), HEADER_SIZE);
        let parsed = Header::read_from(buf.as_slice()).unwrap();
        assert_eq!(h, parsed);
        assert!(parsed.is_zstd());
    }

    #[test]
    fn rejects_bad_magic() {
        let mut buf = vec![0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(b"XXXX");
        match Header::read_from(buf.as_slice()) {
            Err(AccacheError::BadMagic { .. }) => {}
            other => panic!("expected BadMagic, got {other:?}"),
        }
    }

    #[test]
    fn rejects_future_version() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&999u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 4]);
        match Header::read_from(buf.as_slice()) {
            Err(AccacheError::UnsupportedFormatVersion(999)) => {}
            other => panic!("expected UnsupportedFormatVersion, got {other:?}"),
        }
    }
}
