//! Sequence I/O Module
//!
//! Provides unified reading capabilities for biological sequence files.
//! Supports both FASTA and FASTQ formats, including gzip-compressed files.
//!
//! # Supported Formats
//! - FASTA: Standard sequence format with header and sequence lines
//! - FASTQ: Sequence format with quality scores (plain or gzipped)
//!
//! # Examples
//! ```no_run
//! use argenus::seqio::{FastaReader, FastqFile};
//!
//! // Read FASTA file
//! let mut reader = FastaReader::open("sequences.fas").unwrap();
//! while let Some(record) = reader.read_next().unwrap() {
//!     println!("{}: {} bp", record.name, record.seq.len());
//! }
//!
//! // Read FASTQ file (auto-detects gzip)
//! let mut reader = FastqFile::open("reads.fq.gz").unwrap();
//! while let Some(record) = reader.read_next().unwrap() {
//!     println!("{}: {} bp", record.name, record.seq.len());
//! }
//! ```

use anyhow::{Context, Result};
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

// ============================================================================
// FASTA Format
// ============================================================================

/// A FASTA record containing sequence name and nucleotide sequence.
///
/// # Fields
/// - `name`: Sequence identifier (text after '>' up to first whitespace)
/// - `seq`: Nucleotide sequence (concatenated from all sequence lines)
#[derive(Debug, Clone)]
pub struct FastaRecord {
    /// Sequence identifier extracted from the header line.
    pub name: String,
    /// Nucleotide sequence (may contain standard IUPAC codes).
    pub seq: String,
}

/// Sequential reader for FASTA format files.
///
/// Reads records one at a time with minimal memory footprint.
/// Handles multi-line sequences and strips whitespace automatically.
pub struct FastaReader {
    reader: BufReader<File>,
    line_buf: String,
    current_name: Option<String>,
}

impl FastaReader {
    /// Opens a FASTA file for reading.
    ///
    /// # Arguments
    /// * `path` - Path to the FASTA file
    ///
    /// # Returns
    /// A new FastaReader instance, or an error if the file cannot be opened.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open FASTA: {}", path.as_ref().display()))?;
        let mut reader = Self {
            reader: BufReader::with_capacity(1024 * 1024, file),
            line_buf: String::with_capacity(256),
            current_name: None,
        };

        // Read first header line to initialise state
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

    /// Reads the next FASTA record from the file.
    ///
    /// # Returns
    /// - `Ok(Some(record))` - Successfully read a record
    /// - `Ok(None)` - End of file reached
    /// - `Err(e)` - I/O error occurred
    pub fn read_next(&mut self) -> Result<Option<FastaRecord>> {
        let name = match self.current_name.take() {
            Some(n) => n,
            None => return Ok(None),
        };

        let mut seq = String::with_capacity(10000);

        loop {
            self.line_buf.clear();
            if self.reader.read_line(&mut self.line_buf)? == 0 {
                // End of file reached
                break;
            }

            if self.line_buf.starts_with('>') {
                // New record header encountered
                self.current_name = Some(
                    self.line_buf[1..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                );
                break;
            } else {
                // Sequence line - append to current sequence
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

// ============================================================================
// FASTQ Format
// ============================================================================

/// A FASTQ record containing sequence name, nucleotide sequence, and quality scores.
///
/// # Fields
/// - `name`: Read identifier (text after '@')
/// - `seq`: Nucleotide sequence
/// - `qual`: Quality string (Phred+33 encoded)
#[derive(Debug, Clone)]
pub struct FastqRecord {
    /// Read identifier from the header line.
    pub name: String,
    /// Nucleotide sequence.
    pub seq: String,
    /// Quality scores (same length as seq, Phred+33 encoded).
    pub qual: String,
}

/// Generic FASTQ reader supporting any Read source.
///
/// Use `FastqReader<File>` for plain files or
/// `FastqReader<MultiGzDecoder<File>>` for gzipped files.
pub struct FastqReader<R: Read> {
    reader: BufReader<R>,
    line_buf: String,
}

impl FastqReader<File> {
    /// Opens a plain (uncompressed) FASTQ file.
    ///
    /// # Arguments
    /// * `path` - Path to the FASTQ file
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
    /// Opens a gzip-compressed FASTQ file.
    ///
    /// # Arguments
    /// * `path` - Path to the .fastq.gz or .fq.gz file
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
    /// Reads the next FASTQ record (4 lines per record).
    ///
    /// # FASTQ Format
    /// ```text
    /// @read_name
    /// SEQUENCE
    /// +
    /// QUALITY
    /// ```
    ///
    /// # Returns
    /// - `Ok(Some(record))` - Successfully read a record
    /// - `Ok(None)` - End of file reached
    /// - `Err(e)` - I/O or format error
    pub fn read_next(&mut self) -> Result<Option<FastqRecord>> {
        // Line 1: @name
        self.line_buf.clear();
        if self.reader.read_line(&mut self.line_buf)? == 0 {
            return Ok(None);
        }
        let name = self.line_buf.trim_start_matches('@').trim_end().to_string();
        if name.is_empty() {
            return Ok(None);
        }

        // Line 2: sequence
        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;
        let seq = self.line_buf.trim_end().to_string();

        // Line 3: + (separator, ignored)
        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;

        // Line 4: quality scores
        self.line_buf.clear();
        self.reader.read_line(&mut self.line_buf)?;
        let qual = self.line_buf.trim_end().to_string();

        Ok(Some(FastqRecord { name, seq, qual }))
    }
}

/// Auto-detecting FASTQ file reader.
///
/// Automatically selects plain or gzip reader based on file extension.
/// Files ending in `.gz` are treated as gzip-compressed.
pub enum FastqFile {
    /// Plain text FASTQ file.
    Plain(FastqReader<File>),
    /// Gzip-compressed FASTQ file.
    Gzipped(FastqReader<MultiGzDecoder<File>>),
}

impl FastqFile {
    /// Opens a FASTQ file with automatic compression detection.
    ///
    /// # Arguments
    /// * `path` - Path to FASTQ file (plain or .gz)
    ///
    /// # Compression Detection
    /// Files with `.gz` extension are opened with gzip decompression.
    /// All other files are opened as plain text.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext == "gz" {
            Ok(FastqFile::Gzipped(FastqReader::open_gz(path)?))
        } else {
            Ok(FastqFile::Plain(FastqReader::open(path)?))
        }
    }

    /// Reads the next FASTQ record.
    ///
    /// Delegates to the appropriate reader based on file type.
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
