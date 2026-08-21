//! nt_prok-based 5000bp Flanking Database Builder
//!
//! Uses BLASTN against NCBI nt_prok database for gene alignment
//! and blastdbcmd for flanking sequence extraction.
//!
//! # External Dependencies (user-provided paths)
//! - blastn: NCBI BLAST+ nucleotide alignment
//! - blastdbcmd: NCBI BLAST+ sequence retrieval
//! - nt_prok: NCBI prokaryotic nucleotide database
//! - taxdump: NCBI taxonomy dump (names.dmp, nodes.dmp)
//!
//! # Strand Handling (Internal Only)
//! Strand information from BLAST (sstart > send indicates minus strand)
//! is used INTERNALLY to:
//! 1. Calculate correct upstream/downstream coordinates
//! 2. Reverse-complement sequences when extracting from minus strand
//!
//! Strand is NOT written to FDB TSV because sequences are pre-oriented
//! during extraction. The classifier uses strand from ArgPosition
//! (live detection from reads), not from FDB.

use anyhow::{Context, Result};
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

const NCBI_TAXDUMP_URL: &str = "https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz";

/// Configuration for 5000bp FDB building
pub struct NtProkConfig {
    pub blastn_path: PathBuf,
    pub blastdbcmd_path: PathBuf,
    pub nt_prok_db: PathBuf,
    pub taxdump_dir: PathBuf,
    pub flanking_length: usize,  // 5000
    pub threads: usize,
    pub blast_identity: f64,     // 95.0
    pub blast_qcov: f64,         // 90.0
}

/// Taxonomy database loaded from taxdump
pub struct TaxonomyDb {
    names: FxHashMap<u32, String>,        // taxid -> scientific_name
    nodes: FxHashMap<u32, (u32, String)>, // taxid -> (parent_id, rank)
}

/// BLAST result for a single hit
#[derive(Debug, Clone)]
struct BlastHit {
    qseqid: String,     // Gene name
    sseqid: String,     // Subject accession (e.g., NC_001234.1)
    #[allow(dead_code)] // Parsed for potential future filtering
    pident: f64,        // Percent identity
    sstart: usize,      // Subject start
    send: usize,        // Subject end
    staxid: u32,        // Subject taxonomy ID
    ssciname: String,   // Subject scientific name (taxonomy fallback)
}

impl BlastHit {
    /// Determine strand from coordinates (internal use only)
    fn is_minus_strand(&self) -> bool {
        self.sstart > self.send
    }

    /// Get normalized coordinates (always start < end)
    fn normalized_coords(&self) -> (usize, usize) {
        if self.is_minus_strand() {
            (self.send, self.sstart)
        } else {
            (self.sstart, self.send)
        }
    }
}

/// Extracted flanking sequences (already oriented for plus strand)
#[derive(Default, Clone)]
struct FlankingSeqs {
    upstream: String,
    downstream: String,
}

/// Validate ARG database input is FASTA format
/// The .mmi format is a lossy compressed index that cannot recover sequences
pub fn validate_arg_db_format(arg_db: &Path) -> Result<()> {
    let extension = arg_db.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "mmi" => {
            anyhow::bail!(
                "Flanking database build (--mode long) requires FASTA with full sequences.\n\
                The .mmi index does not contain sequence data.\n\
                Please provide the original .fas or .fasta file.\n\
                \n\
                Path provided: {}", arg_db.display()
            );
        }
        "fa" | "fas" | "fasta" | "fna" => {
            // Valid FASTA extension - verify content
            let mut file = File::open(arg_db)
                .with_context(|| format!("Failed to open ARG database file: {}", arg_db.display()))?;
            let mut buffer = [0u8; 1];
            file.read_exact(&mut buffer)
                .with_context(|| "ARG database file is empty")?;
            if buffer[0] != b'>' {
                anyhow::bail!(
                    "ARG database file does not appear to be FASTA format.\n\
                    Expected file starting with '>' character.\n\
                    Path provided: {}", arg_db.display()
                );
            }
            Ok(())
        }
        _ => {
            // Check if file starts with '>' (FASTA format)
            let mut file = File::open(arg_db)
                .with_context(|| format!("Failed to open ARG database file: {}", arg_db.display()))?;
            let mut buffer = [0u8; 1];
            file.read_exact(&mut buffer)
                .with_context(|| "ARG database file is empty")?;
            if buffer[0] != b'>' {
                anyhow::bail!(
                    "ARG database file does not appear to be FASTA format.\n\
                    Expected file starting with '>' character.\n\
                    Path provided: {}", arg_db.display()
                );
            }
            Ok(())
        }
    }
}

impl TaxonomyDb {
    /// Load taxonomy from NCBI taxdump directory
    pub fn load(taxdump_dir: &Path) -> Result<Self> {
        let names_path = taxdump_dir.join("names.dmp");
        let nodes_path = taxdump_dir.join("nodes.dmp");

        if !names_path.exists() {
            anyhow::bail!("names.dmp not found in taxdump directory: {}", taxdump_dir.display());
        }
        if !nodes_path.exists() {
            anyhow::bail!("nodes.dmp not found in taxdump directory: {}", taxdump_dir.display());
        }

        eprintln!("Loading taxonomy from {}...", taxdump_dir.display());
        let names = Self::parse_names_dmp(&names_path)?;
        eprintln!("  Loaded {} scientific names", names.len());

        let nodes = Self::parse_nodes_dmp(&nodes_path)?;
        eprintln!("  Loaded {} taxonomy nodes", nodes.len());

        Ok(Self { names, nodes })
    }

    /// Parse names.dmp file
    /// Format: taxid | name | unique name | name class |
    fn parse_names_dmp(path: &Path) -> Result<FxHashMap<u32, String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut names = FxHashMap::default();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split("\t|\t").collect();
            if fields.len() >= 4 {
                // Only keep "scientific name" entries
                let name_class = fields[3].trim_end_matches("\t|");
                if name_class == "scientific name" {
                    if let Ok(taxid) = fields[0].parse::<u32>() {
                        names.insert(taxid, fields[1].to_string());
                    }
                }
            }
        }

        Ok(names)
    }

    /// Parse nodes.dmp file
    /// Format: taxid | parent_taxid | rank | ...
    fn parse_nodes_dmp(path: &Path) -> Result<FxHashMap<u32, (u32, String)>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut nodes = FxHashMap::default();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split("\t|\t").collect();
            if fields.len() >= 3 {
                if let (Ok(taxid), Ok(parent)) = (fields[0].parse::<u32>(), fields[1].parse::<u32>()) {
                    let rank = fields[2].to_string();
                    nodes.insert(taxid, (parent, rank));
                }
            }
        }

        Ok(nodes)
    }

    /// Get genus name for a given taxid by traversing up the taxonomy tree
    pub fn get_genus(&self, taxid: u32) -> Option<String> {
        let mut current = taxid;
        for _ in 0..50 {  // Max depth to prevent infinite loop
            let (parent, rank) = self.nodes.get(&current)?;
            if rank == "genus" {
                return self.names.get(&current).cloned();
            }
            if *parent == current {  // Root reached
                break;
            }
            current = *parent;
        }
        None
    }

    /// Extract genus from scientific name (sscinames fallback)
    /// Handles format: "Genus species" or "Genus species subsp. X"
    pub fn extract_genus_from_sciname(sciname: &str) -> Option<String> {
        let trimmed = sciname.trim();
        if trimmed.is_empty() || trimmed == "N/A" {
            return None;
        }
        trimmed.split_whitespace().next().map(|s| s.to_string())
    }
}

/// Ensure taxdump is present, downloading if necessary
/// Returns the taxdump directory path
pub fn ensure_taxdump(taxdump_dir: &Path) -> Result<()> {
    let names_path = taxdump_dir.join("names.dmp");
    let nodes_path = taxdump_dir.join("nodes.dmp");

    // If both files exist, taxdump is ready
    if names_path.exists() && nodes_path.exists() {
        eprintln!("Taxdump already exists at {}", taxdump_dir.display());
        return Ok(());
    }

    // Download and extract taxdump
    eprintln!("Taxdump not found. Downloading from NCBI (~60MB)...");
    std::fs::create_dir_all(taxdump_dir)?;

    let tar_path = taxdump_dir.join("taxdump.tar.gz");

    // Download with retry
    download_file_with_retry(NCBI_TAXDUMP_URL, &tar_path)?;

    // Extract
    eprintln!("  Extracting taxdump...");
    let status = Command::new("tar")
        .args(["-xzf", tar_path.to_str().unwrap(), "-C", taxdump_dir.to_str().unwrap()])
        .status()
        .with_context(|| "Failed to extract taxdump")?;

    if !status.success() {
        anyhow::bail!("tar extraction failed");
    }

    // Clean up tar file
    std::fs::remove_file(&tar_path).ok();

    eprintln!("  Taxdump downloaded and extracted successfully");
    Ok(())
}

/// Download a file from URL with retry
fn download_file_with_retry(url: &str, output_path: &Path) -> Result<()> {
    for attempt in 0..3 {
        match download_file_once(url, output_path) {
            Ok(_) => return Ok(()),
            Err(e) if attempt < 2 => {
                eprintln!("    Download failed (attempt {}): {}", attempt + 1, e);
                eprintln!("    Retrying in 5 seconds...");
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Download a file from URL (single attempt)
fn download_file_once(url: &str, output_path: &Path) -> Result<()> {
    let response = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(3600))
        .call()
        .with_context(|| format!("Failed to download {}", url))?;

    let mut file = File::create(output_path)?;
    let mut reader = response.into_reader();

    let mut buffer = [0u8; 65536];
    let mut total = 0usize;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                file.write_all(&buffer[..n])?;
                total += n;
                if total % (10 * 1024 * 1024) == 0 {
                    eprintln!("    Downloaded {} MB...", total / (1024 * 1024));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    eprintln!("    Total: {} MB", total / (1024 * 1024));
    Ok(())
}

/// Read ARG FASTA file and return gene names and sequences
fn read_arg_fasta(path: &Path) -> Result<Vec<(String, String)>> {
    use crate::seqio::FastaReader;

    let mut genes = Vec::new();
    let reader = FastaReader::open(path)?;

    for record in reader {
        let record = record?;
        // Extract gene name (first word of header, remove '>')
        let name = record.name.split_whitespace()
            .next()
            .unwrap_or(&record.name)
            .to_string();
        genes.push((name, record.seq));
    }

    Ok(genes)
}

/// Run BLAST for all genes against nt_prok with resume support
fn run_blast_all_genes(
    genes: &[(String, String)],
    config: &NtProkConfig,
    output_dir: &Path,
) -> Result<Vec<BlastHit>> {
    let blast_cache = output_dir.join("blast_cache");
    std::fs::create_dir_all(&blast_cache)?;

    let total = genes.len();
    let completed = AtomicUsize::new(0);
    let completed = Arc::new(completed);

    eprintln!("Running BLASTN against nt_prok for {} genes...", total);

    // Process genes sequentially (BLAST already uses internal parallelism)
    let all_hits: Vec<Vec<BlastHit>> = genes.iter()
        .map(|(name, seq)| {
            let hits = run_blast_single(name, seq, config, &blast_cache);
            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
            if done % 100 == 0 || done == total {
                eprintln!("  BLAST progress: {}/{} genes", done, total);
            }
            hits.unwrap_or_else(|e| {
                eprintln!("  Warning: BLAST failed for {}: {}", name, e);
                Vec::new()
            })
        })
        .collect();

    Ok(all_hits.into_iter().flatten().collect())
}

/// Run BLAST for a single gene with resume support
fn run_blast_single(
    gene_name: &str,
    gene_seq: &str,
    config: &NtProkConfig,
    cache_dir: &Path,
) -> Result<Vec<BlastHit>> {
    // Sanitize gene name for filename
    let safe_name: String = gene_name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();

    let result_file = cache_dir.join(format!("{}.blast.tsv", safe_name));
    let done_marker = cache_dir.join(format!("{}.blast.done", safe_name));

    // Resume support: skip if already completed
    if done_marker.exists() && result_file.exists() {
        return parse_blast_output(&result_file, gene_name);
    }

    // Create temp query file
    let query_file = cache_dir.join(format!("{}.query.fa", safe_name));
    {
        let mut f = File::create(&query_file)?;
        writeln!(f, ">{}", gene_name)?;
        writeln!(f, "{}", gene_seq)?;
    }

    // Run BLAST
    // outfmt: qseqid sseqid pident length mismatch gapopen qstart qend sstart send evalue bitscore staxids sscinames
    let output = Command::new(&config.blastn_path)
        .args([
            "-query", query_file.to_str().unwrap(),
            "-db", config.nt_prok_db.to_str().unwrap(),
            "-outfmt", "6 qseqid sseqid pident length mismatch gapopen qstart qend sstart send evalue bitscore staxids sscinames",
            "-perc_identity", &config.blast_identity.to_string(),
            "-qcov_hsp_perc", &config.blast_qcov.to_string(),
            "-max_target_seqs", "10000000",
            "-evalue", "1e-10",
            "-num_threads", "4",  // Per-gene parallelism
            "-out", result_file.to_str().unwrap(),
        ])
        .output()
        .with_context(|| format!("Failed to run blastn for gene {}", gene_name))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("BLAST failed for {}: {}", gene_name, stderr);
    }

    // Create done marker
    File::create(&done_marker)?;

    // Clean up query file
    let _ = std::fs::remove_file(&query_file);

    parse_blast_output(&result_file, gene_name)
}

/// Parse BLAST outfmt 6 output with sscinames field
fn parse_blast_output(path: &Path, _expected_gene: &str) -> Result<Vec<BlastHit>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut hits = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        // Fields: qseqid sseqid pident length mismatch gapopen qstart qend sstart send evalue bitscore staxids sscinames
        // Index:     0      1      2      3       4        5       6     7      8     9      10       11       12       13
        if fields.len() < 14 {
            continue;
        }

        let qseqid = fields[0].to_string();
        let sseqid = fields[1].to_string();
        let pident: f64 = fields[2].parse().unwrap_or(0.0);
        let sstart: usize = fields[8].parse().unwrap_or(0);
        let send: usize = fields[9].parse().unwrap_or(0);

        // staxids can have multiple values separated by ;
        let staxid: u32 = fields[12]
            .split(';')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let ssciname = fields[13].to_string();

        hits.push(BlastHit {
            qseqid,
            sseqid,
            pident,
            sstart,
            send,
            staxid,
            ssciname,
        });
    }

    Ok(hits)
}

/// Calculate upstream/downstream coordinates (strand-aware)
/// Returns (upstream_start, upstream_end, downstream_start, downstream_end, is_minus)
fn calculate_flanking_coords(
    sstart: usize,
    send: usize,
    flanking_length: usize,
) -> (usize, usize, usize, usize, bool) {
    let is_minus = sstart > send;
    let (hit_start, hit_end) = if is_minus {
        (send, sstart)  // Swap for minus strand
    } else {
        (sstart, send)
    };

    let upstream_start = hit_start.saturating_sub(flanking_length);
    let upstream_end = hit_start.saturating_sub(1);
    let downstream_start = hit_end + 1;
    let downstream_end = hit_end + flanking_length;

    (upstream_start, upstream_end, downstream_start, downstream_end, is_minus)
}

/// Extract clean accession from BLAST sseqid
/// Handles formats like:
/// - "gi|1787534517|gb|CP047050.1|" -> "CP047050.1"
/// - "gi|123|ref|NC_001234.1|" -> "NC_001234.1"
/// - "CP047050.1" -> "CP047050.1" (already clean)
fn extract_accession(sseqid: &str) -> String {
    // Check for gi| format: gi|NUMBER|db|ACCESSION|
    if sseqid.starts_with("gi|") {
        let parts: Vec<&str> = sseqid.split('|').collect();
        // Format is: gi | number | db_type | accession | (optional empty)
        if parts.len() >= 4 {
            return parts[3].to_string();
        }
    }
    // Return as-is if not gi format
    sseqid.to_string()
}

/// Reverse complement a DNA sequence
fn reverse_complement(seq: &str) -> String {
    seq.chars()
        .rev()
        .map(|c| match c {
            'A' | 'a' => 'T',
            'T' | 't' => 'A',
            'G' | 'g' => 'C',
            'C' | 'c' => 'G',
            'N' | 'n' => 'N',
            other => other,
        })
        .collect()
}

/// Extract flanking sequences using blastdbcmd in batches
fn extract_flanking_batch(
    hits: &[BlastHit],
    config: &NtProkConfig,
    output_dir: &Path,
) -> Result<FxHashMap<String, FlankingSeqs>> {
    const BATCH_SIZE: usize = 100;  // Memory management: 100 entries per blastdbcmd call

    let temp_dir = output_dir.join("blastdbcmd_temp");
    std::fs::create_dir_all(&temp_dir)?;

    let mut results: FxHashMap<String, FlankingSeqs> = FxHashMap::default();
    let total_batches = (hits.len() + BATCH_SIZE - 1) / BATCH_SIZE;

    eprintln!("Extracting flanking sequences ({} hits in {} batches)...", hits.len(), total_batches);

    for (batch_idx, chunk) in hits.chunks(BATCH_SIZE).enumerate() {
        if (batch_idx + 1) % 10 == 0 || batch_idx + 1 == total_batches {
            eprintln!("  Extraction progress: batch {}/{}", batch_idx + 1, total_batches);
        }

        // Create batch files for upstream and downstream extraction
        let batch_upstream = temp_dir.join(format!("batch_{}_upstream.txt", batch_idx));
        let batch_downstream = temp_dir.join(format!("batch_{}_downstream.txt", batch_idx));

        {
            let mut up_writer = BufWriter::new(File::create(&batch_upstream)?);
            let mut down_writer = BufWriter::new(File::create(&batch_downstream)?);

            for hit in chunk {
                let (up_start, up_end, down_start, down_end, _) =
                    calculate_flanking_coords(hit.sstart, hit.send, config.flanking_length);

                let accession = extract_accession(&hit.sseqid);

                // blastdbcmd format: accession range (1-based)
                if up_end > up_start {
                    writeln!(up_writer, "{} {}-{}", accession, up_start.max(1), up_end)?;
                }
                if down_end > down_start {
                    writeln!(down_writer, "{} {}-{}", accession, down_start, down_end)?;
                }
            }
        }

        // Run blastdbcmd for upstream
        let up_output = temp_dir.join(format!("batch_{}_upstream.fa", batch_idx));
        let _ = Command::new(&config.blastdbcmd_path)
            .args([
                "-db", config.nt_prok_db.to_str().unwrap(),
                "-entry_batch", batch_upstream.to_str().unwrap(),
                "-outfmt", "%a|%s",
                "-out", up_output.to_str().unwrap(),
            ])
            .output();

        // Run blastdbcmd for downstream
        let down_output = temp_dir.join(format!("batch_{}_downstream.fa", batch_idx));
        let _ = Command::new(&config.blastdbcmd_path)
            .args([
                "-db", config.nt_prok_db.to_str().unwrap(),
                "-entry_batch", batch_downstream.to_str().unwrap(),
                "-outfmt", "%a|%s",
                "-out", down_output.to_str().unwrap(),
            ])
            .output();

        // Parse outputs and associate with hits
        let up_seqs = parse_blastdbcmd_output(&up_output).unwrap_or_default();
        let down_seqs = parse_blastdbcmd_output(&down_output).unwrap_or_default();

        for hit in chunk {
            let hit_key = format!("{}:{}:{}-{}", hit.qseqid, hit.sseqid, hit.sstart, hit.send);
            let is_minus = hit.is_minus_strand();

            let accession = extract_accession(&hit.sseqid);
            let mut upstream = up_seqs.get(&accession).cloned().unwrap_or_default();
            let mut downstream = down_seqs.get(&accession).cloned().unwrap_or_default();

            // For minus strand: swap and reverse-complement
            if is_minus {
                std::mem::swap(&mut upstream, &mut downstream);
                upstream = reverse_complement(&upstream);
                downstream = reverse_complement(&downstream);
            }

            results.insert(hit_key, FlankingSeqs { upstream, downstream });
        }

        // Cleanup batch files
        let _ = std::fs::remove_file(&batch_upstream);
        let _ = std::fs::remove_file(&batch_downstream);
        let _ = std::fs::remove_file(&up_output);
        let _ = std::fs::remove_file(&down_output);
    }

    Ok(results)
}

/// Parse blastdbcmd output (format: accession|sequence per line)
fn parse_blastdbcmd_output(path: &Path) -> Result<FxHashMap<String, String>> {
    let mut seqs = FxHashMap::default();

    if !path.exists() {
        return Ok(seqs);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if let Some((acc, seq)) = line.split_once('|') {
            seqs.insert(acc.to_string(), seq.to_string());
        }
    }

    Ok(seqs)
}

/// Generate TSV compatible with fdb::build()
/// Note: NO strand column - sequences are pre-oriented during extraction
fn write_flanking_tsv(
    hits: &[BlastHit],
    flanking_seqs: &FxHashMap<String, FlankingSeqs>,
    taxonomy: &TaxonomyDb,
    output_path: &Path,
) -> Result<usize> {
    let mut writer = BufWriter::new(File::create(output_path)?);

    // TSV header (must match existing format: 7 columns, NO strand)
    writeln!(writer, "Gene\tContig\tGenus\tStart\tEnd\tUpstream\tDownstream")?;

    let mut count = 0;
    for hit in hits {
        let hit_key = format!("{}:{}:{}-{}", hit.qseqid, hit.sseqid, hit.sstart, hit.send);
        let seqs = flanking_seqs.get(&hit_key).cloned().unwrap_or_default();

        // Skip if no flanking sequences extracted
        if seqs.upstream.is_empty() && seqs.downstream.is_empty() {
            continue;
        }

        // Taxonomy resolution with sscinames fallback
        let genus = taxonomy.get_genus(hit.staxid)
            .or_else(|| {
                // Fallback: extract genus from sscinames if taxdump lookup fails
                if !hit.ssciname.is_empty() && hit.ssciname != "N/A" {
                    TaxonomyDb::extract_genus_from_sciname(&hit.ssciname)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        // Use normalized coordinates (always start < end)
        let (start, end) = hit.normalized_coords();

        writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            hit.qseqid,      // Gene
            hit.sseqid,      // Contig
            genus,           // Genus (with sscinames fallback)
            start,           // Start (normalized)
            end,             // End (normalized)
            seqs.upstream,   // Upstream (pre-oriented)
            seqs.downstream, // Downstream (pre-oriented)
        )?;
        count += 1;
    }

    Ok(count)
}

/// Main entry point for 5000bp FDB building
pub fn build(
    output_dir: &Path,
    arg_db: &Path,
    config: NtProkConfig,
) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    // Step 1: Validate ARG database format
    eprintln!("Validating ARG database format...");
    validate_arg_db_format(arg_db)?;
    eprintln!("  ARG database: {} (FASTA format verified)", arg_db.display());

    // Step 2: Ensure taxdump exists (auto-download if needed)
    ensure_taxdump(&config.taxdump_dir)?;
    let taxonomy = TaxonomyDb::load(&config.taxdump_dir)?;

    // Step 3: Read ARG sequences
    eprintln!("Reading ARG sequences from {}...", arg_db.display());
    let genes = read_arg_fasta(arg_db)?;
    eprintln!("  Loaded {} genes", genes.len());

    // Step 4: Run BLAST
    let hits = run_blast_all_genes(&genes, &config, output_dir)?;
    eprintln!("  Found {} BLAST hits", hits.len());

    if hits.is_empty() {
        eprintln!("Warning: No BLAST hits found. Check nt_prok database path.");
        return Ok(());
    }

    // Step 5: Extract flanking sequences
    let flanking_seqs = extract_flanking_batch(&hits, &config, output_dir)?;
    eprintln!("  Extracted flanking for {} hits", flanking_seqs.len());

    // Step 6: Write TSV
    let tsv_path = output_dir.join("flanking_5000bp.tsv");
    let count = write_flanking_tsv(&hits, &flanking_seqs, &taxonomy, &tsv_path)?;
    eprintln!("  Wrote {} records to {}", count, tsv_path.display());

    // Step 7: Build FDB using existing fdb::build
    let fdb_path = output_dir.join("flanking_5000bp.fdb");
    eprintln!("Building FDB index...");
    crate::fdb::build(&tsv_path, &fdb_path, 1024, config.threads)?;
    eprintln!("  Created: {}", fdb_path.display());

    eprintln!();
    eprintln!("============================================================");
    eprintln!(" 5000bp Flanking Database Build Complete!");
    eprintln!("============================================================");
    eprintln!("  TSV: {}", tsv_path.display());
    eprintln!("  FDB: {}", fdb_path.display());
    eprintln!("  Records: {}", count);

    Ok(())
}
