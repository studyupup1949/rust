use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use bincode::Options;

use crate::error::{AccacheError, Result};
use crate::format::compression::CompressedReader;
use crate::format::entry::Entry;
use crate::format::header::Header;

/// Streaming reader for `.acc` files.
pub struct AccReader<R: Read> {
    pub header: Header,
    payload: CompressedReader<R>,
}

impl<R: Read> AccReader<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let header = Header::read_from(&mut inner)?;
        let payload = CompressedReader::new(inner, header.is_zstd())?;
        Ok(Self { header, payload })
    }

    pub fn read_entries(mut self) -> Result<Vec<Entry>> {
        let opts = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_little_endian();
        let entries: Vec<Entry> = opts
            .deserialize_from(&mut self.payload)
            .map_err(AccacheError::Bincode)?;
        if entries.len() as u32 != self.header.count {
            return Err(AccacheError::Corrupt(format!(
                "header count {} != decoded entries {}",
                self.header.count,
                entries.len(),
            )));
        }
        Ok(entries)
    }
}

/// One-shot reader: load every entry from the file at `path`.
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<(Header, Vec<Entry>)> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|e| AccacheError::InvalidFile {
        path: path.to_path_buf(),
        reason: format!("open: {e}"),
    })?;
    let reader = AccReader::new(BufReader::new(file)).map_err(|e| match e {
        AccacheError::Io(io) => AccacheError::InvalidFile {
            path: path.to_path_buf(),
            reason: format!("io: {io}"),
        },
        other => other,
    })?;
    let header = reader.header;
    let entries = reader.read_entries()?;
    Ok((header, entries))
}
