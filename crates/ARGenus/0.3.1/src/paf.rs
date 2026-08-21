//! PAF (Pairwise mApping Format) Parser Module
//!
//! Provides parsing capabilities for minimap2 PAF alignment output.
//! PAF is a text-based format containing alignment information.
//!
//! # PAF Format (12 mandatory columns)
//! ```text
//! Col  Type    Description
//! 1    string  Query sequence name
//! 2    int     Query sequence length
//! 3    int     Query start (0-based)
//! 4    int     Query end
//! 5    char    Relative strand: '+' or '-'
//! 6    string  Target sequence name
//! 7    int     Target sequence length
//! 8    int     Target start
//! 9    int     Target end
//! 10   int     Number of matching bases
//! 11   int     Alignment block length
//! 12   int     Mapping quality (0-255; 255 for missing)
//! ```
//!
//! # Example Usage
//! ```no_run
//! use argenus::paf::PafReader;
//!
//! let mut reader = PafReader::open("alignment.paf").unwrap();
//! while let Some(record) = reader.read_next().unwrap() {
//!     println!("{} -> {} ({:.1}% identity)",
//!              record.query_name, record.target_name, record.calculate_identity());
//! }
//! ```

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// ============================================================================
// PAF Record
// ============================================================================

/// A single PAF alignment record.
///
/// Contains the 12 mandatory PAF columns parsed from a single line.
/// Optional tags (columns 13+) are not currently parsed.
#[derive(Debug, Clone)]
pub struct PafRecord {
    /// Query sequence name (column 1).
    pub query_name: String,
    /// Query sequence length (column 2).
    pub query_len: usize,
    /// Query start position, 0-based (column 3).
    pub query_start: usize,
    /// Query end position (column 4).
    pub query_end: usize,
    /// Relative strand: '+' or '-' (column 5).
    pub strand: char,
    /// Target sequence name (column 6).
    pub target_name: String,
    /// Target sequence length (column 7).
    pub target_len: usize,
    /// Target start position (column 8).
    pub target_start: usize,
    /// Target end position (column 9).
    pub target_end: usize,
    /// Number of matching bases (column 10).
    pub matches: usize,
    /// Alignment block length (column 11).
    pub block_len: usize,
}

impl PafRecord {
    /// Parses a PAF record from a tab-separated line.
    ///
    /// # Arguments
    /// * `line` - A single line from a PAF file
    ///
    /// # Returns
    /// A parsed PafRecord, or an error if the line is malformed.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The line has fewer than 12 fields
    /// - Any numeric field cannot be parsed
    pub fn parse_line(line: &str) -> Result<Self> {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 12 {
            anyhow::bail!("Invalid PAF line: fewer than 12 fields");
        }

        Ok(Self {
            query_name: fields[0].to_string(),
            query_len: fields[1].parse().context("Invalid query length")?,
            query_start: fields[2].parse().context("Invalid query start")?,
            query_end: fields[3].parse().context("Invalid query end")?,
            strand: fields[4].chars().next().unwrap_or('+'),
            target_name: fields[5].to_string(),
            target_len: fields[6].parse().context("Invalid target length")?,
            target_start: fields[7].parse().context("Invalid target start")?,
            target_end: fields[8].parse().context("Invalid target end")?,
            matches: fields[9].parse().context("Invalid matches count")?,
            block_len: fields[10].parse().context("Invalid block length")?,
        })
    }

    /// Calculates alignment identity percentage.
    ///
    /// Identity = (matching bases / alignment block length) × 100
    ///
    /// # Returns
    /// Identity percentage (0-100), or 0 if block_len is 0.
    pub fn calculate_identity(&self) -> f64 {
        if self.block_len == 0 {
            return 0.0;
        }
        (self.matches as f64 / self.block_len as f64) * 100.0
    }

    /// Calculates target coverage percentage.
    ///
    /// Coverage = (aligned target length / total target length) × 100
    ///
    /// # Returns
    /// Coverage percentage (0-100), or 0 if target_len is 0.
    pub fn calculate_coverage(&self) -> f64 {
        if self.target_len == 0 {
            return 0.0;
        }
        ((self.target_end - self.target_start) as f64 / self.target_len as f64) * 100.0
    }
}

// ============================================================================
// PAF Reader
// ============================================================================

/// Sequential reader for PAF format files.
///
/// Provides efficient line-by-line reading with internal buffering.
/// Implements Iterator for convenient use in for loops.
pub struct PafReader {
    reader: BufReader<File>,
    line_buf: String,
}

impl PafReader {
    /// Opens a PAF file for reading.
    ///
    /// # Arguments
    /// * `path` - Path to the PAF file
    ///
    /// # Returns
    /// A new PafReader, or an error if the file cannot be opened.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open PAF: {}", path.as_ref().display()))?;
        Ok(Self {
            reader: BufReader::with_capacity(1024 * 1024, file),
            line_buf: String::with_capacity(512),
        })
    }

    /// Reads the next PAF record from the file.
    ///
    /// Skips empty lines automatically.
    ///
    /// # Returns
    /// - `Ok(Some(record))` - Successfully read a record
    /// - `Ok(None)` - End of file reached
    /// - `Err(e)` - I/O or parsing error
    pub fn read_next(&mut self) -> Result<Option<PafRecord>> {
        self.line_buf.clear();
        if self.reader.read_line(&mut self.line_buf)? == 0 {
            return Ok(None);
        }

        let line = self.line_buf.trim_end();
        if line.is_empty() {
            return self.read_next(); // Skip empty lines
        }

        Ok(Some(PafRecord::parse_line(line)?))
    }
}

impl Iterator for PafReader {
    type Item = Result<PafRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_next() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paf_line() {
        let line = "read1\t150\t10\t140\t+\tgene1\t1000\t100\t230\t120\t130\t60";
        let record = PafRecord::parse_line(line).unwrap();

        assert_eq!(record.query_name, "read1");
        assert_eq!(record.query_len, 150);
        assert_eq!(record.query_start, 10);
        assert_eq!(record.query_end, 140);
        assert_eq!(record.strand, '+');
        assert_eq!(record.target_name, "gene1");
        assert_eq!(record.target_len, 1000);
        assert_eq!(record.matches, 120);
        assert_eq!(record.block_len, 130);
    }

    #[test]
    fn test_calculate_identity() {
        let record = PafRecord {
            query_name: "test".to_string(),
            query_len: 100,
            query_start: 0,
            query_end: 100,
            strand: '+',
            target_name: "ref".to_string(),
            target_len: 100,
            target_start: 0,
            target_end: 100,
            matches: 95,
            block_len: 100,
        };

        assert_eq!(record.calculate_identity(), 95.0);
    }

    #[test]
    fn test_calculate_coverage() {
        let record = PafRecord {
            query_name: "test".to_string(),
            query_len: 100,
            query_start: 0,
            query_end: 100,
            strand: '+',
            target_name: "ref".to_string(),
            target_len: 200,
            target_start: 0,
            target_end: 100,
            matches: 100,
            block_len: 100,
        };

        assert_eq!(record.calculate_coverage(), 50.0);
    }

    #[test]
    fn test_invalid_paf_line() {
        let line = "incomplete\tline";
        assert!(PafRecord::parse_line(line).is_err());
    }
}
