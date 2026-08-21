use std::io::{BufWriter, Write};
use std::path::Path;

use bincode::Options;

use crate::config::Compression;
use crate::error::{AccacheError, Result};
use crate::format::compression::CompressedWriter;
use crate::format::entry::Entry;
use crate::format::header::Header;

/// Streaming writer for `.acc` files.
///
/// Buffers entries in memory and writes the full file on [`finish`](Self::finish).
/// The header (which includes the entry count) is written first; the payload is then
/// streamed through an optional zstd encoder.
pub struct AccWriter<W: Write> {
    inner: Option<W>,
    entries: Vec<Entry>,
    compression: Compression,
}

impl<W: Write> AccWriter<W> {
    pub fn new(inner: W, compression: Compression) -> Result<Self> {
        Ok(Self {
            inner: Some(inner),
            entries: Vec::new(),
            compression,
        })
    }

    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn extend<I: IntoIterator<Item = Entry>>(&mut self, iter: I) {
        self.entries.extend(iter);
    }

    pub fn finish(mut self) -> Result<W> {
        let raw = self.inner.take().expect("writer used twice");
        let compressed = matches!(self.compression, Compression::Zstd { .. });

        let mut buf = BufWriter::new(raw);
        let header = Header::new(self.entries.len() as u32, compressed);
        header.write_to(&mut buf)?;

        let mut payload_writer = CompressedWriter::new(buf, self.compression)?;
        let opts = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_little_endian();
        opts.serialize_into(&mut payload_writer, &self.entries)
            .map_err(AccacheError::Bincode)?;
        let buf = payload_writer.finish()?;
        let raw = buf
            .into_inner()
            .map_err(|e| AccacheError::Io(std::io::Error::other(e.to_string())))?;
        Ok(raw)
    }
}

/// One-shot writer: encode `entries` into the `.acc` file at `path`.
///
/// Writes via a temp file in the same directory and atomically renames into place,
/// so a partial/crashed write does not corrupt an existing fixture.
pub fn write_file<P: AsRef<Path>>(
    path: P,
    entries: &[Entry],
    compression: Compression,
) -> Result<()> {
    let path = path.as_ref();
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let tmp = tempfile::NamedTempFile::new_in(parent)?;
    {
        let file = tmp.as_file().try_clone()?;
        let mut w = AccWriter::new(file, compression)?;
        w.extend(entries.iter().cloned());
        let _ = w.finish()?;
    }
    tmp.persist(path).map_err(|e| AccacheError::Io(e.error))?;
    Ok(())
}
