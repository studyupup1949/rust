//! Genus Classifier Module
//!
//! Classifies the source genus of detected ARGs using flanking sequence analysis.
//! Compares extracted flanking regions against a pre-built database of known
//! gene-genus associations.
//!
//! # Classification Method
//! 1. Extract upstream and downstream flanking sequences from contig
//! 2. Query the flanking database for the detected ARG
//! 3. Align query flanking sequences against reference flanking sequences
//! 4. Score genus candidates based on alignment identity and coverage
//! 5. Report top genus with confidence and specificity metrics
//!
//! # Key Metrics
//! - **Confidence**: Alignment identity score (0-100%)
//! - **Specificity**: Gene-genus association strength in the database (0-100%)

use anyhow::{Context, Result};
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::Command;

use crate::snp::{self, SnpStatus};

const FDB_MAGIC: &[u8; 8] = b"FLANKDB\0";

// ============================================================================
// Data Structures
// ============================================================================

/// ARG hit with position information for flanking extraction.
///
/// Contains all information needed to extract and classify flanking sequences.
#[derive(Debug, Clone)]
pub struct ArgPosition {
    /// ARG gene name (e.g., "blaTEM-1").
    pub arg_name: String,
    /// Contig identifier where ARG was detected.
    pub contig_name: String,
    /// Full contig nucleotide sequence.
    pub contig_seq: String,
    /// Contig length in base pairs.
    pub contig_len: usize,
    /// ARG start position on contig (0-based).
    pub arg_start: usize,
    /// ARG end position on contig.
    pub arg_end: usize,
    /// Strand orientation ('+' or '-').
    pub strand: char,
}

/// Genus classification result for a single ARG.
#[derive(Debug, Clone)]
pub struct GenusResult {
    /// ARG gene name.
    pub arg_name: String,
    /// Contig identifier.
    pub contig_name: String,
    /// Classified genus (None if unresolved).
    pub genus: Option<String>,
    /// Classification confidence (alignment identity, 0-100).
    pub confidence: f64,
    /// Gene-genus specificity in database (0-100).
    pub specificity: f64,
    /// Extracted upstream flanking length.
    pub upstream_len: usize,
    /// Extracted downstream flanking length.
    pub downstream_len: usize,
    /// Top genus matches with scores: [(genus, score), ...].
    pub top_matches: Vec<(String, f64)>,
    /// SNP verification status (for point mutation ARGs).
    pub snp_status: SnpStatus,
}

impl Default for GenusResult {
    fn default() -> Self {
        Self {
            arg_name: String::new(),
            contig_name: String::new(),
            genus: None,
            confidence: 0.0,
            specificity: 0.0,
            upstream_len: 0,
            downstream_len: 0,
            top_matches: vec![],
            snp_status: SnpStatus::NotApplicable,
        }
    }
}

/// Flanking database record from FDB file.
///
/// Represents a single flanking sequence entry in the database.
#[derive(Debug, Clone)]
pub struct FlankingRecord {
    /// Source contig identifier.
    pub contig: String,
    /// Source organism genus.
    pub genus: String,
    /// Upstream flanking sequence.
    pub upstream: String,
    /// Downstream flanking sequence.
    pub downstream: String,
}

// ============================================================================
// FDB Index Entry
// ============================================================================

/// Index entry for compressed gene blocks in FDB format.
#[derive(Debug, Clone)]
struct FdbIndexEntry {
    offset: u64,
    compressed_len: u32,
    record_count: u32,
}

// ============================================================================
// Flanking Database Reader
// ============================================================================

/// Reader for compressed flanking database (.fdb) files.
///
/// The FDB format stores flanking sequences grouped by gene,
/// with zstd compression for each gene block.
///
/// # File Format
/// ```text
/// [Header: 8 bytes magic + 4 bytes version + 4 bytes gene_count + 8 bytes index_offset]
/// [Gene blocks: zstd compressed TSV data]
/// [Index: gene name -> (offset, compressed_len, record_count)]
/// ```
pub struct FlankingDatabase {
    file: File,
    index: FxHashMap<String, FdbIndexEntry>,
    /// Maps gene name (first field before '|') to full key
    /// Enables lookup by just gene name when FDB uses full header format (e.g., "mexQ|DRUG|CLASS|CODE")
    gene_name_to_key: FxHashMap<String, String>,
}

impl FlankingDatabase {
    /// Opens a flanking database file.
    ///
    /// Reads and validates the header, then loads the gene index into memory.
    ///
    /// # Arguments
    /// * `path` - Path to the .fdb file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open fdb: {}", path.as_ref().display()))?;

        // Read and verify header
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic != FDB_MAGIC {
            anyhow::bail!("Invalid fdb magic");
        }

        let mut buf4 = [0u8; 4];
        let mut buf8 = [0u8; 8];

        file.read_exact(&mut buf4)?;
        let _version = u32::from_le_bytes(buf4);

        file.read_exact(&mut buf4)?;
        let gene_count = u32::from_le_bytes(buf4);

        file.read_exact(&mut buf8)?;
        let index_offset = u64::from_le_bytes(buf8);

        // Read index from end of file
        file.seek(SeekFrom::Start(index_offset))?;
        let mut index = FxHashMap::default();

        for _ in 0..gene_count {
            let mut buf2 = [0u8; 2];
            file.read_exact(&mut buf2)?;
            let name_len = u16::from_le_bytes(buf2) as usize;

            let mut name_buf = vec![0u8; name_len];
            file.read_exact(&mut name_buf)?;
            let gene = String::from_utf8(name_buf)?;

            file.read_exact(&mut buf8)?;
            let offset = u64::from_le_bytes(buf8);

            file.read_exact(&mut buf4)?;
            let compressed_len = u32::from_le_bytes(buf4);

            file.read_exact(&mut buf4)?;
            let record_count = u32::from_le_bytes(buf4);

            index.insert(gene, FdbIndexEntry {
                offset,
                compressed_len,
                record_count,
            });
        }

        // Build gene_name -> full_key mapping for flexible lookup
        // This handles cases where FDB keys are "gene|class|class|code" but lookup uses just "gene"
        let mut gene_name_to_key = FxHashMap::default();
        for full_key in index.keys() {
            // Extract gene name (first field before '|')
            let gene_name = full_key.split('|').next().unwrap_or(full_key);
            // Only add if not already present (first match wins)
            if !gene_name_to_key.contains_key(gene_name) {
                gene_name_to_key.insert(gene_name.to_string(), full_key.clone());
            }
        }

        Ok(Self { file, index, gene_name_to_key })
    }

    /// Checks if a gene exists in the database.
    /// First tries direct key lookup, then falls back to gene name mapping.
    pub fn has_gene(&self, gene: &str) -> bool {
        // Try direct lookup first
        if self.index.contains_key(gene) {
            return true;
        }
        // Fall back to gene name mapping (for "gene" -> "gene|class|class|code" lookup)
        self.gene_name_to_key.contains_key(gene)
    }

    /// Resolves a gene name to its full key in the index.
    /// Returns the original gene if it's already a valid key, or the mapped full key.
    fn resolve_gene_key(&self, gene: &str) -> Option<&String> {
        if self.index.contains_key(gene) {
            // Gene is already the full key
            None // Signal to use gene directly
        } else {
            // Try gene name mapping
            self.gene_name_to_key.get(gene)
        }
    }

    /// Retrieves all flanking records for a specific gene.
    ///
    /// Decompresses the gene block on demand.
    /// Supports both direct key lookup and gene name mapping (e.g., "mexQ" -> "mexQ|DRUG|CLASS|CODE").
    pub fn get_gene_records(&mut self, gene: &str) -> Result<Vec<FlankingRecord>> {
        // Try direct lookup, then gene name mapping
        let lookup_key = if self.index.contains_key(gene) {
            gene.to_string()
        } else if let Some(full_key) = self.gene_name_to_key.get(gene) {
            full_key.clone()
        } else {
            anyhow::bail!("Gene not found: {}", gene);
        };

        let entry = self.index.get(&lookup_key)
            .ok_or_else(|| anyhow::anyhow!("Gene not found in index: {}", lookup_key))?
            .clone();

        // Read compressed block
        self.file.seek(SeekFrom::Start(entry.offset))?;
        let mut compressed = vec![0u8; entry.compressed_len as usize];
        self.file.read_exact(&mut compressed)?;

        // Decompress with zstd
        let decompressed = zstd::decode_all(&compressed[..])?;
        let content = String::from_utf8(decompressed)?;

        // Parse TSV content
        let mut records = Vec::with_capacity(entry.record_count as usize);
        let mut lines = content.lines();

        // Skip header line
        let _header = lines.next();

        for line in lines {
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            // Format: Gene | Contig | Genus | Start | End | Upstream | Downstream
            if fields.len() < 7 {
                continue;
            }

            records.push(FlankingRecord {
                contig: fields[1].to_string(),
                genus: fields[2].to_string(),
                upstream: fields[5].to_string(),
                downstream: fields[6].to_string(),
            });
        }

        Ok(records)
    }

    /// Computes genus distribution for a gene.
    ///
    /// Returns a map of genus → occurrence count.
    pub fn get_genus_distribution(&mut self, gene: &str) -> Result<FxHashMap<String, usize>> {
        let records = self.get_gene_records(gene)?;
        let mut dist: FxHashMap<String, usize> = FxHashMap::default();

        for rec in records {
            *dist.entry(rec.genus).or_default() += 1;
        }

        Ok(dist)
    }
}

// ============================================================================
// Genus Classifier
// ============================================================================

/// Minimap2-based genus classifier using flanking sequence alignment.
///
/// Classifies source genus by aligning extracted flanking sequences
/// against reference flanking sequences in the database.
pub struct GenusClassifier {
    db: FlankingDatabase,
    minimap2_path: String,
    min_identity: f64,
    min_align_len: usize,
    max_flanking: usize,
}

impl GenusClassifier {
    /// Creates a new genus classifier.
    ///
    /// # Arguments
    /// * `db_path` - Path to flanking database (.fdb)
    /// * `minimap2_path` - Path to minimap2 executable
    /// * `min_identity` - Minimum alignment identity (0-1)
    /// * `min_align_len` - Minimum alignment length in bp
    /// * `max_flanking` - Maximum flanking length to extract
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        minimap2_path: &str,
        min_identity: f64,
        min_align_len: usize,
        max_flanking: usize,
    ) -> Result<Self> {
        let db = FlankingDatabase::open(db_path)?;
        Ok(Self {
            db,
            minimap2_path: minimap2_path.to_string(),
            min_identity,
            min_align_len,
            max_flanking,
        })
    }

    /// Classifies genus for multiple ARG positions.
    ///
    /// Processes each position sequentially to avoid temporary file conflicts.
    pub fn classify_batch(&mut self, positions: &[ArgPosition], threads: usize) -> Result<Vec<GenusResult>> {
        let mut results = Vec::with_capacity(positions.len());

        for pos in positions {
            let result = self.classify_single(pos, threads)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Classifies genus for a single ARG position.
    ///
    /// # Algorithm
    /// 1. Extract flanking sequences from contig
    /// 2. Write query and reference FASTA files
    /// 3. Run minimap2 alignment
    /// 4. Parse PAF and score genus candidates
    /// 5. Return top genus with confidence metrics
    pub fn classify_single(&mut self, pos: &ArgPosition, threads: usize) -> Result<GenusResult> {
        // Extract flanking sequences
        let (upstream, downstream) = self.extract_flanking_regions(pos);

        let upstream_len = upstream.len();
        let downstream_len = downstream.len();

        // Verify SNP for point mutation genes
        let snp_status = snp::verify_snp(
            &pos.contig_seq,
            &pos.arg_name,
            0,
            pos.arg_end - pos.arg_start,
            pos.arg_start,
            pos.arg_end,
            pos.strand,
        );

        // Require minimum flanking for classification
        if upstream_len < 50 && downstream_len < 50 {
            return Ok(GenusResult {
                arg_name: pos.arg_name.clone(),
                contig_name: pos.contig_name.clone(),
                genus: None,
                confidence: 0.0,
                specificity: 0.0,
                upstream_len,
                downstream_len,
                top_matches: vec![],
                snp_status,
            });
        }

        // Check if gene exists in database
        if !self.db.has_gene(&pos.arg_name) {
            return Ok(GenusResult {
                arg_name: pos.arg_name.clone(),
                contig_name: pos.contig_name.clone(),
                genus: None,
                confidence: 0.0,
                specificity: 0.0,
                upstream_len,
                downstream_len,
                top_matches: vec![("gene_not_in_db".to_string(), 0.0)],
                snp_status,
            });
        }

        // Get reference flanking sequences
        let ref_records = self.db.get_gene_records(&pos.arg_name)?;
        if ref_records.is_empty() {
            return Ok(GenusResult {
                arg_name: pos.arg_name.clone(),
                contig_name: pos.contig_name.clone(),
                genus: None,
                confidence: 0.0,
                specificity: 0.0,
                upstream_len,
                downstream_len,
                top_matches: vec![("no_ref_records".to_string(), 0.0)],
                snp_status,
            });
        }

        // Create temporary files for alignment
        let temp_dir = std::env::temp_dir();
        let pid = std::process::id();
        let query_path = temp_dir.join(format!("argenus_query_{}.fas", pid));
        let ref_path = temp_dir.join(format!("argenus_ref_{}.fas", pid));
        let paf_path = temp_dir.join(format!("argenus_align_{}.paf", pid));

        // Write query FASTA
        {
            let mut query_file = BufWriter::new(File::create(&query_path)?);
            if !upstream.is_empty() {
                writeln!(query_file, ">upstream")?;
                writeln!(query_file, "{}", upstream)?;
            }
            if !downstream.is_empty() {
                writeln!(query_file, ">downstream")?;
                writeln!(query_file, "{}", downstream)?;
            }
        }

        // Write reference FASTA (grouped by genus)
        {
            let mut ref_file = BufWriter::new(File::create(&ref_path)?);
            for (i, rec) in ref_records.iter().enumerate() {
                if !rec.upstream.is_empty() {
                    writeln!(ref_file, ">{}|{}|up_{}", rec.genus, rec.contig, i)?;
                    writeln!(ref_file, "{}", rec.upstream)?;
                }
                if !rec.downstream.is_empty() {
                    writeln!(ref_file, ">{}|{}|down_{}", rec.genus, rec.contig, i)?;
                    writeln!(ref_file, "{}", rec.downstream)?;
                }
            }
        }

        // Run minimap2 with sr preset for short queries
        let output = Command::new(&self.minimap2_path)
            .args(["-x", "sr", "-t", &threads.to_string(), "-c", "--secondary=yes", "-N", "100", "-k", "15", "-w", "5"])
            .arg(&ref_path)
            .arg(&query_path)
            .arg("-o").arg(&paf_path)
            .stderr(std::process::Stdio::null())
            .output()
            .context("Failed to run minimap2")?;

        if !output.status.success() {
            // Cleanup and return error result
            let _ = std::fs::remove_file(&query_path);
            let _ = std::fs::remove_file(&ref_path);
            let _ = std::fs::remove_file(&paf_path);

            return Ok(GenusResult {
                arg_name: pos.arg_name.clone(),
                contig_name: pos.contig_name.clone(),
                genus: None,
                confidence: 0.0,
                specificity: 0.0,
                upstream_len,
                downstream_len,
                top_matches: vec![("minimap2_failed".to_string(), 0.0)],
                snp_status,
            });
        }

        // Parse PAF and calculate genus scores
        let genus_scores = self.calculate_genus_scores(&paf_path)?;

        // Cleanup temporary files
        let _ = std::fs::remove_file(&query_path);
        let _ = std::fs::remove_file(&ref_path);
        let _ = std::fs::remove_file(&paf_path);

        // Calculate genus specificity from database
        let genus_dist = self.db.get_genus_distribution(&pos.arg_name)?;
        let total_in_db: usize = genus_dist.values().sum();

        // Determine best genus
        let mut sorted_scores: Vec<(String, f64)> = genus_scores.into_iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (genus, confidence, specificity) = if let Some((best_genus, best_score)) = sorted_scores.first() {
            let genus_count = genus_dist.get(best_genus).copied().unwrap_or(0);
            let specificity = if total_in_db > 0 {
                (genus_count as f64 / total_in_db as f64) * 100.0
            } else {
                0.0
            };

            (Some(best_genus.clone()), *best_score, specificity)
        } else {
            (None, 0.0, 0.0)
        };

        let top_matches: Vec<(String, f64)> = sorted_scores.into_iter().take(5).collect();

        Ok(GenusResult {
            arg_name: pos.arg_name.clone(),
            contig_name: pos.contig_name.clone(),
            genus,
            confidence,
            specificity,
            upstream_len,
            downstream_len,
            top_matches,
            snp_status,
        })
    }

    /// Extracts flanking sequences from a contig.
    ///
    /// Handles strand orientation automatically.
    fn extract_flanking_regions(&self, pos: &ArgPosition) -> (String, String) {
        let seq = &pos.contig_seq;

        // Extract upstream (before ARG)
        let upstream_end = pos.arg_start;
        let upstream_start = upstream_end.saturating_sub(self.max_flanking);
        let upstream = if upstream_end > upstream_start {
            seq[upstream_start..upstream_end].to_string()
        } else {
            String::new()
        };

        // Extract downstream (after ARG)
        let downstream_start = pos.arg_end;
        let downstream_end = (downstream_start + self.max_flanking).min(seq.len());
        let downstream = if downstream_end > downstream_start {
            seq[downstream_start..downstream_end].to_string()
        } else {
            String::new()
        };

        // Handle reverse strand
        if pos.strand == '-' {
            (reverse_complement(&downstream), reverse_complement(&upstream))
        } else {
            (upstream, downstream)
        }
    }

    /// Parses PAF alignment file and calculates genus scores.
    fn calculate_genus_scores(&self, paf_path: &Path) -> Result<FxHashMap<String, f64>> {
        let file = File::open(paf_path)?;
        let reader = BufReader::new(file);

        let mut genus_matches: FxHashMap<String, Vec<f64>> = FxHashMap::default();
        let min_identity_pct = self.min_identity * 100.0;

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 12 {
                continue;
            }

            let block_len: usize = fields[10].parse().unwrap_or(0);
            let matches: usize = fields[9].parse().unwrap_or(0);

            if block_len < self.min_align_len {
                continue;
            }

            let identity = if block_len > 0 {
                (matches as f64 / block_len as f64) * 100.0
            } else {
                0.0
            };

            if identity < min_identity_pct {
                continue;
            }

            // Extract genus from target name (genus|assembly|direction_idx)
            let target_name = fields[5];
            if let Some(genus) = target_name.split('|').next() {
                genus_matches.entry(genus.to_string()).or_default().push(identity);
            }
        }

        // Calculate average score per genus
        let mut genus_scores: FxHashMap<String, f64> = FxHashMap::default();
        for (genus, scores) in genus_matches {
            if scores.is_empty() {
                continue;
            }
            // Weighted score: average identity with count bonus
            let avg_identity = scores.iter().sum::<f64>() / scores.len() as f64;
            let count_bonus = (scores.len() as f64).ln().max(1.0);
            genus_scores.insert(genus, avg_identity * count_bonus / count_bonus.max(1.0));
        }

        Ok(genus_scores)
    }

}

// ============================================================================
// Utility Functions
// ============================================================================

/// Computes the reverse complement of a DNA sequence.
fn reverse_complement(seq: &str) -> String {
    seq.chars()
        .rev()
        .map(|c| match c.to_ascii_uppercase() {
            'A' => 'T',
            'T' => 'A',
            'G' => 'C',
            'C' => 'G',
            _ => 'N',
        })
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_complement() {
        assert_eq!(reverse_complement("ATGC"), "GCAT");
        assert_eq!(reverse_complement("AAAA"), "TTTT");
        assert_eq!(reverse_complement(""), "");
    }

    #[test]
    fn test_genus_result_default() {
        let result = GenusResult::default();
        assert!(result.genus.is_none());
        assert_eq!(result.confidence, 0.0);
    }
}
