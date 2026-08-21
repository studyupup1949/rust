//! FDB (Flanking Database) Format Module
//!
//! Converts TSV flanking sequences to compressed binary FDB format.
//! Uses external sorting with LZ4 compression for temporary files
//! and zstd compression for final gene blocks.
//!
//! # File Format
//! ```text
//! [Header]
//!   - Magic: "FLANKDB\0" (8 bytes)
//!   - Version: u32 (4 bytes)
//!   - Gene count: u32 (4 bytes)
//!   - Index offset: u64 (8 bytes)
//! [Gene Blocks]
//!   - Zstd-compressed TSV data per gene
//! [Index]
//!   - Gene name → (offset, compressed_len, record_count)
//! ```
//!
//! # Pipeline
//! 1. Read TSV line by line → FlankingRecord
//! 2. External sort by gene name (parallel, LZ4-compressed temp files)
//! 3. Stream sorted records → group by gene → zstd compress → write FDB
//!
//! # Performance
//! - Memory-efficient: External sort handles files larger than RAM
//! - Compression: ~10x reduction in file size

use anyhow::{Context, Result};
use extsort_iter::*;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"FLANKDB\0";
const VERSION: u32 = 2; // Version 2: new TSV column format

/// Flanking record for sorting
/// Keep this compact - extsort-iter buffers these in memory
#[derive(Clone)]
pub struct FlankingRecord {
    /// Gene name (sort key)
    pub gene: String,
    /// Full TSV line (excluding header)
    pub line: String,
}

impl PartialEq for FlankingRecord {
    fn eq(&self, other: &Self) -> bool {
        self.gene == other.gene
    }
}

impl Eq for FlankingRecord {}

impl PartialOrd for FlankingRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FlankingRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gene.cmp(&other.gene)
    }
}

/// Builds an FDB file from TSV input.
///
/// Performs external sorting and zstd compression to create
/// an indexed, compressed flanking database.
///
/// # Arguments
/// * `tsv_path` - Input TSV file (Gene\tContig\tGenus\tStart\tEnd\tUpstream\tDownstream)
/// * `fdb_path` - Output FDB file path
/// * `buffer_size_mb` - Memory buffer size in MB for external sort (default: 1024)
/// * `threads` - Number of threads for parallel sorting
///
/// # Example
/// ```no_run
/// use argenus::fdb;
/// use std::path::Path;
///
/// fdb::build(
///     Path::new("flanking.tsv"),
///     Path::new("flanking.fdb"),
///     1024,  // 1GB buffer
///     8,     // 8 threads
/// ).unwrap();
/// ```
pub fn build(
    tsv_path: &Path,
    fdb_path: &Path,
    buffer_size_mb: usize,
    threads: usize,
) -> Result<()> {
    eprintln!("Building FDB from TSV...");
    eprintln!("  Input: {}", tsv_path.display());
    eprintln!("  Output: {}", fdb_path.display());
    eprintln!("  Buffer: {} MB, Threads: {}", buffer_size_mb, threads);

    // Set rayon thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok(); // Ignore if already set

    // Create temp directory for sort runs
    let temp_dir = tempfile::Builder::new()
        .prefix("fdb_sort_")
        .tempdir()
        .context("Failed to create temp directory")?;

    eprintln!("  Temp dir: {}", temp_dir.path().display());

    // Phase 1: Read TSV and create record iterator
    eprintln!("\n[Phase 1] Reading TSV and external sorting...");

    let file = File::open(tsv_path).context("Failed to open TSV file")?;
    let file_size = file.metadata()?.len();
    let reader = BufReader::with_capacity(8 * 1024 * 1024, file);
    let mut lines = reader.lines();

    // Read and save header
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty TSV file"))??;

    // Create record iterator
    let record_iter = lines.filter_map(|line_result| {
        let line = line_result.ok()?;
        if line.is_empty() {
            return None;
        }
        // Extract gene name (first column)
        let gene = line.split('\t').next()?.to_string();
        if gene.is_empty() {
            return None;
        }
        Some(FlankingRecord { gene, line })
    });

    // Configure external sort
    // Buffer size in bytes, with LZ4 compression for temp files
    let buffer_bytes = buffer_size_mb * 1024 * 1024;
    let config = ExtsortConfig::with_buffer_size(buffer_bytes)
        .compress_lz4_flex();

    // Perform parallel external sort
    let sorted_iter = record_iter
        .par_external_sort(config)
        .context("External sort failed")?;

    // Phase 2: Build FDB from sorted records
    eprintln!("[Phase 2] Building FDB from sorted records...");

    let mut output = BufWriter::with_capacity(4 * 1024 * 1024, File::create(fdb_path)?);

    // Write header placeholder
    output.write_all(MAGIC)?;
    output.write_all(&VERSION.to_le_bytes())?;
    output.write_all(&0u32.to_le_bytes())?; // gene_count placeholder
    output.write_all(&0u64.to_le_bytes())?; // index_offset placeholder

    // Index entries: (gene, offset, compressed_len, record_count)
    let mut index_entries: Vec<(String, u64, u32, u32)> = Vec::new();
    let mut compressor = zstd::bulk::Compressor::new(3)?;

    let mut current_gene: Option<String> = None;
    let mut current_records: Vec<String> = Vec::new();
    let mut total_records = 0u64;
    let mut gene_count = 0u32;

    for record in sorted_iter {
        total_records += 1;

        if current_gene.as_ref() != Some(&record.gene) {
            // Write previous gene block if exists
            if let Some(prev_gene) = current_gene.take() {
                write_gene_block(
                    &mut output,
                    &mut compressor,
                    &header,
                    &prev_gene,
                    &current_records,
                    &mut index_entries,
                )?;
                gene_count += 1;

                if gene_count.is_multiple_of(1000) {
                    eprintln!(
                        "  Processed {} genes, {} records...",
                        gene_count, total_records
                    );
                }
            }

            current_gene = Some(record.gene);
            current_records.clear();
        }

        current_records.push(record.line);
    }

    // Write last gene block
    if let Some(gene) = current_gene {
        write_gene_block(
            &mut output,
            &mut compressor,
            &header,
            &gene,
            &current_records,
            &mut index_entries,
        )?;
        gene_count += 1;
    }

    eprintln!("  Total: {} genes, {} records", gene_count, total_records);

    // Phase 3: Write index
    eprintln!("[Phase 3] Writing index...");

    let index_offset = output.stream_position()?;

    for (gene, offset, comp_len, record_count) in &index_entries {
        let gene_bytes = gene.as_bytes();
        output.write_all(&(gene_bytes.len() as u16).to_le_bytes())?;
        output.write_all(gene_bytes)?;
        output.write_all(&offset.to_le_bytes())?;
        output.write_all(&comp_len.to_le_bytes())?;
        output.write_all(&record_count.to_le_bytes())?;
    }

    // Update header with actual values
    output.seek(SeekFrom::Start(12))?;
    output.write_all(&gene_count.to_le_bytes())?;
    output.write_all(&index_offset.to_le_bytes())?;
    output.flush()?;

    // Cleanup temp directory (automatically done by tempfile)
    drop(temp_dir);

    // Summary
    let input_size = file_size;
    let output_size = std::fs::metadata(fdb_path)?.len();
    let ratio = input_size as f64 / output_size as f64;

    eprintln!("\n=== FDB Build Complete ===");
    eprintln!("  Input:  {:.2} MB", input_size as f64 / 1024.0 / 1024.0);
    eprintln!("  Output: {:.2} MB", output_size as f64 / 1024.0 / 1024.0);
    eprintln!("  Compression ratio: {:.1}x", ratio);
    eprintln!("  Genes: {}", gene_count);
    eprintln!("  Records: {}", total_records);

    Ok(())
}

/// Build FDB from a pre-sorted TSV file (streaming, memory-efficient).
///
/// This function reads the TSV line by line without loading everything into memory.
/// The input TSV MUST be sorted by gene name (first column).
///
/// # Arguments
/// * `tsv_path` - Input TSV file (MUST be pre-sorted by gene name)
/// * `fdb_path` - Output FDB file path
pub fn build_from_sorted(
    tsv_path: &Path,
    fdb_path: &Path,
) -> Result<()> {
    eprintln!("Building FDB from pre-sorted TSV (streaming mode)...");
    eprintln!("  Input: {}", tsv_path.display());
    eprintln!("  Output: {}", fdb_path.display());

    let file = File::open(tsv_path).context("Failed to open TSV file")?;
    let file_size = file.metadata()?.len();
    let reader = BufReader::with_capacity(8 * 1024 * 1024, file);
    let mut lines = reader.lines();

    // Read header
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty TSV file"))??;

    // Create output file
    let mut output = BufWriter::with_capacity(4 * 1024 * 1024, File::create(fdb_path)?);

    // Write header placeholder
    output.write_all(MAGIC)?;
    output.write_all(&VERSION.to_le_bytes())?;
    output.write_all(&0u32.to_le_bytes())?; // gene_count placeholder
    output.write_all(&0u64.to_le_bytes())?; // index_offset placeholder

    let mut index_entries: Vec<(String, u64, u32, u32)> = Vec::new();
    let mut compressor = zstd::bulk::Compressor::new(3)?;

    let mut current_gene: Option<String> = None;
    let mut current_records: Vec<String> = Vec::new();
    let mut total_records = 0u64;
    let mut gene_count = 0u32;

    eprintln!("\n[Streaming] Processing sorted TSV...");

    for line_result in lines {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.is_empty() {
            continue;
        }

        // Extract gene name (first column)
        let gene = match line.split('\t').next() {
            Some(g) if !g.is_empty() => g.to_string(),
            _ => continue,
        };

        total_records += 1;

        // Check if we've moved to a new gene
        if current_gene.as_ref() != Some(&gene) {
            // Write previous gene block
            if let Some(prev_gene) = current_gene.take() {
                write_gene_block(
                    &mut output,
                    &mut compressor,
                    &header,
                    &prev_gene,
                    &current_records,
                    &mut index_entries,
                )?;
                gene_count += 1;

                if gene_count % 1000 == 0 {
                    eprintln!(
                        "  Processed {} genes, {} records...",
                        gene_count, total_records
                    );
                }
            }

            current_gene = Some(gene);
            current_records.clear();
        }

        current_records.push(line);
    }

    // Write last gene block
    if let Some(gene) = current_gene {
        write_gene_block(
            &mut output,
            &mut compressor,
            &header,
            &gene,
            &current_records,
            &mut index_entries,
        )?;
        gene_count += 1;
    }

    eprintln!("  Total: {} genes, {} records", gene_count, total_records);

    // Write index
    eprintln!("[Writing index]");
    let index_offset = output.stream_position()?;

    for (gene, offset, comp_len, record_count) in &index_entries {
        let gene_bytes = gene.as_bytes();
        output.write_all(&(gene_bytes.len() as u16).to_le_bytes())?;
        output.write_all(gene_bytes)?;
        output.write_all(&offset.to_le_bytes())?;
        output.write_all(&comp_len.to_le_bytes())?;
        output.write_all(&record_count.to_le_bytes())?;
    }

    // Update header with actual values
    output.seek(SeekFrom::Start(12))?;
    output.write_all(&gene_count.to_le_bytes())?;
    output.write_all(&index_offset.to_le_bytes())?;
    output.flush()?;

    // Summary
    let output_size = std::fs::metadata(fdb_path)?.len();
    let ratio = file_size as f64 / output_size as f64;

    eprintln!("\n=== FDB Build Complete (Streaming) ===");
    eprintln!("  Input:  {:.2} MB", file_size as f64 / 1024.0 / 1024.0);
    eprintln!("  Output: {:.2} MB", output_size as f64 / 1024.0 / 1024.0);
    eprintln!("  Compression ratio: {:.1}x", ratio);
    eprintln!("  Genes: {}", gene_count);
    eprintln!("  Records: {}", total_records);

    Ok(())
}

/// Write a compressed gene block to FDB
fn write_gene_block(
    output: &mut BufWriter<File>,
    compressor: &mut zstd::bulk::Compressor<'_>,
    header: &str,
    gene: &str,
    records: &[String],
    index_entries: &mut Vec<(String, u64, u32, u32)>,
) -> Result<()> {
    // Build block content: header + records
    let mut content = String::with_capacity(records.len() * 3000);
    content.push_str(header);
    content.push('\n');
    for record in records {
        content.push_str(record);
        content.push('\n');
    }

    // Compress with zstd
    let compressed = compressor.compress(content.as_bytes())?;
    let offset = output.stream_position()?;
    output.write_all(&compressed)?;

    // Add to index
    index_entries.push((
        gene.to_string(),
        offset,
        compressed.len() as u32,
        records.len() as u32,
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flanking_record_ordering() {
        let r1 = FlankingRecord {
            gene: "aac(6')".to_string(),
            line: "test1".to_string(),
        };
        let r2 = FlankingRecord {
            gene: "blaTEM".to_string(),
            line: "test2".to_string(),
        };
        assert!(r1 < r2);
    }
}
