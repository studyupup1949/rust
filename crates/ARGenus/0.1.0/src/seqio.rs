
use anyhow::{Context, Result};
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FastaRecord {

    pub name: String,

    pub seq: String,
}

pub struct FastaReader {
    reader: BufReader<File>,
    line_buf: String,
    current_name: Option<String>,
}

impl FastaReader {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open FASTA: {}", path.as_ref().display()))?;
        let mut reader = Self {
            reader: BufReader::with_capacity(1024 * 1024, file),
            line_buf: String::with_capacity(256),
            current_name: None,
        };

        reader.line_buf.clear();
        if reader.reader.read_line(&mut reader.line_buf)? > 0
            && reader.line_buf.starts_with('>') {
                reader.current_name = Some(
                    reader.line_buf[1..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                );
            }

        Ok(reader)
    }

    pub fn read_next(&mut self) -> Result<Option<FastaRecord>> {
        let name = match self.current_name.take() {
            Some(n) => n,
            None => return Ok(None),
        };

        let mut seq = String::with_capacity(10000);

        loop {
            self.line_buf.clear();
            if self.reader.read_line(&mut self.line_buf)? == 0 {

                break;
            }

            if self.line_buf.starts_with('>') {

                self.current_name = Some(
                    self.line_buf[1..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                );
                break;
            } else {

                seq.push_str(self.line_buf.trim_end());
            }
        }

        Ok(Some(FastaRecord { name, seq }))
    }
}

impl Iterator for FastaReader {
    type Item = Result<FastaRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_next() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FastqRecord {

    pub name: String,

    pub seq: String,

    pub qual: String,
}

pub struct FastqReader<R: Read> {
    reader: BufReader<R>,
    line_buf: String,
}

impl FastqReader<File> {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open FASTQ: {}", path.as_ref().display()))?;
        Ok(Self {
            reader: BufReader::with_capacity(1024 * 1024, file),
            line_buf: String::with_capacity(512),
        })
    }
}

impl FastqReader<MultiGzDecoder<File>> {

    pub fn open_gz<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open FASTQ.gz: {}", path.as_ref().display()))?;
        let decoder = MultiGzDecoder::new(file);
        Ok(Self {
            reader: BufReader::with_capacity(1024 * 1024, decoder),
            line_buf: String::with_capacity(512),
        })
    }
}

impl<R: Read> FastqReader<R> {

    pub fn read_next(&mut self) -> Result<Option<FastqRecord>> {

        self.line_buf.clear();
        if self.reader.read_line(&mut self.line_buf)? == 0 {
            return Ok(None);
        }
        let name = self.line_buf.trim_start_matches('@').trim_end().to_string();
        if name.is_empty() {
            return Ok(None);
        }

        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;
        let seq = self.line_buf.trim_end().to_string();

        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;

        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;
        let qual = self.line_buf.trim_end().to_string();

        Ok(Some(FastqRecord { name, seq, qual }))
    }
}

pub enum FastqFile {

    Plain(FastqReader<File>),

    Gzipped(FastqReader<MultiGzDecoder<File>>),
}

impl FastqFile {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext == "gz" {
            Ok(FastqFile::Gzipped(FastqReader::open_gz(path)?))
        } else {
            Ok(FastqFile::Plain(FastqReader::open(path)?))
        }
    }

    pub fn read_next(&mut self) -> Result<Option<FastqRecord>> {
        match self {
            FastqFile::Plain(r) => r.read_next(),
            FastqFile::Gzipped(r) => r.read_next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fastq_record() {
        let record = FastqRecord {
            name: "read1".to_string(),
            seq: "ATGC".to_string(),
            qual: "IIII".to_string(),
        };
        assert_eq!(record.seq.len(), record.qual.len());
    }
}
