
use anyhow::{Context, Result};
use extsort_iter::*;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"FLANKDB\0";
const VERSION: u32 = 2;

#[derive(Clone)]
pub struct FlankingRecord {

    pub gene: String,

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

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();

    let temp_dir = tempfile::Builder::new()
        .prefix("fdb_sort_")
        .tempdir()
        .context("Failed to create temp directory")?;

    eprintln!("  Temp dir: {}", temp_dir.path().display());

    eprintln!("\n[Phase 1] Reading TSV and external sorting...");

    let file = File::open(tsv_path).context("Failed to open TSV file")?;
    let file_size = file.metadata()?.len();
    let reader = BufReader::with_capacity(8 * 1024 * 1024, file);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty TSV file"))??;

    let record_iter = lines.filter_map(|line_result| {
        let line = line_result.ok()?;
        if line.is_empty() {
            return None;
        }

        let gene = line.split('\t').next()?.to_string();
        if gene.is_empty() {
            return None;
        }
        Some(FlankingRecord { gene, line })
    });

    let buffer_bytes = buffer_size_mb * 1024 * 1024;
    let config = ExtsortConfig::with_buffer_size(buffer_bytes)
        .compress_lz4_flex();

    let sorted_iter = record_iter
        .par_external_sort(config)
        .context("External sort failed")?;

    eprintln!("[Phase 2] Building FDB from sorted records...");

    let mut output = BufWriter::with_capacity(4 * 1024 * 1024, File::create(fdb_path)?);

    output.write_all(MAGIC)?;
    output.write_all(&VERSION.to_le_bytes())?;
    output.write_all(&0u32.to_le_bytes())?;
    output.write_all(&0u64.to_le_bytes())?;

    let mut index_entries: Vec<(String, u64, u32, u32)> = Vec::new();
    let mut compressor = zstd::bulk::Compressor::new(3)?;

    let mut current_gene: Option<String> = None;
    let mut current_records: Vec<String> = Vec::new();
    let mut total_records = 0u64;
    let mut gene_count = 0u32;

    for record in sorted_iter {
        total_records += 1;

        if current_gene.as_ref() != Some(&record.gene) {

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

    output.seek(SeekFrom::Start(12))?;
    output.write_all(&gene_count.to_le_bytes())?;
    output.write_all(&index_offset.to_le_bytes())?;
    output.flush()?;

    drop(temp_dir);

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

fn write_gene_block(
    output: &mut BufWriter<File>,
    compressor: &mut zstd::bulk::Compressor<'_>,
    header: &str,
    gene: &str,
    records: &[String],
    index_entries: &mut Vec<(String, u64, u32, u32)>,
) -> Result<()> {

    let mut content = String::with_capacity(records.len() * 3000);
    content.push_str(header);
    content.push('\n');
    for record in records {
        content.push_str(record);
        content.push('\n');
    }

    let compressed = compressor.compress(content.as_bytes())?;
    let offset = output.stream_position()?;
    output.write_all(&compressed)?;

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
