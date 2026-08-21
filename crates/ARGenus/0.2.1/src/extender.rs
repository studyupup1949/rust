//! Contig Extender Module
//!
//! Provides k-mer based contig extension using paired-end reads.
//! Extends assembled contigs by finding overlapping reads at contig edges.
//!
//! # Extension Modes
//! - **Strict mode**: High coverage requirement, low branching tolerance.
//!   Use for high-confidence extensions where accuracy is critical.
//!   Parameters: `min_coverage=3`, `branching_threshold=0.1`, `max_n_ratio=0.02`
//!
//! - **Relaxed mode**: Lower coverage, higher branching tolerance.
//!   Use for unresolved cases where longer extensions are needed.
//!   Parameters: `min_coverage=2`, `branching_threshold=0.2`, `max_n_ratio=0.05`
//!
//! # Algorithm Overview
//! 1. Build k-mer index from contig edges
//! 2. Scan reads for matching k-mers
//! 3. Collect extension candidates from overlapping reads
//! 4. Build consensus sequence with branching detection
//! 5. Repeat until no further extension possible

use anyhow::Result;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;

use crate::seqio::{FastaRecord, FastqFile};

// ============================================================================
// Configuration
// ============================================================================

/// Configuration parameters for contig extension.
///
/// # Strict vs Relaxed Mode
/// Strict mode (default) prioritises accuracy over length.
/// Relaxed mode allows more aggressive extension for difficult cases.
///
/// | Parameter           | Strict | Relaxed | Description                    |
/// |---------------------|--------|---------|--------------------------------|
/// | min_coverage        | 3      | 2       | Minimum read support           |
/// | branching_threshold | 0.1    | 0.2     | Minor allele frequency for N   |
/// | max_n_ratio         | 0.02   | 0.05    | Maximum allowed N proportion   |
#[derive(Clone)]
pub struct ExtenderConfig {
    /// K-mer size for extension (default: 21).
    pub kmer_size: usize,
    /// Number of edge k-mers to use from each end (default: 5).
    pub num_edge_kmers: usize,
    /// Minimum read coverage for consensus building (default: 2).
    pub min_coverage: usize,
    /// Branching threshold: if minor allele frequency >= this, mark as N (default: 0.2).
    /// Lower values = stricter, fewer Ns allowed.
    pub branching_threshold: f64,
    /// Maximum N ratio allowed in extension (default: 0.05).
    /// Extensions exceeding this are rejected.
    pub max_n_ratio: f64,
    /// Extension length per iteration in base pairs (default: 200).
    pub extension_step: usize,
    /// Maximum consecutive failures before stopping (default: 2).
    pub max_consecutive_failures: usize,
}

impl Default for ExtenderConfig {
    fn default() -> Self {
        Self {
            kmer_size: 21,
            num_edge_kmers: 5,
            min_coverage: 2,
            branching_threshold: 0.2,
            max_n_ratio: 0.05,
            extension_step: 200,
            max_consecutive_failures: 2,
        }
    }
}

// ============================================================================
// Results
// ============================================================================

/// Result of extension for a single contig.
#[derive(Debug, Clone)]
pub struct ExtendedContig {
    /// Original contig name.
    pub name: String,
    /// Extended sequence (original + extensions).
    pub extended_seq: String,
}

// ============================================================================
// Contig Extender
// ============================================================================

/// K-mer based contig extender using paired-end reads.
///
/// Loads reads into memory for efficient repeated scanning.
/// Supports both sequential and parallel extension strategies.
pub struct ContigExtender {
    config: ExtenderConfig,
    reads: Vec<String>,
}

impl ContigExtender {
    /// Creates a new contig extender with the given configuration.
    pub fn new(config: ExtenderConfig) -> Self {
        Self {
            config,
            reads: Vec::new(),
        }
    }

    /// Loads paired-end reads into memory for extension.
    ///
    /// Reads are loaded in parallel from both files.
    /// Both forward and reverse reads are stored for extension.
    ///
    /// # Arguments
    /// * `r1_path` - Path to forward reads (R1)
    /// * `r2_path` - Path to reverse reads (R2)
    pub fn load_reads(&mut self, r1_path: &Path, r2_path: &Path) -> Result<()> {
        eprintln!("Loading reads into memory...");

        let r1_owned = r1_path.to_path_buf();
        let r2_owned = r2_path.to_path_buf();

        // Load R1 and R2 in parallel
        let handle_r1 = std::thread::spawn(move || -> Result<Vec<String>> {
            let mut reads = Vec::new();
            let mut reader = FastqFile::open(&r1_owned)?;
            while let Some(record) = reader.read_next()? {
                reads.push(record.seq);
            }
            Ok(reads)
        });

        let handle_r2 = std::thread::spawn(move || -> Result<Vec<String>> {
            let mut reads = Vec::new();
            let mut reader = FastqFile::open(&r2_owned)?;
            while let Some(record) = reader.read_next()? {
                reads.push(record.seq);
            }
            Ok(reads)
        });

        let reads_r1 = handle_r1.join().map_err(|_| anyhow::anyhow!("R1 load thread panicked"))??;
        let reads_r2 = handle_r2.join().map_err(|_| anyhow::anyhow!("R2 load thread panicked"))??;

        self.reads = reads_r1;
        self.reads.extend(reads_r2);

        eprintln!("Loaded {} reads into memory", self.reads.len());
        Ok(())
    }

    /// Extends all contigs using hybrid parallel strategy.
    ///
    /// Combines shared read scanning with parallel extension application.
    /// This is the recommended method for most use cases.
    ///
    /// # Algorithm
    /// 1. Build edge k-mer index for active contigs
    /// 2. Parallel read scanning (shared across all contigs)
    /// 3. Parallel extension application per contig
    /// 4. Repeat until no contigs can be extended
    pub fn extend_contigs(&self, contigs: &[FastaRecord]) -> Result<Vec<ExtendedContig>> {
        let k = self.config.kmer_size;
        let max_failures = self.config.max_consecutive_failures;

        // Use Arc<Mutex> for thread-safe state updates
        let states: Vec<Mutex<ContigState>> = contigs.iter().map(|c| {
            Mutex::new(ContigState {
                name: c.name.clone(),
                current_seq: c.seq.clone(),
                left_failures: 0,
                right_failures: 0,
            })
        }).collect();

        loop {
            // Identify contigs that still need extension
            let active_indices: Vec<usize> = states.iter().enumerate()
                .filter(|(_, s)| {
                    let s = s.lock().unwrap();
                    s.left_failures < max_failures || s.right_failures < max_failures
                })
                .map(|(i, _)| i)
                .collect();

            if active_indices.is_empty() {
                break;
            }

            // Build edge k-mer index for active contigs
            let mut edge_kmers: FxHashMap<u64, Vec<(usize, bool, usize)>> = FxHashMap::default();

            for &idx in &active_indices {
                let state = states[idx].lock().unwrap();
                let seq = &state.current_seq;
                if seq.len() < k {
                    continue;
                }

                // Index left edge k-mers
                if state.left_failures < max_failures {
                    for offset in 0..self.config.num_edge_kmers.min(seq.len() - k + 1) {
                        if let Some(hash) = compute_kmer_hash(&seq[offset..offset+k]) {
                            edge_kmers.entry(hash).or_default().push((idx, true, offset));
                        }
                    }
                }

                // Index right edge k-mers
                if state.right_failures < max_failures {
                    let seq_len = seq.len();
                    for offset in 0..self.config.num_edge_kmers.min(seq.len() - k + 1) {
                        let start = seq_len - k - offset;
                        if let Some(hash) = compute_kmer_hash(&seq[start..start+k]) {
                            edge_kmers.entry(hash).or_default().push((idx, false, offset));
                        }
                    }
                }
            }

            // Parallel read scanning with local buffers to reduce lock contention
            let left_candidates: Mutex<FxHashMap<usize, Vec<String>>> = Mutex::new(FxHashMap::default());
            let right_candidates: Mutex<FxHashMap<usize, Vec<String>>> = Mutex::new(FxHashMap::default());

            self.reads.par_iter().for_each(|read_seq| {
                if read_seq.len() < k {
                    return;
                }

                // Local buffers for this read
                let mut local_left: FxHashMap<usize, Vec<String>> = FxHashMap::default();
                let mut local_right: FxHashMap<usize, Vec<String>> = FxHashMap::default();

                for i in 0..=(read_seq.len() - k) {
                    let kmer_seq = &read_seq[i..i+k];
                    if let Some(hash) = compute_kmer_hash(kmer_seq) {
                        if let Some(matches) = edge_kmers.get(&hash) {
                            for &(contig_idx, is_left, edge_offset) in matches {
                                let state = states[contig_idx].lock().unwrap();
                                let contig_kmer = if is_left {
                                    &state.current_seq[edge_offset..edge_offset+k]
                                } else {
                                    let clen = state.current_seq.len();
                                    &state.current_seq[clen-k-edge_offset..clen-edge_offset]
                                };

                                let (is_forward, is_revcomp) = check_kmer_match(kmer_seq, contig_kmer);
                                drop(state); // Release lock early

                                if is_left {
                                    if is_forward && i > edge_offset {
                                        let prefix = &read_seq[..i - edge_offset];
                                        if !prefix.is_empty() {
                                            let ext: String = prefix.chars().rev().collect();
                                            local_left.entry(contig_idx).or_default().push(ext);
                                        }
                                    } else if is_revcomp && i + k + edge_offset < read_seq.len() {
                                        let suffix = &read_seq[i+k+edge_offset..];
                                        if !suffix.is_empty() {
                                            let ext = reverse_complement(suffix);
                                            local_left.entry(contig_idx).or_default().push(ext);
                                        }
                                    }
                                } else if is_forward && i + k + edge_offset < read_seq.len() {
                                    let suffix = &read_seq[i+k+edge_offset..];
                                    if !suffix.is_empty() {
                                        local_right.entry(contig_idx).or_default().push(suffix.to_string());
                                    }
                                } else if is_revcomp && i > edge_offset {
                                    let prefix = &read_seq[..i - edge_offset];
                                    if !prefix.is_empty() {
                                        let ext = reverse_complement(prefix);
                                        local_right.entry(contig_idx).or_default().push(ext);
                                    }
                                }
                            }
                        }
                    }
                }

                // Merge local buffers to global (reduces lock contention)
                if !local_left.is_empty() {
                    let mut global = left_candidates.lock().unwrap();
                    for (idx, candidates) in local_left {
                        global.entry(idx).or_default().extend(candidates);
                    }
                }
                if !local_right.is_empty() {
                    let mut global = right_candidates.lock().unwrap();
                    for (idx, candidates) in local_right {
                        global.entry(idx).or_default().extend(candidates);
                    }
                }
            });

            let left_candidates = left_candidates.into_inner().unwrap();
            let right_candidates = right_candidates.into_inner().unwrap();

            // Parallel extension application
            let any_extended = std::sync::atomic::AtomicBool::new(false);

            active_indices.par_iter().for_each(|&idx| {
                let mut state = states[idx].lock().unwrap();

                // Try left extension
                if state.left_failures < max_failures {
                    if let Some(candidates) = left_candidates.get(&idx) {
                        if candidates.len() >= self.config.min_coverage {
                            let consensus = build_consensus_sequence(
                                candidates,
                                self.config.min_coverage,
                                self.config.branching_threshold,
                                self.config.extension_step,
                            );
                            if !consensus.is_empty() {
                                let n_count = consensus.chars().filter(|&c| c == 'N').count();
                                let n_ratio = n_count as f64 / consensus.len() as f64;

                                if n_ratio <= self.config.max_n_ratio {
                                    state.current_seq = format!("{}{}", consensus, state.current_seq);
                                    state.left_failures = 0;
                                    any_extended.store(true, std::sync::atomic::Ordering::Relaxed);
                                } else {
                                    state.left_failures += 1;
                                }
                            } else {
                                state.left_failures += 1;
                            }
                        } else {
                            state.left_failures += 1;
                        }
                    } else {
                        state.left_failures += 1;
                    }
                }

                // Try right extension
                if state.right_failures < max_failures {
                    if let Some(candidates) = right_candidates.get(&idx) {
                        if candidates.len() >= self.config.min_coverage {
                            let consensus = build_consensus_sequence(
                                candidates,
                                self.config.min_coverage,
                                self.config.branching_threshold,
                                self.config.extension_step,
                            );
                            if !consensus.is_empty() {
                                let n_count = consensus.chars().filter(|&c| c == 'N').count();
                                let n_ratio = n_count as f64 / consensus.len() as f64;

                                if n_ratio <= self.config.max_n_ratio {
                                    state.current_seq = format!("{}{}", state.current_seq, consensus);
                                    state.right_failures = 0;
                                    any_extended.store(true, std::sync::atomic::Ordering::Relaxed);
                                } else {
                                    state.right_failures += 1;
                                }
                            } else {
                                state.right_failures += 1;
                            }
                        } else {
                            state.right_failures += 1;
                        }
                    } else {
                        state.right_failures += 1;
                    }
                }
            });

            if !any_extended.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
        }

        // Convert states to results
        let results = states.into_iter().map(|s| {
            let s = s.into_inner().unwrap();
            ExtendedContig {
                name: s.name,
                extended_seq: s.current_seq,
            }
        }).collect();

        Ok(results)
    }

    /// Alias for extend_contigs() - hybrid parallel method.
    #[inline]
    pub fn extend_all_hybrid(&self, contigs: &[FastaRecord]) -> Result<Vec<ExtendedContig>> {
        self.extend_contigs(contigs)
    }
}

// ============================================================================
// Internal State
// ============================================================================

/// Internal state for each contig during extension.
struct ContigState {
    name: String,
    current_seq: String,
    left_failures: usize,
    right_failures: usize,
}

// ============================================================================
// K-mer Utilities
// ============================================================================

/// Computes canonical k-mer hash (minimum of forward and reverse complement).
///
/// Returns None if the k-mer contains non-ATGC characters.
fn compute_kmer_hash(kmer: &str) -> Option<u64> {
    let bytes = kmer.as_bytes();
    let mut forward = 0u64;
    let mut reverse = 0u64;

    for (i, &b) in bytes.iter().enumerate() {
        let base = match b {
            b'A' | b'a' => 0,
            b'T' | b't' => 3,
            b'G' | b'g' => 1,
            b'C' | b'c' => 2,
            _ => return None,
        };
        forward = (forward << 2) | base;
        reverse |= (3 - base) << (2 * i);
    }

    Some(forward.min(reverse))
}

/// Checks if a read k-mer matches a contig k-mer (forward or reverse complement).
///
/// Returns (is_forward_match, is_reverse_complement_match).
fn check_kmer_match(read_kmer: &str, contig_kmer: &str) -> (bool, bool) {
    let is_forward = read_kmer == contig_kmer;
    let is_revcomp = if is_forward {
        false
    } else {
        reverse_complement(read_kmer) == contig_kmer
    };
    (is_forward, is_revcomp)
}

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
// Consensus Building
// ============================================================================

/// Builds consensus sequence from multiple extension candidates.
///
/// Uses positional voting with branching detection.
/// Positions with minor allele frequency >= threshold are marked as N.
///
/// # Arguments
/// * `sequences` - Extension candidates from overlapping reads
/// * `min_coverage` - Minimum bases required at each position
/// * `branching_threshold` - Minor allele frequency threshold for N
/// * `max_len` - Maximum consensus length
fn build_consensus_sequence(
    sequences: &[String],
    min_coverage: usize,
    branching_threshold: f64,
    max_len: usize,
) -> String {
    if sequences.is_empty() {
        return String::new();
    }

    let actual_max_len = sequences.iter().map(|s| s.len()).max().unwrap_or(0).min(max_len);
    let mut result = String::new();

    for i in 0..actual_max_len {
        // Collect bases at this position
        let bases: Vec<char> = sequences
            .iter()
            .filter_map(|s| s.chars().nth(i))
            .filter(|&c| matches!(c.to_ascii_uppercase(), 'A' | 'T' | 'G' | 'C'))
            .collect();

        if bases.len() < min_coverage {
            break;
        }

        // Count base frequencies
        let mut counts = [0usize; 4]; // A, T, G, C
        for &b in &bases {
            match b.to_ascii_uppercase() {
                'A' => counts[0] += 1,
                'T' => counts[1] += 1,
                'G' => counts[2] += 1,
                'C' => counts[3] += 1,
                _ => {}
            }
        }

        let total = counts.iter().sum::<usize>();
        let max_idx = counts.iter().enumerate()
            .max_by_key(|&(_, &c)| c)
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Check for branching (second most common base)
        let mut sorted_counts = counts;
        sorted_counts.sort_by(|a, b| b.cmp(a));
        let second_count = sorted_counts[1];
        let minor_freq = second_count as f64 / total as f64;

        // Determine consensus base
        let base = if minor_freq >= branching_threshold {
            'N' // Ambiguous position
        } else {
            match max_idx {
                0 => 'A',
                1 => 'T',
                2 => 'G',
                3 => 'C',
                _ => 'N',
            }
        };

        result.push(base);
    }

    result
}

// ============================================================================
// Output Functions
// ============================================================================

/// Writes extended contigs to a FASTA file.
///
/// # Arguments
/// * `results` - Extended contig results
/// * `path` - Output file path
pub fn write_extended_contigs(results: &[ExtendedContig], path: &Path) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);

    for result in results {
        writeln!(writer, ">{}", result.name)?;
        writeln!(writer, "{}", result.extended_seq)?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_kmer_hash() {
        // Same k-mer should produce same hash
        let h1 = compute_kmer_hash("ATGC").unwrap();
        let h2 = compute_kmer_hash("ATGC").unwrap();
        assert_eq!(h1, h2);

        // Reverse complement should produce same canonical hash
        let h3 = compute_kmer_hash("GCAT").unwrap();
        assert_eq!(h1, h3);

        // K-mer with N should return None
        assert!(compute_kmer_hash("ATNG").is_none());
    }

    #[test]
    fn test_reverse_complement() {
        assert_eq!(reverse_complement("ATGC"), "GCAT");
        assert_eq!(reverse_complement("AAAA"), "TTTT");
        assert_eq!(reverse_complement(""), "");
    }

    #[test]
    fn test_build_consensus() {
        let seqs = vec![
            "ATGC".to_string(),
            "ATGC".to_string(),
            "ATGC".to_string(),
        ];
        let consensus = build_consensus_sequence(&seqs, 2, 0.2, 100);
        assert_eq!(consensus, "ATGC");
    }

}
