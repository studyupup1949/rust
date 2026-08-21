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
    /// Cap on total bp added to EACH contig end (default: 2000). Genus/species
    /// classification only uses ~max_flanking (1000 bp) of flanking, so extending a
    /// runaway (repetitive/high-coverage) contig for many more rounds is wasted work
    /// — and re-scanning all reads each round is the dominant runtime. Capping each
    /// side stops runaways early without affecting classification.
    pub max_extension_per_side: usize,
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
            max_extension_per_side: 2000,
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
    /// Read sequences (R1 then R2), shared so a single load can back multiple
    /// extension passes (strict + flexible) without re-reading the FASTQs.
    reads: std::sync::Arc<Vec<String>>,
}

impl ContigExtender {
    /// Creates a new contig extender with the given configuration.
    pub fn new(config: ExtenderConfig) -> Self {
        Self {
            config,
            reads: std::sync::Arc::new(Vec::new()),
        }
    }

    /// Loads paired-end reads (R1 then R2) into a shared `Arc<Vec<String>>`, in
    /// parallel from both files. Reusable across extender instances.
    pub fn load_reads_shared(r1_path: &Path, r2_path: &Path) -> Result<std::sync::Arc<Vec<String>>> {
        eprintln!("Loading reads into memory...");

        let r1_owned = r1_path.to_path_buf();
        let r2_owned = r2_path.to_path_buf();

        let load = |path: std::path::PathBuf| -> Result<Vec<String>> {
            let mut reads = Vec::new();
            let mut reader = FastqFile::open(&path)?;
            while let Some(record) = reader.read_next()? {
                reads.push(record.seq);
            }
            Ok(reads)
        };
        let handle_r1 = std::thread::spawn(move || load(r1_owned));
        let handle_r2 = std::thread::spawn(move || load(r2_owned));

        let mut reads = handle_r1.join().map_err(|_| anyhow::anyhow!("R1 load thread panicked"))??;
        let reads_r2 = handle_r2.join().map_err(|_| anyhow::anyhow!("R2 load thread panicked"))??;
        reads.extend(reads_r2);

        eprintln!("Loaded {} reads into memory", reads.len());
        Ok(std::sync::Arc::new(reads))
    }

    /// Loads reads from the two FASTQs into this extender.
    pub fn load_reads(&mut self, r1_path: &Path, r2_path: &Path) -> Result<()> {
        self.reads = Self::load_reads_shared(r1_path, r2_path)?;
        Ok(())
    }

    /// Reuses an already-loaded shared read set (see `load_reads_shared`).
    pub fn set_reads(&mut self, reads: std::sync::Arc<Vec<String>>) {
        self.reads = reads;
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
                left_grown: 0,
                right_grown: 0,
            })
        }).collect();

        let t_start = std::time::Instant::now();
        let mut rounds = 0usize;

        loop {
            rounds += 1;
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

            // Accumulate per-position base counts (not raw reads) so peak memory is
            // O(extension_step) per contig edge regardless of read depth.
            let left_candidates: Mutex<FxHashMap<usize, Vec<[u32; 4]>>> = Mutex::new(FxHashMap::default());
            let right_candidates: Mutex<FxHashMap<usize, Vec<[u32; 4]>>> = Mutex::new(FxHashMap::default());
            let max_len = self.config.extension_step;

            // Stream all reads (no index): metagenomic reads have mostly-distinct
            // k-mers, so building a read index costs far more (tens of millions of
            // per-k-mer allocations) than it saves — measured ~26x slower. The
            // allocation-free streaming scan is the right structure here.
            self.reads.par_iter().for_each(|read_seq| {
                scan_read(read_seq, &edge_kmers, &states, k, max_len,
                          &left_candidates, &right_candidates);
            });

            let left_candidates = left_candidates.into_inner().unwrap();
            let right_candidates = right_candidates.into_inner().unwrap();

            // Parallel extension application
            let any_extended = std::sync::atomic::AtomicBool::new(false);

            active_indices.par_iter().for_each(|&idx| {
                let mut state = states[idx].lock().unwrap();

                // Try left extension
                if state.left_failures < max_failures {
                    if let Some(counts) = left_candidates.get(&idx) {
                        let consensus = build_consensus_from_counts(
                            counts,
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
                                state.left_grown += consensus.len();
                                // Stop this side once it has enough flanking (cap).
                                if state.left_grown >= self.config.max_extension_per_side {
                                    state.left_failures = max_failures;
                                }
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
                }

                // Try right extension
                if state.right_failures < max_failures {
                    if let Some(counts) = right_candidates.get(&idx) {
                        let consensus = build_consensus_from_counts(
                            counts,
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
                                state.right_grown += consensus.len();
                                // Stop this side once it has enough flanking (cap).
                                if state.right_grown >= self.config.max_extension_per_side {
                                    state.right_failures = max_failures;
                                }
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
                }
            });

            if !any_extended.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
        }

        eprintln!(
            "        [extender] {} contigs, {} rounds, {} reads, {:.1}s",
            contigs.len(), rounds, self.reads.len(), t_start.elapsed().as_secs_f64()
        );

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
    /// bp added to each end so far (for the per-side extension cap).
    left_grown: usize,
    right_grown: usize,
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
/// Scan one read against the contig-edge k-mer index and accumulate its overhangs
/// (as per-position base counts) into the shared left/right count matrices. Shared
/// by both the full-scan and indexed dispatch paths so they produce identical output.
#[allow(clippy::too_many_arguments)]
fn scan_read(
    read_seq: &str,
    edge_kmers: &FxHashMap<u64, Vec<(usize, bool, usize)>>,
    states: &[Mutex<ContigState>],
    k: usize,
    max_len: usize,
    left_candidates: &Mutex<FxHashMap<usize, Vec<[u32; 4]>>>,
    right_candidates: &Mutex<FxHashMap<usize, Vec<[u32; 4]>>>,
) {
    if read_seq.len() < k {
        return;
    }
    let mut local_left: FxHashMap<usize, Vec<[u32; 4]>> = FxHashMap::default();
    let mut local_right: FxHashMap<usize, Vec<[u32; 4]>> = FxHashMap::default();

    for i in 0..=(read_seq.len() - k) {
        let kmer_seq = &read_seq[i..i + k];
        if let Some(hash) = compute_kmer_hash(kmer_seq) {
            if let Some(matches) = edge_kmers.get(&hash) {
                for &(contig_idx, is_left, edge_offset) in matches {
                    let state = states[contig_idx].lock().unwrap();
                    let contig_kmer = if is_left {
                        &state.current_seq[edge_offset..edge_offset + k]
                    } else {
                        let clen = state.current_seq.len();
                        &state.current_seq[clen - k - edge_offset..clen - edge_offset]
                    };

                    let (is_forward, is_revcomp) = check_kmer_match(kmer_seq, contig_kmer);
                    drop(state); // Release lock early

                    if is_left {
                        if is_forward && i > edge_offset {
                            let prefix = &read_seq[..i - edge_offset];
                            if !prefix.is_empty() {
                                let ext: String = prefix.chars().rev().collect();
                                accumulate_counts(local_left.entry(contig_idx).or_default(), &ext, max_len);
                            }
                        } else if is_revcomp && i + k + edge_offset < read_seq.len() {
                            let suffix = &read_seq[i + k + edge_offset..];
                            if !suffix.is_empty() {
                                let ext = reverse_complement(suffix);
                                accumulate_counts(local_left.entry(contig_idx).or_default(), &ext, max_len);
                            }
                        }
                    } else if is_forward && i + k + edge_offset < read_seq.len() {
                        let suffix = &read_seq[i + k + edge_offset..];
                        if !suffix.is_empty() {
                            accumulate_counts(local_right.entry(contig_idx).or_default(), suffix, max_len);
                        }
                    } else if is_revcomp && i > edge_offset {
                        let prefix = &read_seq[..i - edge_offset];
                        if !prefix.is_empty() {
                            let ext = reverse_complement(prefix);
                            accumulate_counts(local_right.entry(contig_idx).or_default(), &ext, max_len);
                        }
                    }
                }
            }
        }
    }

    if !local_left.is_empty() {
        let mut global = left_candidates.lock().unwrap();
        for (idx, local_counts) in local_left {
            merge_counts(global.entry(idx).or_default(), &local_counts);
        }
    }
    if !local_right.is_empty() {
        let mut global = right_candidates.lock().unwrap();
        for (idx, local_counts) in local_right {
            merge_counts(global.entry(idx).or_default(), &local_counts);
        }
    }
}

/// Accumulate one oriented overhang's bases into a per-position [A,T,G,C] count
/// matrix (index 0 = first base past the contig edge). Non-ACGT bases are skipped
/// (matching build_consensus_sequence). Positions beyond `max_len` are ignored.
/// This lets extension consensus be built incrementally without ever storing the
/// raw reads — bounding memory to O(max_len) per contig edge regardless of depth.
fn accumulate_counts(counts: &mut Vec<[u32; 4]>, overhang: &str, max_len: usize) {
    for (i, c) in overhang.bytes().enumerate() {
        if i >= max_len {
            break;
        }
        let bi = match c.to_ascii_uppercase() {
            b'A' => 0,
            b'T' => 1,
            b'G' => 2,
            b'C' => 3,
            _ => continue,
        };
        if i >= counts.len() {
            counts.resize(i + 1, [0; 4]);
        }
        counts[i][bi] += 1;
    }
}

/// Element-wise add a (thread-local) count matrix into a shared one.
fn merge_counts(dst: &mut Vec<[u32; 4]>, src: &[[u32; 4]]) {
    if dst.len() < src.len() {
        dst.resize(src.len(), [0; 4]);
    }
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        for b in 0..4 {
            d[b] += s[b];
        }
    }
}

/// Consensus from a per-position [A,T,G,C] count matrix. Identical decision logic
/// to `build_consensus_sequence` (majority base per position; 'N' when the minor
/// allele frequency >= branching_threshold; stop at the first position with
/// coverage < min_coverage), but reads all positions from accumulated counts so no
/// raw reads are retained. Using ALL reads (no truncation) makes the consensus
/// exact, not a sampled approximation.
fn build_consensus_from_counts(
    counts: &[[u32; 4]],
    min_coverage: usize,
    branching_threshold: f64,
    max_len: usize,
) -> String {
    let mut result = String::new();
    for col in counts.iter().take(max_len) {
        let total: u32 = col.iter().sum();
        if (total as usize) < min_coverage {
            break;
        }
        let max_idx = col.iter().enumerate().max_by_key(|&(_, &c)| c).map(|(i, _)| i).unwrap_or(0);
        let mut sorted = *col;
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        let minor_freq = sorted[1] as f64 / total as f64;
        let base = if minor_freq >= branching_threshold {
            'N'
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

/// Reference (string-based) consensus, retained as the correctness oracle for
/// `build_consensus_from_counts` (see test_count_consensus_matches_string_version).
#[cfg_attr(not(test), allow(dead_code))]
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

    /// Count-based consensus must extend correctly and be independent of read depth:
    /// 50 vs 50000 agreeing reads give the SAME extension (the count matrix uses all
    /// reads, no truncation). This is the memory fix — raw reads are never stored.
    #[test]
    fn test_count_consensus_extension() {
        // Non-repetitive contig + a distinct right-side extension truth.
        let contig = "GTTCAGACCTAGGCATTACGGATCCGATTACGGCATTAGCCATTAGGCAT";
        let ext_truth = "ACAGTGGTCATGCATGCTAGCTAGCATCGAT";
        // Reads share the contig's right edge and carry the extension.
        let read = format!("{}{}", &contig[contig.len() - 40..], ext_truth);
        let contigs = vec![FastaRecord { name: "c1".into(), seq: contig.to_string() }];

        let run = |n_reads: usize| -> String {
            let mut cfg = ExtenderConfig::default();
            cfg.max_consecutive_failures = 1;
            let mut ext = ContigExtender::new(cfg);
            ext.reads = std::sync::Arc::new(std::iter::repeat(read.clone()).take(n_reads).collect());
            ext.extend_contigs(&contigs).unwrap()[0].extended_seq.clone()
        };

        let few = run(50);
        let many = run(50_000);
        assert_eq!(few, many, "extension depends on read depth (should not)");
        // And the extension actually happened (grew past the original contig).
        assert!(few.len() > contig.len(), "no extension occurred");
        assert!(few.contains(contig), "original contig not preserved");
    }

    /// The count matrix must reproduce build_consensus_sequence exactly (majority
    /// base, 'N' on branching, stop below min_coverage).
    #[test]
    fn test_count_consensus_matches_string_version() {
        let seqs = vec![
            "ACGT".to_string(),
            "ACGT".to_string(),
            "ACTT".to_string(), // position 2 branches: G vs T
        ];
        let mut counts: Vec<[u32; 4]> = Vec::new();
        for s in &seqs {
            accumulate_counts(&mut counts, s, 100);
        }
        let from_counts = build_consensus_from_counts(&counts, 2, 0.2, 100);
        let from_strings = build_consensus_sequence(&seqs, 2, 0.2, 100);
        assert_eq!(from_counts, from_strings);
    }

    /// Peak RSS (VmHWM, KB) of this process, from /proc/self/status.
    fn peak_rss_kb() -> u64 {
        std::fs::read_to_string("/proc/self/status").ok()
            .and_then(|s| s.lines().find(|l| l.starts_with("VmHWM"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok()))
            .unwrap_or(0)
    }

    /// Micro-benchmark (node-load-independent): the per-position count matrix bounds
    /// extension memory regardless of read depth — the explosion (millions of reads
    /// on ONE contig edge) now adds ~0. Run with `--ignored --nocapture`; vary NREADS
    /// to confirm the RSS delta stays flat.
    #[test]
    #[ignore]
    fn bench_count_memory() {
        let n_reads: usize = std::env::var("NREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(2_000_000);
        let contig = "GTTCAGACCTAGGCATTACGGATCCGATTACGGCATTAGCCATTAGGCAT";
        let ext_truth = "ACAGTGGTCATGCATGCTAGCTAGCATCGAT";
        let read = format!("{}{}", &contig[contig.len() - 40..], ext_truth);
        let contigs = vec![FastaRecord { name: "c1".into(), seq: contig.to_string() }];

        let mut cfg = ExtenderConfig::default();
        cfg.max_consecutive_failures = 1;
        let mut ext = ContigExtender::new(cfg);
        ext.reads = std::sync::Arc::new(std::iter::repeat(read.clone()).take(n_reads).collect());
        let reads_rss = peak_rss_kb();
        let out = ext.extend_contigs(&contigs).unwrap();
        let after = peak_rss_kb();
        eprintln!("NREADS={} | reads_loaded={}MB after_extend={}MB | delta_extend={}MB | ext_len={}",
            n_reads, reads_rss / 1024, after / 1024, (after.saturating_sub(reads_rss)) / 1024, out[0].extended_seq.len());
    }
}
