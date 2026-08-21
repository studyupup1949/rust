mod seqio;
mod paf;
mod extender;
mod classifier;
mod snp;
mod flanking_db;
mod arg_db;
mod fdb;
mod flanking_db_ntprok;
mod reassemble;

use anyhow::{Context, Result};
use clap::Parser;
use rustc_hash::FxHashSet;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use seqio::{FastaReader, FastaRecord, FastqFile};
use paf::PafReader;
use extender::{ContigExtender, ExtenderConfig, write_extended_contigs};
use classifier::{ArgPosition, GenusResult, GenusClassifier};

/// Parse and validate ARG identity threshold (must be >= 0.8)
fn parse_arg_identity(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| format!("Invalid number: {}", s))?;
    if !(0.8..=1.0).contains(&val) {
        Err(format!("ARG identity must be between 0.8 and 1.0, got {}", val))
    } else {
        Ok(val)
    }
}

/// Parse and validate ARG coverage threshold (must be >= 0.7)
fn parse_arg_coverage(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| format!("Invalid number: {}", s))?;
    if !(0.7..=1.0).contains(&val) {
        Err(format!("ARG coverage must be between 0.7 and 1.0, got {}", val))
    } else {
        Ok(val)
    }
}

#[derive(Parser)]
#[command(name = "argenus")]
#[command(version)]
#[command(about = "ARG detection and genus classification from metagenomic reads")]
#[command(long_about = r#"
argenus - Antibiotic Resistance Gene detection with GENUS classification

A targeted assembly pipeline that:
  1. Filters reads matching ARG database
  2. Assembles filtered reads with MEGAHIT (parallel processing)
  3. Extends contigs using k-mer overlap
  4. Detects ARGs and classifies source genus using flanking sequences

WORKFLOW:
  Reads → read filter (strobealign/minimap2/bwa-mem2) → MEGAHIT assembly → Extension → ARG detection → Genus classification

ALIGNMENT TIE-BREAKING (for equal-score hits):
  Priority: Score (higher first) → Gene length (higher first) → MapQ (higher first)
            → Divergence (lower first) → Gap count (lower first) → Gene name (alphabetical)

OUTPUT FILES:
  results.tsv          Main output with detected ARGs and genus assignments
    Columns: Sample, Contig_ID, ARG_Name, ARG_Class, Genus, Confidence, Specificity,
             ARG_Identity, ARG_Coverage, Contig_Len, Upstream_Len,
             Downstream_Len, Extension_Method, Top_Matches

  {sample}/            Per-sample directory (kept with -u flag)
    contigs_strict.fasta   Extended contigs (k-mer overlap)
    contigs_to_argdb.paf   Contig-to-ARG alignments
    megahit/               MEGAHIT assembly output

INPUT MODES:
  Single   One FASTQ pair (-1 R1.fq -2 R2.fq)
  Batch    Directory or ID list file (-l), auto-finds {id}_R[12].fastq.gz

EXAMPLES:
  # Single sample
  argenus -1 R1.fq -2 R2.fq -a AMR_NCBI.mmi -f flanking.fdb -o output/

  # Batch: directory (auto-detect all FASTQ pairs)
  argenus -l /path/to/fastq_dir/ -a AMR_NCBI.mmi -f flanking.fdb -o results/

  # Batch: ID list file (one sample ID per line)
  argenus -l samples.txt -a AMR_NCBI.mmi -f flanking.fdb -o results/
"#)]
#[command(after_help = r#"
For more information, visit: https://github.com/necoli1822/ARGenus
"#)]
struct Args {
    // ===== INPUT OPTIONS =====
    /// Forward reads (FASTQ/FASTQ.GZ), comma-separated for multiple samples
    #[arg(short = '1', long, value_name = "FILE(S)", help_heading = "Input")]
    r1: Option<String>,

    /// Reverse reads (FASTQ/FASTQ.GZ), comma-separated for multiple samples
    #[arg(short = '2', long, value_name = "FILE(S)", help_heading = "Input")]
    r2: Option<String>,

    /// Sample list: file (IDs, one per line) or directory (auto-detect FASTQs)
    #[arg(short = 'l', long, value_name = "PATH", help_heading = "Input")]
    samples: Option<PathBuf>,

    // ===== DATABASE OPTIONS =====
    /// Build database: 'arg' (AMR sequences from NCBI/CARD) or 'flank' (flanking DB)
    #[arg(short = 'b', long = "build-db", value_name = "TYPE", help_heading = "Database")]
    build_db: Option<String>,

    /// Data source for ARG database: 'ncbi', 'card', 'panres', or 'unified' (default: ncbi).
    /// - ncbi: NCBI AMRFinderPlus (~8,000 genes, curated)
    /// - card: CARD database (~6,000 genes, NCBI-mapped + CARD-only marked with '^')
    /// - panres: PanRes combined database (~14,000 genes from ARGprofiler)
    /// - unified: Use a pre-built unified ARG database (requires --unified-db)
    #[arg(short = 'x', long, value_name = "SOURCE", default_value = "ncbi", help_heading = "Database")]
    source: String,

    /// Path to pre-built unified ARG database FASTA (required when --source unified).
    /// The FASTA should have headers in format: >ARO_ID|gene_name|source|length
    #[arg(long = "unified-db", value_name = "FILE", help_heading = "Database")]
    unified_db: Option<PathBuf>,

    /// ARG reference database (.mmi from 'argenus -b arg', or custom .fas)
    #[arg(short = 'a', long = "arg-db", value_name = "FILE", help_heading = "Database")]
    arg_db: Option<PathBuf>,

    /// Flanking sequence database (.fdb format) for genus classification
    #[arg(short = 'f', long = "flanking-db", value_name = "FILE", help_heading = "Database")]
    flanking_db: Option<PathBuf>,

    /// Database directory: auto-discovers the ARG reference (.mmi or FASTA), the
    /// flanking DB (.fdb), and the optional plasmid_contigs.txt / contig_species.tsv
    /// inside it — a one-flag alternative to giving -a/-f separately. Explicit
    /// -a/-f/--plasmid-contigs/--species-map still override what's found here.
    #[arg(short = 'd', long = "db-dir", value_name = "DIR", help_heading = "Database")]
    db_dir: Option<PathBuf>,

    /// NCBI email for API access (required for --build-db flank)
    /// Provides higher rate limits (10 req/s vs 5 req/s)
    #[arg(short = 'e', long, value_name = "EMAIL", help_heading = "Database")]
    email: Option<String>,

    /// Flanking region length in bp (default: 1000)
    /// Used for --build-db flank to extract upstream/downstream sequences
    #[arg(short = 'p', long = "flanking-length", value_name = "BP", default_value = "1000", help_heading = "Database")]
    flanking_length: usize,

    /// Download queue buffer size in GB for --build-db flank (default: 30)
    /// Controls backpressure when alignment is slower than download
    #[arg(short = 'q', long = "queue-buffer", value_name = "GB", default_value = "30", help_heading = "Database")]
    queue_buffer_gb: u32,

    /// Pre-downloaded PLSDB directory (contains meta.tar.gz and sequences.fasta)
    /// Use this if PLSDB server is slow or unreliable
    #[arg(short = 'd', long = "plsdb-dir", value_name = "DIR", help_heading = "Database")]
    plsdb_dir: Option<PathBuf>,

    /// Skip PLSDB plasmid sequences (use only NCBI genomes)
    #[arg(short = 'z', long = "skip-plsdb", help_heading = "Database")]
    skip_plsdb: bool,

    /// Flanking database build mode: 'short' (1000bp, GenBank/PLSDB) or 'long' (5000bp, nt_prok)
    #[arg(long = "mode", value_name = "MODE", default_value = "short", help_heading = "Database")]
    fdb_mode: String,

    /// Path to blastn executable (required for --mode long)
    #[arg(long = "blastn-path", value_name = "PATH", help_heading = "Database")]
    blastn_path: Option<PathBuf>,

    /// Path to blastdbcmd executable (required for --mode long)
    #[arg(long = "blastdbcmd-path", value_name = "PATH", help_heading = "Database")]
    blastdbcmd_path: Option<PathBuf>,

    /// Path to nt_prok BLAST database (required for --mode long)
    #[arg(long = "nt-prok-db", value_name = "PATH", help_heading = "Database")]
    nt_prok_db: Option<PathBuf>,

    /// Path to NCBI taxdump directory (optional, auto-downloads to output/taxonomy if not specified)
    #[arg(long = "taxdump-dir", value_name = "PATH", help_heading = "Database")]
    taxdump_dir: Option<PathBuf>,

    /// For -b fdb: input TSV is already sorted by gene name (streaming mode, memory-efficient)
    #[arg(long = "sorted", help_heading = "Database")]
    sorted: bool,

    // ===== OUTPUT OPTIONS =====
    /// Output directory (created if not exists)
    #[arg(short = 'o', long, value_name = "DIR", default_value = ".", help_heading = "Output")]
    outdir: PathBuf,

    /// Keep intermediate files (filtered reads, contigs, PAF files)
    #[arg(short = 'u', long, help_heading = "Output")]
    keep_temp: bool,

    /// Verbose output to stderr (progress and statistics)
    #[arg(short = 'v', long, help_heading = "Output")]
    verbose: bool,

    /// Include all hits (WildType and NotCovered) in output
    /// By default, only true resistance genes are reported (Acquired, Confirmed, Novel)
    #[arg(long = "all-hits", help_heading = "Output")]
    all_hits: bool,

    // ===== ARG DETECTION =====
    /// Minimum identity for ARG detection [0.8-1.0]
    #[arg(short = 'i', long = "arg-identity", value_name = "FLOAT",
          default_value = "0.80", value_parser = parse_arg_identity, help_heading = "ARG Detection")]
    arg_identity: f64,

    /// Minimum query coverage for ARG detection [0.7-1.0]
    #[arg(short = 'c', long = "arg-coverage", value_name = "FLOAT",
          default_value = "0.70", value_parser = parse_arg_coverage, help_heading = "ARG Detection")]
    arg_coverage: f64,

    // ===== GENUS CLASSIFICATION =====
    /// Minimum specificity for genus assignment [0-100%]
    #[arg(short = 'r', long, value_name = "PERCENT", default_value = "95", help_heading = "Genus Classification")]
    resolution: f64,

    /// Maximum flanking sequence length to extract (bp)
    #[arg(short = 'n', long, value_name = "BP", default_value = "1000", help_heading = "Genus Classification")]
    max_flanking: usize,

    /// Cap on reference flanks aligned per locus [0 = unlimited]. A few genes carry up
    /// to ~128k flanks; a positive cap speeds classification (evenly-strided sample) but
    /// shifts the genus/species call on those heavy loci, so it is off (0) by default.
    #[arg(long = "max-ref-flanks", value_name = "N", default_value = "0", help_heading = "Genus Classification")]
    max_ref_flanks: usize,

    /// Target coverage for the conformal credible set: the rate at which the true host's
    /// clade must fall inside the reported Credible_Set. Higher = more certain but broader
    /// answers (more genera, coarser Resolution_Rank). Picks the calibrated threshold from
    /// the database's conformal.tsv; without that table the set is uncalibrated (0.9 mass).
    #[arg(long = "target-coverage", value_name = "P", default_value = "0.90", help_heading = "Genus Classification")]
    target_coverage: f64,

    // ===== ASSEMBLY & EXTENSION =====
    /// Minimum contig length to keep (bp)
    #[arg(short = 'g', long, value_name = "BP", default_value = "200", help_heading = "Assembly")]
    min_contig_len: usize,

    /// Cap on bp added to EACH contig end during extension [0 = auto 2×max-flanking].
    /// Classification uses only ~max-flanking of flanking, so capping stops runaway
    /// contigs (which otherwise force many extra read-rescan rounds) with no loss.
    /// Set very high to disable the cap.
    #[arg(long = "max-extension", value_name = "BP", default_value = "0", help_heading = "Assembly")]
    max_extension: usize,

    /// K-mer size for contig extension [31-127, odd]
    #[arg(short = 'k', long, value_name = "SIZE", default_value = "62", help_heading = "Assembly")]
    ext_kmer_size: usize,

    /// Extension step length (bp)
    #[arg(short = 'j', long, value_name = "BP", default_value = "100", help_heading = "Assembly")]
    ext_length: usize,

    /// Cap read depth in contig INTERIORS before extension [0 = off].
    /// The extender only uses reads matching contig-edge k-mers, so interior
    /// reads (>= --end-zone bp from both ends) are redundant. Terminal-zone and
    /// unmapped reads are always kept; interior coverage is capped to this value.
    /// Cuts extension memory on high-abundance loci without losing flanking.
    #[arg(long = "cap-interior", value_name = "X", default_value = "0", help_heading = "Assembly")]
    cap_interior: usize,

    /// Distance from each contig end kept at full depth for --cap-interior (bp)
    #[arg(long = "end-zone", value_name = "BP", default_value = "150", help_heading = "Assembly")]
    end_zone: usize,

    /// Classify a pre-assembled contig FASTA (skip read filtering/assembly).
    /// Runs only ARG detection + genus classification + unresolved exports on the
    /// given contigs and writes results.tsv. Enables A/B testing of alternative
    /// assemblies (e.g. per-locus reassembly) without rerunning the read pipeline.
    #[arg(long = "classify-contigs", value_name = "FASTA", help_heading = "Assembly")]
    classify_contigs: Option<PathBuf>,

    /// List of plasmid source-contig accessions (one per line), used to report the
    /// replicon Context (plasmid/chromosome) of each ARG's flanking. When the
    /// flanking mostly matches plasmid-derived references the genus is unreliable
    /// (mobile element). Without this, Context is reported as "NA".
    #[arg(long = "plasmid-contigs", value_name = "FILE", help_heading = "Classification")]
    plasmid_contigs: Option<PathBuf>,

    /// Contig→species map (TSV: contig<TAB>species) for species-level reporting.
    /// Auto-loaded from contig_species.tsv beside the flanking DB if present.
    #[arg(long = "species-map", value_name = "FILE", help_heading = "Classification")]
    species_map: Option<PathBuf>,

    /// Minimum flanking identity to distinguish GENERA [0.8-1.0].
    #[arg(long = "genus-identity", value_name = "F", default_value = "0.90", help_heading = "Classification")]
    genus_identity: f64,

    /// Minimum flanking identity to call SPECIES (higher than genus; species need
    /// near-identical flanking). Set 0 to disable species reporting. Default 0.96
    /// tuned on the low benchmark: recovers over-strict cases while keeping
    /// chromosome-context species precision ~94%.
    #[arg(long = "species-identity", value_name = "F", default_value = "0.96", help_heading = "Classification")]
    species_identity: f64,

    /// Context call: plasmid-fraction of matched flanking >= this → "plasmid".
    /// Tuned 0.5 on the low benchmark (plasmid calls ~95% correct).
    #[arg(long = "context-plasmid-frac", value_name = "F", default_value = "0.5", help_heading = "Classification")]
    context_plasmid_frac: f64,

    /// Context call: plasmid-fraction of matched flanking <= this → "chromosome".
    /// Between the two thresholds → "ambiguous". Tuned 0.1 (chromosome ~90% correct).
    #[arg(long = "context-chromosome-frac", value_name = "F", default_value = "0.1", help_heading = "Classification")]
    context_chromosome_frac: f64,

    // ===== LOCUS OUTPUTS =====
    /// Emit per-locus FASTA outputs for selected classes. Comma list of any of:
    /// resolved (genus assigned), flanknomatch (gene in flank-DB but flank did not
    /// match), genenotindb (gene absent from flank-DB). Providing any --emit-*
    /// flag turns the feature on; an unspecified dimension defaults to all.
    #[arg(long = "emit-class", value_delimiter = ',', value_name = "LIST", help_heading = "Locus outputs")]
    emit_class: Vec<String>,

    /// Parts to emit: comma list of gene (ARG-DB-matched region), flank (terminal
    /// flanking). Default: both (when the feature is on).
    #[arg(long = "emit-part", value_delimiter = ',', value_name = "LIST", help_heading = "Locus outputs")]
    emit_part: Vec<String>,

    /// States to emit: comma list of asm (assembled contig sequence), reads (reads
    /// mapping to the region). Default: both. 'reads' needs the read pipeline
    /// (ignored with a warning in --classify-contigs).
    #[arg(long = "emit-state", value_delimiter = ',', value_name = "LIST", help_heading = "Locus outputs")]
    emit_state: Vec<String>,

    // ===== READ FILTERING =====
    /// Read-filter aligner: 'strobealign' (default), 'minimap2', or 'bwa-mem2'.
    /// strobealign/bwa-mem2 require a FASTA reference (see --ref-fasta); they map
    /// R1+R2 as pairs, emit SAM, and it is converted to PAF so the downstream
    /// identity/block-length filter is identical to minimap2.
    /// Use '--mapper minimap2' to restore the original minimap2 read filter.
    #[arg(long = "mapper", value_name = "TOOL", default_value = "strobealign",
          value_parser = ["minimap2", "strobealign", "bwa-mem2"], help_heading = "Read Filtering")]
    mapper: String,

    /// FASTA reference for --mapper strobealign|bwa-mem2 (they cannot read a .mmi).
    /// If omitted, argenus tries to derive it from --arg-db (e.g. foo.mmi -> foo.fa/.fas/.fasta).
    #[arg(long = "ref-fasta", value_name = "FILE", help_heading = "Read Filtering")]
    ref_fasta: Option<PathBuf>,

    /// Path to strobealign executable (default: search PATH)
    #[arg(long = "strobealign-path", value_name = "FILE", help_heading = "Read Filtering")]
    strobealign_path: Option<PathBuf>,

    /// Path to bwa-mem2 executable (default: search PATH)
    #[arg(long = "bwa-mem2-path", value_name = "FILE", help_heading = "Read Filtering")]
    bwa_mem2_path: Option<PathBuf>,

    /// Path to paftools.sh for SAM->PAF (optional; a built-in converter is used if absent)
    #[arg(long = "paftools-path", value_name = "FILE", help_heading = "Read Filtering")]
    paftools_path: Option<PathBuf>,

    /// Minimum alignment identity for read filtering [0.0-1.0]
    #[arg(short = 'm', long, value_name = "FLOAT", default_value = "0.80", help_heading = "Read Filtering")]
    identity: f64,

    /// Minimum alignment length for read filtering (bp)
    #[arg(short = 'w', long, value_name = "BP", default_value = "50", help_heading = "Read Filtering")]
    min_align_len: usize,

    // ===== RUNTIME =====
    /// Number of threads [0 = auto-detect]
    #[arg(short = 't', long, value_name = "NUM", default_value = "0", help_heading = "Runtime")]
    threads: usize,

    /// Threads per sample for parallel processing (default: 8)
    #[arg(short = 's', long, value_name = "NUM", default_value = "8", help_heading = "Runtime")]
    threads_per_sample: usize,

    /// Skip confirmation prompts (auto-yes)
    #[arg(short = 'y', long, help_heading = "Runtime")]
    yes: bool,

    /// Per-locus reassembly (v3 core/flank read split + SPAdes) for STALLED loci
    /// (short flanking, non-plasmid). Opt-in: recovers a classifiable flanking for
    /// loci that are NA on every axis. Requires --spades-path (or the default pixi
    /// SPAdes). Plasmid-context loci are skipped (reassembly can't fix a mobile-
    /// element genus).
    #[arg(long, help_heading = "Reassembly")]
    reassemble: bool,

    /// Path to spades.py (default: search PATH). Only used with --reassemble.
    #[arg(long, value_name = "FILE", default_value = "spades.py", help_heading = "Reassembly")]
    spades_path: PathBuf,

    /// Python interpreter to run SPAdes (its env python; the system python is often
    /// too old). Default: sibling python3 of --spades-path. Empty => call spades.py
    /// directly.
    #[arg(long, value_name = "FILE", help_heading = "Reassembly")]
    spades_python: Option<PathBuf>,

    /// Concurrent SPAdes jobs during reassembly.
    #[arg(long, value_name = "NUM", default_value = "4", help_heading = "Reassembly")]
    reassemble_jobs: usize,

    // Internal fields (not CLI options)
    /// Path to minimap2 (auto-detected)
    #[arg(skip)]
    minimap2: String,

    /// Path to megahit (auto-detected)
    #[arg(skip)]
    megahit: String,

    /// Resolved path to strobealign (set in main when --mapper strobealign)
    #[arg(skip)]
    strobealign: String,

    /// Resolved path to bwa-mem2 (set in main when --mapper bwa-mem2)
    #[arg(skip)]
    bwa_mem2: String,

    /// Path to blastn (auto-detected; --blastn-path overrides). Used for contig→ARG detection.
    #[arg(skip)]
    blastn: String,

    /// Path to makeblastdb (auto-detected). Builds the ARG BLAST DB for detection.
    #[arg(skip)]
    makeblastdb: String,

    /// Resolved FASTA reference for alt mappers (set in main)
    #[arg(skip)]
    ref_fasta_resolved: Option<PathBuf>,
}

/// Find executable in system PATH
fn find_executable(name: &str) -> Result<PathBuf> {
    // First check if it's an absolute path or in current directory
    let path = Path::new(name);
    if path.is_absolute() && path.exists() {
        return Ok(path.to_path_buf());
    }

    // Search in PATH
    if let Ok(paths) = env::var("PATH") {
        for dir in env::split_paths(&paths) {
            let full_path = dir.join(name);
            if full_path.exists() && full_path.is_file() {
                return Ok(full_path);
            }
        }
    }

    anyhow::bail!("{} not found in PATH. Please install it or add it to your PATH.", name)
}

/// Simple counting semaphore for limiting concurrent operations
struct Semaphore {
    count: Mutex<usize>,
    cvar: Condvar,
}

impl Semaphore {
    fn new(count: usize) -> Self {
        Semaphore {
            count: Mutex::new(count),
            cvar: Condvar::new(),
        }
    }

    fn acquire(&self) {
        let mut count = self.count.lock().unwrap();
        while *count == 0 {
            count = self.cvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    fn release(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        self.cvar.notify_one();
    }
}

/// Sample information
#[derive(Debug, Clone)]
struct Sample {
    name: String,
    r1: PathBuf,
    r2: PathBuf,
}

/// Final result row for output
#[derive(Debug, Clone)]
struct ResultRow {
    sample: String,
    contig_id: String,
    arg_name: String,
    arg_class: String,
    genus: String,
    confidence: f64,    // Jaccard similarity × 100 (0-100)
    specificity: f64,   // Gene specificity × 100 (0-100)
    identity: f64,
    coverage: f64,
    contig_len: usize,
    arg_start: usize,   // ARG span start on the contig (true coordinate, 0-based)
    arg_end: usize,     // ARG span end on the contig (true coordinate)
    upstream_len: usize,
    downstream_len: usize,
    extension_method: String,  // "strict" (tadpole) or "flexible" (rust extender)
    top_matches: String,
    snp_status: String,  // SNP verification status (internal; not emitted in RAPID output)
    context: String,     // replicon context: plasmid / chromosome / ambiguous / NA
    species: String,     // species call (or multi-species(N):... / Unknown)
    credible_set: String, // kernel credible set: "genusA(0.62);genusB(0.31)" to 0.9 mass
    support: f64,        // posterior mass of the credible set (UNCALIBRATED)
    resolution_distance: f64, // mash radius of the credible set (0 = single genus)
    limited_by: String,  // query / biology / none
    resolution_rank: String,  // LCA rank of the credible set (ragged: species..root)
    resolution_taxon: String, // LCA taxon name (e.g. Enterobacteriaceae)
}

/// Formats the reported genus from a classification result. When several genera are
/// near-tied (within GENUS_TIE_PCT of the top score), the ARG cannot be assigned to a
/// single genus (e.g. a promiscuous plasmid gene). We then report the TRUE count of
/// tied genera and list up to 5, e.g. "multi-genus(12):A/B/C/D/E(+7)".
fn format_genus_call(res: &GenusResult) -> String {
    let best = match &res.genus {
        Some(g) => g.clone(),
        None => return "Unknown".to_string(),
    };
    if res.n_genera_tied < 2 {
        return best;
    }
    let top = res.top_matches.first().map(|(_, s)| *s).unwrap_or(0.0);
    let shown: Vec<String> = res.top_matches.iter()
        .filter(|(g, s)| !g.is_empty() && *s >= top - classifier::GENUS_TIE_PCT)
        .map(|(g, _)| g.clone())
        .take(5)
        .collect();
    let remainder = res.n_genera_tied.saturating_sub(shown.len());
    if remainder > 0 {
        format!("multi-genus({}):{}(+{})", res.n_genera_tied, shown.join("/"), remainder)
    } else {
        format!("multi-genus({}):{}", res.n_genera_tied, shown.join("/"))
    }
}

/// Formats the species call, mirroring `format_genus_call`: single species, or
/// "multi-species(N):s1/s2/...(+K)" when several species are tied. "Unknown" when
/// no species cleared the species-identity threshold (e.g. plasmid/short flanking).
fn format_species_call(res: &GenusResult) -> String {
    let best = match &res.species {
        Some(s) => s.clone(),
        None => return "Unknown".to_string(),
    };
    if res.n_species_tied < 2 {
        return best;
    }
    let top = res.species_top_matches.first().map(|(_, s)| *s).unwrap_or(0.0);
    let shown: Vec<String> = res.species_top_matches.iter()
        .filter(|(s, sc)| !s.is_empty() && *sc >= top - classifier::GENUS_TIE_PCT)
        .map(|(s, _)| s.clone())
        .take(5)
        .collect();
    let remainder = res.n_species_tied.saturating_sub(shown.len());
    if remainder > 0 {
        format!("multi-species({}):{}(+{})", res.n_species_tied, shown.join("/"), remainder)
    } else {
        format!("multi-species({}):{}", res.n_species_tied, shown.join("/"))
    }
}

/// Detected ARG information with position
#[derive(Debug, Clone)]
struct ArgHit {
    arg_name: String,
    arg_class: String,
    contig: String,
    contig_len: usize,
    identity: f64,
    coverage: f64,
    contig_start: usize,
    contig_end: usize,
    strand: char,
    /// All PanRes reference IDs that tie at this locus (same identity & coverage as the
    /// representative). These are redundant DB representations of the same physical gene;
    /// classification unions their flanking reference sets so no source's evidence is lost.
    members: Vec<String>,
}

/// Validate ARG database file format
/// Returns (is_fasta, is_mmi) tuple
fn validate_arg_db_file(path: &Path) -> Result<(bool, bool)> {
    use std::io::{BufRead, BufReader, Read};

    let mut file = std::fs::File::open(path)?;

    // Read first 16 bytes for binary detection
    let mut header = [0u8; 16];
    let bytes_read = file.read(&mut header)?;

    if bytes_read < 4 {
        anyhow::bail!(
            "ARG database file too small: {}\n\
             Expected FASTA or minimap2 index file",
            path.display()
        );
    }

    // MMI files contain null bytes in header (binary format)
    if header[..bytes_read].contains(&0u8) {
        return Ok((false, true));
    }

    // Validate as FASTA: reopen and parse
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Find first non-empty line (should be header starting with '>')
    let header_line = loop {
        match lines.next() {
            Some(Ok(line)) if !line.trim().is_empty() => break line,
            Some(Ok(_)) => continue,
            Some(Err(e)) => anyhow::bail!("Failed to read {}: {}", path.display(), e),
            None => anyhow::bail!("Empty file: {}", path.display()),
        }
    };

    if !header_line.starts_with('>') {
        anyhow::bail!(
            "Invalid FASTA: first line must start with '>'\n\
             Found: {}\n\
             File: {}",
            &header_line[..header_line.len().min(50)],
            path.display()
        );
    }

    // Read sequence line(s)
    let seq_line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => anyhow::bail!("Failed to read sequence: {}", e),
        None => anyhow::bail!("Invalid FASTA: no sequence after header in {}", path.display()),
    };

    if seq_line.trim().is_empty() {
        anyhow::bail!("Invalid FASTA: empty sequence in {}", path.display());
    }

    // Validate nucleotide characters (ACGTN + IUPAC ambiguity codes)
    const VALID_NUCLEOTIDES: &[u8] = b"ACGTNacgtnRYSWKMBDHVryswkmbdhv";
    let invalid_count = seq_line.bytes()
        .filter(|b| !VALID_NUCLEOTIDES.contains(b))
        .count();

    if invalid_count > seq_line.len() / 10 {
        anyhow::bail!(
            "Invalid FASTA: too many non-nucleotide characters ({}/{}) in {}",
            invalid_count,
            seq_line.len(),
            path.display()
        );
    }

    Ok((true, false))
}

/// Handle --build-db command
fn handle_build_db(
    db_type: &str,
    source: &str,
    output_dir: &Path,
    threads: usize,
    email: Option<&str>,
    arg_db: Option<&Path>,
    unified_db: Option<&Path>,
    config: crate::flanking_db::FlankBuildConfig,
    fdb_mode: &str,
    sorted: bool,
    blastn_path: Option<&Path>,
    blastdbcmd_path: Option<&Path>,
    nt_prok_db: Option<&Path>,
    taxdump_dir: Option<&Path>,
) -> Result<()> {
    match db_type {
        "arg" => {
            let source_desc = match source {
                "ncbi" => "NCBI AMRFinderPlus",
                "card" => "CARD (Comprehensive Antibiotic Resistance Database)",
                "panres" => "PanRes (ARGprofiler combined database)",
                "unified" => "Pre-built unified ARG database",
                _ => {
                    anyhow::bail!("Unknown source '{}'. Use 'ncbi', 'card', 'panres', or 'unified'.", source);
                }
            };

            // For unified source, require --unified-db
            if source == "unified" {
                let unified_path = unified_db.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--unified-db is required when using --source unified.\n\
                         Example: argenus -b arg -x unified --unified-db /path/to/unified_arg_db.fasta -o ./db"
                    )
                })?;

                if !unified_path.exists() {
                    anyhow::bail!("Unified database file not found: {}", unified_path.display());
                }

                eprintln!("============================================================");
                eprintln!(" ARGenus Database Builder - Unified ARG Database");
                eprintln!("============================================================");
                eprintln!();
                eprintln!("Source: Pre-built unified ARG database");
                eprintln!("Input: {}", unified_path.display());
                eprintln!();
                return arg_db::build_from_unified(output_dir, unified_path, threads);
            }

            // Validate: -a should not be used with -b arg
            if arg_db.is_some() {
                anyhow::bail!(
                    "--arg-db (-a) is not used for ARG database build.\n\
                     Did you mean to build the flanking database?\n\n\
                     Build ARG database:     argenus -b arg -o ./db\n\
                     Build flanking database: argenus -b flank -a ./db/AMR_NCBI.mmi -o ./db -e email"
                );
            }

            eprintln!("============================================================");
            eprintln!(" ARGenus Database Builder - AMR Reference Database");
            eprintln!("============================================================");
            eprintln!();
            eprintln!("Source: {}", source_desc);
            eprintln!("This will download AMR sequences and build the reference database.");
            eprintln!();
            eprintln!("Threads: {}", threads);
            eprintln!();
            arg_db::build(output_dir, source, threads)
        }
        "flank" => {
            // Validate arg_db for flank build
            let arg_db = match arg_db {
                Some(p) => p,
                None => {
                    anyhow::bail!(
                        "--arg-db is required for flanking database build.\n\
                         First build the AMR database, then use it for flanking build:\n\n\
                         Step 1: argenus -b arg -o ./db\n\
                         Step 2: argenus -b flank -a ./db/AMR_NCBI.mmi -o ./db -e your@email.com\n\n\
                         Use .mmi (pre-built index) for faster processing, or .fas"
                    );
                }
            };

            // Validate that arg_db exists
            if !arg_db.exists() {
                anyhow::bail!(
                    "AMR database not found: {}\n\
                     Build it first with: argenus -b arg -o ./db",
                    arg_db.display()
                );
            }

            // Validate file content using proper parsers
            let (is_fasta, is_mmi) = validate_arg_db_file(arg_db)?;

            if !is_fasta && !is_mmi {
                anyhow::bail!(
                    "Invalid ARG database file: {}\n\
                     File must be FASTA (valid sequences) or minimap2 index (.mmi)\n\
                     Build with: argenus -b arg -o ./db",
                    arg_db.display()
                );
            }

            // Route based on mode
            match fdb_mode {
                "short" => {
                    // Ensure we have a .mmi index for efficient repeated minimap2 calls
                    let arg_db = if is_mmi {
                        // Already a minimap2 index, use directly
                        arg_db.to_path_buf()
                    } else {
                        // FASTA file - check for existing .mmi or build one
                        let mmi_path = arg_db.with_extension("mmi");
                        if mmi_path.exists() {
                            eprintln!("Using existing minimap2 index: {}", mmi_path.display());
                            mmi_path
                        } else {
                            eprintln!("Building minimap2 index for faster alignment...");
                            let output = std::process::Command::new("minimap2")
                                .args(["-d", mmi_path.to_str().unwrap(), arg_db.to_str().unwrap()])
                                .output();

                            match output {
                                Ok(o) if o.status.success() => {
                                    eprintln!("Created: {}", mmi_path.display());
                                    mmi_path
                                }
                                Ok(o) => {
                                    eprintln!("Warning: minimap2 indexing failed, using FASTA directly");
                                    eprintln!("{}", String::from_utf8_lossy(&o.stderr));
                                    arg_db.to_path_buf()
                                }
                                Err(e) => {
                                    eprintln!("Warning: minimap2 not found ({}), using FASTA directly", e);
                                    arg_db.to_path_buf()
                                }
                            }
                        }
                    };

                    // Validate email for short mode
                    let email = match email {
                        Some(e) => e,
                        None => {
                            anyhow::bail!(
                                "--email is required for --mode short (GenBank/PLSDB download).\n\
                                 Example: argenus -b flank --mode short -a ./db/AMR_NCBI.mmi -o ./db -e your@email.com\n\n\
                                 NCBI requires email for API access. Register at:\n\
                                 https://www.ncbi.nlm.nih.gov/account/"
                            );
                        }
                    };

                    eprintln!("============================================================");
                    eprintln!(" ARGenus Database Builder - Flanking Sequence Database");
                    eprintln!("============================================================");
                    eprintln!();
                    eprintln!("Mode: short (1000bp, GenBank/PLSDB)");
                    eprintln!();
                    eprintln!("This will build the flanking sequence database from genomic");
                    eprintln!("data. This is a resource-intensive process that requires:");
                    eprintln!("  - ~120GB of prokaryotic genome data (NCBI genomes)");
                    eprintln!("  - ~7GB PLSDB plasmid sequences (auto-downloaded)");
                    eprintln!("  - Several hours of processing time");
                    eprintln!("  - ~40GB of disk space for intermediate files");
                    eprintln!();
                    eprintln!("Pipeline:");
                    eprintln!("  1. Download NCBI taxonomy database");
                    eprintln!("  2. Download prokaryotic genomes (bacteria + archaea)");
                    eprintln!("  3. Download PLSDB plasmid sequences");
                    eprintln!("  4. Align AMR genes to genomes (minimap2)");
                    eprintln!("  5. Extract flanking sequences → TSV (~27 GB)");
                    eprintln!("  6. Build FDB (external sort + zstd) → ~350 MB");
                    eprintln!();
                    eprintln!("AMR Database: {}", arg_db.display());
                    eprintln!("NCBI Email: {}", email);
                    eprintln!("Threads: {}", threads);
                    eprintln!("Flanking length: {} bp", config.flanking_length);
                    eprintln!("Queue buffer: {} GB", config.queue_buffer_gb);
                    if let Some(ref dir) = config.plsdb.dir {
                        eprintln!("PLSDB: {} (pre-downloaded)", dir.display());
                    } else if config.plsdb.skip {
                        eprintln!("PLSDB: skipped");
                    } else {
                        eprintln!("PLSDB: auto-download from server");
                    }
                    eprintln!();
                    eprintln!("Note: This process takes several hours. Progress will be displayed.");
                    eprintln!();

                    // Existing GenBank/PLSDB workflow (1000bp)
                    arg_db::build_flanking_db(output_dir, &arg_db, threads, email, config)
                }
                "long" => {
                    // New nt_prok workflow (5000bp)
                    // Validate required paths
                    let blastn = blastn_path.ok_or_else(|| {
                        anyhow::anyhow!("--blastn-path is required for --mode long")
                    })?;
                    let blastdbcmd = blastdbcmd_path.ok_or_else(|| {
                        anyhow::anyhow!("--blastdbcmd-path is required for --mode long")
                    })?;
                    let nt_prok = nt_prok_db.ok_or_else(|| {
                        anyhow::anyhow!("--nt-prok-db is required for --mode long")
                    })?;
                    // Use user-specified taxdump or default to output_dir/taxonomy (auto-download)
                    let taxdump = taxdump_dir
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| output_dir.join("taxonomy"));

                    // Validate paths exist
                    if !blastn.exists() {
                        anyhow::bail!("blastn not found: {}", blastn.display());
                    }
                    if !blastdbcmd.exists() {
                        anyhow::bail!("blastdbcmd not found: {}", blastdbcmd.display());
                    }

                    // For --mode long, require FASTA (not .mmi)
                    // The .mmi format is a lossy index that cannot recover sequences
                    flanking_db_ntprok::validate_arg_db_format(&arg_db)?;

                    let ntprok_config = flanking_db_ntprok::NtProkConfig {
                        blastn_path: blastn.to_path_buf(),
                        blastdbcmd_path: blastdbcmd.to_path_buf(),
                        nt_prok_db: nt_prok.to_path_buf(),
                        taxdump_dir: taxdump.clone(),
                        flanking_length: 5000,
                        threads,
                        blast_identity: 95.0,
                        blast_qcov: 90.0,
                    };

                    eprintln!("============================================================");
                    eprintln!(" ARGenus Database Builder - Long Flanking (5000bp, nt_prok)");
                    eprintln!("============================================================");
                    eprintln!();
                    eprintln!("Mode: long (5000bp flanking via BLASTN against nt_prok)");
                    eprintln!("ARG Database: {}", arg_db.display());
                    eprintln!("BLASTN: {}", blastn.display());
                    eprintln!("blastdbcmd: {}", blastdbcmd.display());
                    eprintln!("nt_prok DB: {}", nt_prok.display());
                    eprintln!("taxdump: {}", taxdump.display());
                    eprintln!("Threads: {}", threads);
                    eprintln!();

                    flanking_db_ntprok::build(output_dir, &arg_db, ntprok_config)
                }
                other => {
                    anyhow::bail!(
                        "Invalid --mode '{}'. Use 'short' (1000bp, GenBank/PLSDB) or 'long' (5000bp, nt_prok).",
                        other
                    )
                }
            }
        }
        "fdb" => {
            // Build FDB directly from TSV
            let tsv_path = arg_db.ok_or_else(|| {
                anyhow::anyhow!(
                    "--arg-db is required for -b fdb (path to input TSV file).\n\
                     Example: argenus -b fdb -a flanking.tsv -o ./output/\n\
                     Use --sorted flag if TSV is pre-sorted by gene name (memory-efficient)"
                )
            })?;

            if !tsv_path.exists() {
                anyhow::bail!("TSV file not found: {}", tsv_path.display());
            }

            let fdb_path = output_dir.join("flanking.fdb");
            std::fs::create_dir_all(output_dir)?;

            eprintln!("============================================================");
            eprintln!(" ARGenus FDB Builder - Compress TSV to FDB");
            eprintln!("============================================================");
            eprintln!();
            eprintln!("Input TSV: {}", tsv_path.display());
            eprintln!("Output FDB: {}", fdb_path.display());

            if sorted {
                eprintln!("Mode: Streaming (pre-sorted input, memory-efficient)");
                eprintln!();
                crate::fdb::build_from_sorted(tsv_path, &fdb_path)?;
            } else {
                eprintln!("Mode: External sort (unsorted input)");
                eprintln!("Threads: {}", threads);
                eprintln!("Buffer: {} MB", config.queue_buffer_gb * 1024);
                eprintln!();
                crate::fdb::build(tsv_path, &fdb_path, (config.queue_buffer_gb * 1024) as usize, threads)?;
            }

            eprintln!();
            eprintln!("FDB build complete: {}", fdb_path.display());
            Ok(())
        }
        _ => {
            anyhow::bail!(
                "Unknown database type '{}'. Use 'arg', 'flank', or 'fdb'.\n\
                 Examples:\n  \
                   argenus -b arg -o ./db    # Build AMR reference database\n  \
                   argenus -b flank -o ./db  # Build flanking sequence database\n  \
                   argenus -b fdb -a in.tsv -o ./db  # Build FDB from TSV",
                db_type
            );
        }
    }
}

fn main() -> Result<()> {
    let mut args = Args::parse();
    let start_time = Instant::now();

    // Auto-detect threads first (needed for build-db too)
    if args.threads == 0 {
        args.threads = num_cpus::get();
    }

    // Handle --build-db mode
    if let Some(db_type) = &args.build_db {
        let config = crate::flanking_db::FlankBuildConfig {
            flanking_length: args.flanking_length,
            queue_buffer_gb: args.queue_buffer_gb,
            plsdb: crate::flanking_db::PlsdbOptions {
                dir: args.plsdb_dir.clone(),
                skip: args.skip_plsdb,
            },
        };
        return handle_build_db(
            db_type,
            &args.source,
            &args.outdir,
            args.threads,
            args.email.as_deref(),
            args.arg_db.as_deref(),
            args.unified_db.as_deref(),
            config,
            &args.fdb_mode,
            args.sorted,
            args.blastn_path.as_deref(),
            args.blastdbcmd_path.as_deref(),
            args.nt_prok_db.as_deref(),
            args.taxdump_dir.as_deref(),
        );
    }

    // Resolve --db-dir: fill in any DB path not given explicitly by discovering it
    // in the directory. Explicit -a/-f/etc. always win.
    if let Some(dir) = args.db_dir.clone() {
        if !dir.is_dir() {
            anyhow::bail!("--db-dir is not a directory: {}", dir.display());
        }
        if args.arg_db.is_none() {
            // Prefer a minimap2 index; fall back to a FASTA (plain or .gz — works
            // for both mappers).
            args.arg_db = find_db_file(&dir, &["mmi"])
                .or_else(|| find_fasta_file(&dir));
        }
        if args.flanking_db.is_none() {
            args.flanking_db = find_flanking_db(&dir)?;
        }
        if args.ref_fasta.is_none() {
            args.ref_fasta = find_fasta_file(&dir);
        }
        if args.plasmid_contigs.is_none() {
            let p = dir.join("plasmid_contigs.txt");
            if p.exists() { args.plasmid_contigs = Some(p); }
        }
        if args.species_map.is_none() {
            let p = dir.join("contig_species.tsv");
            if p.exists() { args.species_map = Some(p); }
        }
        if args.verbose {
            eprintln!("--db-dir {}: arg-db={:?} flanking-db={:?}",
                      dir.display(), args.arg_db, args.flanking_db);
        }
    }

    // Validate required arguments for analysis mode
    if args.arg_db.is_none() {
        anyhow::bail!("--arg-db is required for analysis mode (or use --db-dir)");
    }
    if args.flanking_db.is_none() {
        anyhow::bail!("--flanking-db is required for analysis mode (or use --db-dir)");
    }

    // Auto-load the plasmid contig list from beside the flanking DB (enables the
    // Context column by default). Looked up as plasmid_contigs.txt in the FDB's
    // directory or its parent. --plasmid-contigs overrides this.
    if args.plasmid_contigs.is_none() {
        if let Some(fdb) = args.flanking_db.clone() {
            let candidates = [
                fdb.parent().map(|d| d.join("plasmid_contigs.txt")),
                fdb.parent().and_then(|d| d.parent()).map(|d| d.join("plasmid_contigs.txt")),
            ];
            for cand in candidates.into_iter().flatten() {
                if cand.exists() {
                    if args.verbose { eprintln!("Auto-loaded plasmid list: {}", cand.display()); }
                    args.plasmid_contigs = Some(cand);
                    break;
                }
            }
        }
    }
    // Auto-load contig→species map from beside the flanking DB (enables Species column).
    if args.species_map.is_none() {
        if let Some(fdb) = args.flanking_db.clone() {
            let candidates = [
                fdb.parent().map(|d| d.join("contig_species.tsv")),
                fdb.parent().and_then(|d| d.parent()).map(|d| d.join("contig_species.tsv")),
            ];
            for cand in candidates.into_iter().flatten() {
                if cand.exists() {
                    if args.verbose { eprintln!("Auto-loaded species map: {}", cand.display()); }
                    args.species_map = Some(cand);
                    break;
                }
            }
        }
    }

    // Auto-detect external tools
    args.minimap2 = find_executable("minimap2")?.to_string_lossy().to_string();

    // Contig→ARG detection uses BLAST (dc-megablast); resolve blastn/makeblastdb the
    // same way as the other tools — honour --blastn-path if given, else search PATH.
    args.blastn = match &args.blastn_path {
        Some(p) => p.to_string_lossy().to_string(),
        None => find_executable("blastn")?.to_string_lossy().to_string(),
    };
    args.makeblastdb = find_executable("makeblastdb")?.to_string_lossy().to_string();
    if args.verbose {
        eprintln!("Found blastn: {}", args.blastn);
        eprintln!("Found makeblastdb: {}", args.makeblastdb);
    }

    // --classify-contigs only needs minimap2 (detection + classification); skip the
    // assembly/mapper toolchain and the read pipeline entirely.
    if let Some(contigs_fa) = args.classify_contigs.clone() {
        if args.verbose { eprintln!("Found minimap2: {}", args.minimap2); }
        return run_classify_contigs_mode(&contigs_fa, &args);
    }

    args.megahit = find_executable("megahit")?.to_string_lossy().to_string();

    if args.verbose {
        eprintln!("Found minimap2: {}", args.minimap2);
        eprintln!("Found megahit: {}", args.megahit);
    }

    // Resolve the read-filter mapper (Step 1). minimap2 is the default and keeps
    // the original behaviour; strobealign/bwa-mem2 are opt-in and need a FASTA ref.
    match args.mapper.as_str() {
        "strobealign" => {
            args.strobealign = match &args.strobealign_path {
                Some(p) => p.to_string_lossy().to_string(),
                None => find_executable("strobealign")?.to_string_lossy().to_string(),
            };
            args.ref_fasta_resolved = Some(resolve_ref_fasta(&args)?);
            if args.verbose {
                eprintln!("Found strobealign: {}", args.strobealign);
                eprintln!("Mapper ref FASTA: {:?}", args.ref_fasta_resolved);
            }
        }
        "bwa-mem2" => {
            args.bwa_mem2 = match &args.bwa_mem2_path {
                Some(p) => p.to_string_lossy().to_string(),
                None => find_executable("bwa-mem2")?.to_string_lossy().to_string(),
            };
            let ref_fa = resolve_ref_fasta(&args)?;
            // Build the index once up front so concurrent samples don't race on it.
            ensure_bwamem2_index(&args.bwa_mem2, &ref_fa)?;
            args.ref_fasta_resolved = Some(ref_fa);
            if args.verbose {
                eprintln!("Found bwa-mem2: {}", args.bwa_mem2);
                eprintln!("Mapper ref FASTA: {:?}", args.ref_fasta_resolved);
            }
        }
        _ => {}
    }

    // Configure rayon
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .ok();

    // Parse samples
    let samples = parse_samples(&args)?;
    if samples.is_empty() {
        anyhow::bail!("No samples provided. Use -1/-2 or --samples");
    }

    // Calculate concurrent sample count
    let max_concurrent = (args.threads / args.threads_per_sample).max(1);

    if args.verbose {
        eprintln!("Processing {} sample(s) with {} threads ({} concurrent, {} threads/sample)",
                  samples.len(), args.threads, max_concurrent, args.threads_per_sample);
    }

    // Create output directory
    fs::create_dir_all(&args.outdir)?;

    // Process samples in parallel with semaphore-based concurrency control
    let all_results: Arc<Mutex<Vec<ResultRow>>> = Arc::new(Mutex::new(Vec::new()));
    let semaphore = Semaphore::new(max_concurrent);
    let sample_counter = Mutex::new(0usize);
    let total_samples = samples.len();
    let args_ref = &args;

    std::thread::scope(|s| {
        for sample in &samples {
            let results = Arc::clone(&all_results);
            let sem = &semaphore;
            let counter = &sample_counter;

            s.spawn(move || {
                sem.acquire();

                let sample_num = {
                    let mut c = counter.lock().unwrap();
                    *c += 1;
                    *c
                };

                if args_ref.verbose {
                    eprintln!("\n=== Processing sample {}/{}: {} ===", sample_num, total_samples, sample.name);
                }

                match process_sample(sample, args_ref) {
                    Ok(res) => {
                        let mut all = results.lock().unwrap();
                        all.extend(res);
                    }
                    Err(e) => eprintln!("ERROR processing {}: {}", sample.name, e),
                }

                sem.release();
            });
        }
    });

    // Output results
    let final_results = Arc::try_unwrap(all_results)
        .expect("All threads should have finished")
        .into_inner()
        .unwrap();
    output_results(&final_results, &args)?;

    // Cleanup temp files (keep results)
    if !args.keep_temp {
        // Remove sample subdirectories but keep results file
        for sample in &samples {
            let sample_dir = args.outdir.join(&sample.name);
            let _ = fs::remove_dir_all(&sample_dir);
        }
    }

    if args.verbose {
        eprintln!("\nTotal time: {:.1}s", start_time.elapsed().as_secs_f64());
    }

    Ok(())
}

/// Auto-detect FASTQ pairs in a directory
fn find_samples_in_dir(dir: &Path) -> Result<Vec<Sample>> {
    use std::collections::BTreeSet;

    let mut sample_ids: BTreeSet<String> = BTreeSet::new();

    // R1 patterns to detect sample IDs
    let r1_suffixes = ["_R1.fastq.gz", "_R1.fq.gz", "_1.fastq.gz", "_1.fq.gz",
                       "_R1.fastq", "_R1.fq", "_1.fastq", "_1.fq"];

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Check if this is an R1 file and extract sample ID
        for suffix in &r1_suffixes {
            if filename.ends_with(suffix) {
                let id = filename.strip_suffix(suffix).unwrap();
                sample_ids.insert(id.to_string());
                break;
            }
        }
    }

    // Build sample list
    let mut samples = Vec::new();
    for id in sample_ids {
        let (r1, r2) = find_fastq_pair(dir, &id)?;
        samples.push(Sample {
            name: id,
            r1,
            r2,
        });
    }

    if samples.is_empty() {
        anyhow::bail!("No FASTQ pairs found in {:?}", dir);
    }

    Ok(samples)
}

/// Find FASTQ pair for a sample ID, trying common naming patterns
fn find_fastq_pair(base_dir: &Path, id: &str) -> Result<(PathBuf, PathBuf)> {
    // Common FASTQ naming patterns: {id}_R1.fastq.gz, {id}_1.fq.gz, etc.
    let patterns = [
        ("_R1.fastq.gz", "_R2.fastq.gz"),
        ("_R1.fq.gz", "_R2.fq.gz"),
        ("_1.fastq.gz", "_2.fastq.gz"),
        ("_1.fq.gz", "_2.fq.gz"),
        ("_R1.fastq", "_R2.fastq"),
        ("_R1.fq", "_R2.fq"),
        ("_1.fastq", "_2.fastq"),
        ("_1.fq", "_2.fq"),
    ];

    for (r1_suffix, r2_suffix) in &patterns {
        let r1 = base_dir.join(format!("{}{}", id, r1_suffix));
        let r2 = base_dir.join(format!("{}{}", id, r2_suffix));
        if r1.exists() && r2.exists() {
            return Ok((r1, r2));
        }
    }

    anyhow::bail!(
        "Cannot find FASTQ pair for '{}' in {:?}. Expected {}_R1.fastq.gz and {}_R2.fastq.gz",
        id, base_dir, id, id
    )
}

fn parse_samples(args: &Args) -> Result<Vec<Sample>> {
    let mut samples = Vec::new();

    if let Some(ref samples_path) = args.samples {
        if samples_path.is_dir() {
            // Auto-detect FASTQ pairs in directory
            samples = find_samples_in_dir(samples_path)?;
        } else {
            // Read sample IDs from file (one ID per line)
            let file = File::open(samples_path)
                .with_context(|| format!("Failed to open samples file: {:?}", samples_path))?;
            let reader = BufReader::new(file);
            let base_dir = samples_path.parent().unwrap_or(Path::new("."));

            for line in reader.lines() {
                let line = line?;
                let id = line.trim();
                if id.is_empty() || id.starts_with('#') {
                    continue;
                }

                let (r1, r2) = find_fastq_pair(base_dir, id)?;
                samples.push(Sample {
                    name: id.to_string(),
                    r1,
                    r2,
                });
            }
        }
    } else if let (Some(ref r1_str), Some(ref r2_str)) = (&args.r1, &args.r2) {
        // Parse comma-separated file lists
        let r1_files: Vec<&str> = r1_str.split(',').collect();
        let r2_files: Vec<&str> = r2_str.split(',').collect();

        if r1_files.len() != r2_files.len() {
            anyhow::bail!("Number of R1 and R2 files must match");
        }

        for (r1, r2) in r1_files.iter().zip(r2_files.iter()) {
            let r1_path = PathBuf::from(r1.trim());
            let r2_path = PathBuf::from(r2.trim());

            // Extract sample name from filename
            // Handle .fastq.gz, .fq.gz, .fastq, .fq extensions
            let name = r1_path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| {
                    s.trim_end_matches(".fastq")
                     .trim_end_matches(".fq")
                     .trim_end_matches("_R1")
                     .trim_end_matches("_1")
                     .to_string()
                })
                .unwrap_or_else(|| format!("sample_{}", samples.len() + 1));

            samples.push(Sample {
                name,
                r1: r1_path,
                r2: r2_path,
            });
        }
    }

    Ok(samples)
}

fn process_sample(sample: &Sample, args: &Args) -> Result<Vec<ResultRow>> {
    let sample_dir = args.outdir.join(&sample.name);
    fs::create_dir_all(&sample_dir)?;

    // Validate inputs
    if !sample.r1.exists() {
        anyhow::bail!("R1 file not found: {:?}", sample.r1);
    }
    if !sample.r2.exists() {
        anyhow::bail!("R2 file not found: {:?}", sample.r2);
    }

    // Step 1: Align and filter reads
    if args.verbose {
        eprintln!("  [1/6] Aligning reads to ARG database (mapper: {})...", args.mapper);
    }
    let paf_path = match args.mapper.as_str() {
        "strobealign" => run_strobealign_reads(
            &sample.r1, &sample.r2,
            args.ref_fasta_resolved.as_ref().unwrap(),
            &sample_dir, &args.strobealign, args.paftools_path.as_deref(), args.threads)?,
        "bwa-mem2" => run_bwamem2_reads(
            &sample.r1, &sample.r2,
            args.ref_fasta_resolved.as_ref().unwrap(),
            &sample_dir, &args.bwa_mem2, args.paftools_path.as_deref(), args.threads)?,
        _ => run_minimap2_reads(&sample.r1, &sample.r2, args.arg_db.as_ref().unwrap(), &sample_dir, &args.minimap2, args.threads)?,
    };
    let matching_reads = parse_paf_filter(&paf_path, args.identity, args.min_align_len)?;

    if args.verbose {
        eprintln!("        Reads passing filter: {}", matching_reads.len());
    }

    if matching_reads.is_empty() {
        return Ok(Vec::new());
    }

    let (filtered_r1, filtered_r2) = extract_read_pairs(&sample.r1, &sample.r2, &matching_reads, &sample_dir)?;

    // Step 2: MEGAHIT assembly
    if args.verbose {
        eprintln!("  [2/6] Running MEGAHIT assembly...");
    }
    let megahit_dir = run_megahit(&filtered_r1, &filtered_r2, &sample_dir, &args.megahit, args.threads_per_sample)?;
    let contigs_file = megahit_dir.join("final.contigs.fa");

    if !contigs_file.exists() {
        return Ok(Vec::new());
    }

    let contigs = load_and_filter_contigs(&contigs_file, args.min_contig_len)?;
    if contigs.is_empty() {
        return Ok(Vec::new());
    }

    if args.verbose {
        eprintln!("        Contigs assembled: {}", contigs.len());
    }

    // Load the filtered reads ONCE and share them between the strict (Step 3) and
    // flexible (Step 6) extension passes, avoiding a second full FASTQ load. Only
    // when interior depth is NOT capped (cap-interior uses a different read subset
    // for the strict pass, so it loads its own).
    let shared_reads: Option<std::sync::Arc<Vec<String>>> = if args.cap_interior == 0 {
        Some(ContigExtender::load_reads_shared(&filtered_r1, &filtered_r2)?)
    } else {
        None
    };

    // Step 3: Strict extension (conservative, high confidence)
    if args.verbose {
        eprintln!("  [3/6] Extending contigs (strict)...");
    }
    let mut strict_contigs = extend_contigs_strict(&contigs, &filtered_r1, &filtered_r2, &sample_dir, args, shared_reads.clone())?;

    // Rename contigs for consistency (contig_1, contig_2, ...)
    // This ensures PAF query names match contig names in classify_genera
    for (i, c) in strict_contigs.iter_mut().enumerate() {
        c.name = format!("contig_{}", i + 1);
    }

    // Write contigs for ARG detection
    let contigs_path = sample_dir.join("contigs_strict.fasta");
    write_contigs_simple(&strict_contigs, &contigs_path)?;

    // Step 4: ARG detection
    if args.verbose {
        eprintln!("  [4/6] Detecting ARGs...");
    }
    let arg_hits = detect_args_blast(&contigs_path, args.arg_db.as_ref().unwrap(), &sample_dir,
                                     &args.blastn, &args.makeblastdb,
                                     args.arg_identity, args.arg_coverage, args.threads)?;
    let unique_args = deduplicate_args(arg_hits);

    if unique_args.is_empty() {
        return Ok(Vec::new());
    }

    if args.verbose {
        eprintln!("        ARGs detected: {}", unique_args.len());
    }

    // Step 5: Genus classification (1st attempt with strict extension)
    if args.verbose {
        eprintln!("  [5/6] Classifying genera (strict)...");
    }

    let genus_results = classify_genera(&unique_args, &strict_contigs, args)?;

    // Identify unresolved ARGs (genus unknown or low confidence)
    let min_flanking_for_resolve = 100; // Need at least 100bp flanking
    let unresolved_args: Vec<&ArgHit> = unique_args.iter()
        .filter(|hit| {
            genus_results.iter()
                .find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig)
                .map(|g| {
                    // Unresolved if: no genus OR insufficient flanking
                    g.genus.is_none() ||
                    (g.upstream_len < min_flanking_for_resolve && g.downstream_len < min_flanking_for_resolve)
                })
                .unwrap_or(true)
        })
        .collect();

    // Step 6: Flexible extension for unresolved ARGs only
    let mut flexible_results: HashMap<String, GenusResult> = HashMap::new();

    if !unresolved_args.is_empty() {
        if args.verbose {
            eprintln!("  [6/6] Extending {} unresolved contigs with Rust (flexible)...", unresolved_args.len());
        }

        // Get contigs that need flexible extension
        let unresolved_contig_names: FxHashSet<String> = unresolved_args.iter()
            .map(|h| h.contig.clone())
            .collect();

        let contigs_to_extend: Vec<FastaRecord> = strict_contigs.iter()
            .filter(|c| unresolved_contig_names.contains(&c.name))
            .cloned()
            .collect();

        if !contigs_to_extend.is_empty() {
            // Apply Rust extension
            let flexible_contigs = extend_contigs_flexible(&contigs_to_extend, &filtered_r1, &filtered_r2, &sample_dir, args, shared_reads.clone())?;

            // Re-classify with flexible contigs
            let flexible_genus = classify_genera(&unresolved_args.iter().map(|h| (*h).clone()).collect::<Vec<_>>(), &flexible_contigs, args)?;

            for result in flexible_genus {
                let key = format!("{}:{}", result.arg_name, result.contig_name);
                flexible_results.insert(key, result);
            }

            if args.verbose {
                eprintln!("        Flexible extension improved: {} ARGs", flexible_results.len());
            }
        }
    }

    // Step 6b: opt-in per-locus reassembly (v3 core/flank split) for STALLED loci.
    // Targets loci with short flanking on both sides AND not already flagged
    // plasmid (reassembly can't fix a mobile-element genus). Recovers a
    // classifiable flanking so the 4-axis machinery can label these NA loci.
    let mut reasm_results: HashMap<String, GenusResult> = HashMap::new();
    if args.reassemble {
        let contig_seq: HashMap<&str, &str> = strict_contigs.iter()
            .map(|c| (c.name.as_str(), c.seq.as_str()))
            .collect();
        let mut stalled: Vec<reassemble::StalledLocus> = Vec::new();
        for hit in &unique_args {
            let gr = genus_results.iter()
                .find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig);
            let short_flank = gr.map_or(true, |g| g.upstream_len < min_flanking_for_resolve
                && g.downstream_len < min_flanking_for_resolve);
            let is_plasmid = gr.map_or(false, |g| g.context == "plasmid");
            if !short_flank || is_plasmid {
                continue;
            }
            let cseq = match contig_seq.get(hit.contig.as_str()) {
                Some(s) => *s,
                None => continue,
            };
            let (s, e) = (hit.contig_start.min(cseq.len()), hit.contig_end.min(cseq.len()));
            if e <= s {
                continue;
            }
            stalled.push(reassemble::StalledLocus {
                key: format!("{}:{}", hit.arg_name, hit.contig),
                arg_name: hit.arg_name.clone(),
                arg_class: hit.arg_class.clone(),
                identity: hit.identity,
                coverage: hit.coverage,
                gene_seq: cseq[s..e].to_string(),
                seed_seq: cseq.to_string(),
            });
        }
        if !stalled.is_empty() {
            if args.verbose {
                eprintln!("  [6b] Reassembling {} stalled loci (core/flank split)...", stalled.len());
            }
            let spades_python = args.spades_python.clone().unwrap_or_else(|| {
                args.spades_path.parent().map(|p| p.join("python3")).unwrap_or_default()
            });
            let cfg = reassemble::ReasmConfig {
                minimap2: args.minimap2.clone(),
                spades_py: args.spades_path.clone(),
                spades_python,
                threads: args.threads_per_sample.max(1),
                core_cov: 0.70,
                jobs: args.reassemble_jobs.max(1),
            };
            let workdir = sample_dir.join("reassemble");
            match reassemble::reassemble_stalled(&stalled, &filtered_r1, &filtered_r2, &workdir, &cfg) {
                Ok(stitched_loci) => {
                    // Build ArgHits + FASTA for the stitched contigs, then re-classify.
                    let mut recs: Vec<FastaRecord> = Vec::new();
                    let mut hits: Vec<ArgHit> = Vec::new();
                    for sl in &stitched_loci {
                        for sc in &sl.contigs {
                            recs.push(FastaRecord { name: sc.name.clone(), seq: sc.seq.clone() });
                            hits.push(ArgHit {
                                members: vec![sl.arg_name.clone()],
                                arg_name: sl.arg_name.clone(),
                                arg_class: sl.arg_class.clone(),
                                contig: sc.name.clone(),
                                contig_len: sc.seq.len(),
                                identity: sl.identity,
                                coverage: sl.coverage,
                                contig_start: sc.arg_start,
                                contig_end: sc.arg_end,
                                strand: '+',
                            });
                        }
                    }
                    if !recs.is_empty() {
                        if let Ok(reclass) = classify_genera(&hits, &recs, args) {
                            // Group re-classified stitched contigs back to their
                            // original locus key; adopt the single best resolved
                            // result per locus (conservative: only recovers a genus,
                            // multi-genome nuance is noted but not merged).
                            let mut per_locus: HashMap<String, Vec<GenusResult>> = HashMap::new();
                            for r in reclass {
                                // stitched name = "<key sanitized>__cN" -> recover locus via prefix match
                                if let Some(sl) = stitched_loci.iter().find(|sl| {
                                    r.contig_name.starts_with(&sl.key.replace([':', '|', ' '], "_"))
                                }) {
                                    per_locus.entry(sl.key.clone()).or_default().push(r);
                                }
                            }
                            for (key, mut grs) in per_locus {
                                // best = has genus, then most flanking
                                grs.sort_by(|a, b| {
                                    let fa = a.upstream_len + a.downstream_len;
                                    let fb = b.upstream_len + b.downstream_len;
                                    b.genus.is_some().cmp(&a.genus.is_some())
                                        .then(fb.cmp(&fa))
                                });
                                if let Some(best) = grs.into_iter().next() {
                                    reasm_results.insert(key, best);
                                }
                            }
                        }
                    }
                    if args.verbose {
                        eprintln!("        Reassembly resolved: {} loci", reasm_results.values().filter(|g| g.genus.is_some()).count());
                    }
                }
                Err(e) => eprintln!("        (warning) reassembly failed: {}", e),
            }
        }
    }

    // Build result rows with extension_method
    let results: Vec<ResultRow> = unique_args.iter()
        .map(|hit| {
            let key = format!("{}:{}", hit.arg_name, hit.contig);

            // Precedence: reassembly (if it resolved a genus) > flexible > strict.
            let (genus_info, ext_method) = if let Some(r) = reasm_results.get(&key).filter(|r| r.genus.is_some()) {
                (r.clone(), "reassemble")
            } else if let Some(flex_result) = flexible_results.get(&key) {
                if flex_result.genus.is_some() {
                    (flex_result.clone(), "flexible")
                } else {
                    // Flexible didn't help, use strict result
                    let strict = genus_results.iter()
                        .find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig)
                        .cloned()
                        .unwrap_or_default();
                    (strict, "strict")
                }
            } else {
                // Use strict result
                let strict = genus_results.iter()
                    .find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig)
                    .cloned()
                    .unwrap_or_default();
                (strict, "strict")
            };

            let top_matches_str = genus_info.top_matches.iter()
                .map(|(g, s)| format!("{}:{:.1}", g, s))
                .collect::<Vec<_>>()
                .join(";");

            // Specificity: convert 0-1 to 0-100 if needed
            let specificity = if genus_info.specificity <= 1.0 {
                genus_info.specificity * 100.0
            } else {
                genus_info.specificity
            };

            ResultRow {
                sample: sample.name.clone(),
                contig_id: hit.contig.clone(),
                arg_name: hit.arg_name.clone(),
                arg_class: hit.arg_class.clone(),
                genus: format_genus_call(&genus_info),
                confidence: genus_info.confidence,
                specificity,
                identity: hit.identity,
                coverage: hit.coverage,
                contig_len: hit.contig_len,
                arg_start: hit.contig_start,
                arg_end: hit.contig_end,
                upstream_len: genus_info.upstream_len,
                downstream_len: genus_info.downstream_len,
                extension_method: ext_method.to_string(),
                top_matches: top_matches_str,
                snp_status: format!("{}", genus_info.snp_status),
                context: genus_info.context.clone(),
                species: format_species_call(&genus_info),
                credible_set: if genus_info.credible_set_grouped.is_empty() {
                    "NA".to_string()
                } else {
                    genus_info.credible_set_grouped.clone()
                },
                support: genus_info.support,
                resolution_distance: genus_info.resolution_distance,
                limited_by: genus_info.limited_by.clone(),
                resolution_rank: genus_info.resolution_rank.clone(),
                resolution_taxon: genus_info.resolution_taxon.clone(),
            }
        })
        .collect();

    // Per-locus outputs (opt-in via --emit-*): assembled gene/flank sequences per
    // class, for inspection / BLAST / flanking-DB growth.
    let emit_sel = EmitSel::resolve(args)?;
    if emit_sel.on {
        if let Err(e) = emit_locus_asm(&sample.name, &unique_args, &strict_contigs, &genus_results, &sample_dir, &emit_sel) {
            eprintln!("        (warning) locus asm emit failed: {}", e);
        }
        if emit_sel.states.iter().any(|s| s == "reads") {
            if let Err(e) = emit_locus_reads(&sample.name, &unique_args, &strict_contigs, &genus_results,
                                             &filtered_r1, &filtered_r2, &sample_dir, &emit_sel, args) {
                eprintln!("        (warning) locus reads emit failed: {}", e);
            }
        }
        if args.verbose { eprintln!("        Per-locus outputs written ({} classes)", emit_sel.classes.len()); }
    }

    Ok(results)
}

fn classify_genera(
    arg_hits: &[ArgHit],
    contigs: &[FastaRecord],
    args: &Args,
) -> Result<Vec<GenusResult>> {
    // Build contig map
    let contig_map: HashMap<String, String> = contigs.iter()
        .map(|c| (c.name.split_whitespace().next().unwrap_or(&c.name).to_string(), c.seq.clone()))
        .collect();

    // Build ArgPositions
    let positions: Vec<ArgPosition> = arg_hits.iter()
        .filter_map(|hit| {
            let contig_key = hit.contig.split_whitespace().next().unwrap_or(&hit.contig);
            let contig_seq = contig_map.get(contig_key)?;

            Some(ArgPosition {
                arg_name: hit.arg_name.clone(),
                contig_name: hit.contig.clone(),
                contig_seq: contig_seq.clone(),
                contig_len: hit.contig_len,
                arg_start: hit.contig_start,
                arg_end: hit.contig_end,
                strand: hit.strand,
                members: if hit.members.is_empty() { vec![hit.arg_name.clone()] } else { hit.members.clone() },
            })
        })
        .collect();

    if positions.is_empty() {
        return Ok(Vec::new());
    }

    // Check if flanking database exists
    if !args.flanking_db.as_ref().unwrap().exists() {
        // Return placeholder results if no flanking database
        if args.verbose {
            eprintln!("        Flanking database not found, skipping genus classification");
        }
        let results: Vec<GenusResult> = positions.iter()
            .map(|pos| {
                let upstream_len = pos.arg_start.min(args.max_flanking);
                let downstream_len = (pos.contig_len - pos.arg_end).min(args.max_flanking);

                // Verify SNP for point mutation genes even without flanking DB
                let snp_status = snp::verify_snp(
                    &pos.contig_seq,
                    &pos.arg_name,
                    0,
                    pos.arg_end - pos.arg_start,
                    pos.arg_start,
                    pos.arg_end,
                    pos.strand,
                );

                GenusResult {
                    arg_name: pos.arg_name.clone(),
                    contig_name: pos.contig_name.clone(),
                    genus: None,
                    confidence: 0.0,
                    specificity: 0.0,
                    upstream_len,
                    downstream_len,
                    top_matches: vec![("no_flanking_db".to_string(), 0.0)],
                    snp_status,
                    upstream_seq: String::new(),
                    downstream_seq: String::new(),
                    context: "NA".to_string(),
                    n_genera_tied: 0,
                    species: None,
                    species_top_matches: vec![],
                    n_species_tied: 0,
                    credible_set: vec![],
                    support: 0.0,
                    resolution_distance: 0.0,
                    limited_by: "none".to_string(),
                    resolution_rank: "NA".to_string(),
                    resolution_taxon: "NA".to_string(),
                    credible_set_grouped: String::new(),
                }
            })
            .collect();
        return Ok(results);
    }

    // Use GenusClassifier for minimap2-based classification
    // Based on divergence analysis: intra-genus ~96% identity, inter-genus ~87%
    // Use 90% threshold to distinguish genera
    let mut classifier = GenusClassifier::new(
        args.flanking_db.as_ref().unwrap(),
        &args.minimap2,
        args.genus_identity,  // min_identity to distinguish intra vs inter-genus
        100,                  // min_align_len: require decent overlap
        args.max_flanking,
        args.plasmid_contigs.as_deref(),
        args.species_identity,
        args.species_map.as_deref(),
        args.context_plasmid_frac,
        args.context_chromosome_frac,
        args.max_ref_flanks,
        args.target_coverage,
    )?;

    classifier.classify_batch(&positions, args.threads)
}

fn output_results(results: &[ResultRow], args: &Args) -> Result<()> {
    let header = "Sample\tContig_ID\tARG_Name\tARG_Class\tGenus\tSpecies\tConfidence\tSpecificity\tContext\tARG_Identity\tARG_Coverage\tContig_Len\tARG_Start\tARG_End\tUpstream_Len\tDownstream_Len\tExtension_Method\tTop_Matches\tCredible_Set\tResolution_Rank\tResolution_Taxon\tSupport\tResolution_Distance\tLimited_By";

    // By default, filter out WildType and NotCovered (not true resistance genes)
    // WildType: SNP position checked but found wild-type allele (no resistance mutation)
    // NotCovered: Flanking region not covered, cannot verify SNP status
    // Use --all-hits to include these in output
    let output_results: Vec<_> = if args.all_hits {
        results.iter().collect()
    } else {
        results.iter()
            .filter(|r| r.snp_status != "WildType" && r.snp_status != "NotCovered")
            .collect()
    };

    let excluded_count = results.len() - output_results.len();

    // Output to directory/results.tsv
    let output_path = args.outdir.join("results.tsv");
    let mut output = BufWriter::new(File::create(&output_path)?);

    writeln!(output, "{}", header)?;

    for r in &output_results {
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}\t{:.1}\t{:.1}\t{}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{}",
            r.sample,
            r.contig_id,
            r.arg_name,
            r.arg_class,
            r.genus,
            r.species,
            r.confidence,
            r.specificity,
            r.context,
            r.identity,
            r.coverage,
            r.contig_len,
            r.arg_start,
            r.arg_end,
            r.upstream_len,
            r.downstream_len,
            r.extension_method,
            r.top_matches,
            r.credible_set,
            r.resolution_rank,
            r.resolution_taxon,
            r.support,
            r.resolution_distance,
            r.limited_by
        )?;
    }

    if args.verbose {
        if args.all_hits {
            eprintln!("Results: {} ARGs written (all hits included)", output_results.len());
        } else {
            eprintln!("Results: {} ARGs written ({} WildType/NotCovered excluded)",
                      output_results.len(), excluded_count);
        }
        eprintln!("Results written to: {}", output_path.display());
    }

    Ok(())
}

/// Strict extension (conservative, high confidence) - uses exact k-mer matching
/// Positionally cap read depth in contig interiors before extension.
///
/// The k-mer extender only ever uses reads that match a contig's EDGE k-mers, so
/// reads mapping deep in the interior (>= `end_zone` bp from both ends) are dead
/// weight — loaded into RAM and scanned every iteration for nothing. This maps the
/// reads to the contigs and drops interior reads once local coverage exceeds `cap`,
/// while keeping every terminal-zone and unmapped read (extension/flanking fuel).
/// Deterministic: alignments are processed in a fixed sort order.
fn cap_interior_reads(
    contigs: &[FastaRecord],
    r1: &Path,
    r2: &Path,
    sample_dir: &Path,
    cap: usize,
    end_zone: usize,
    minimap2: &str,
    threads: usize,
) -> Result<(PathBuf, PathBuf)> {
    let contigs_fa = sample_dir.join("cap_contigs.fa");
    write_contigs_simple(contigs, &contigs_fa)?;

    // Build the set of interior reads to DROP for one read file.
    let build_drop_set = |reads: &Path, tag: &str| -> Result<FxHashSet<String>> {
        let paf = sample_dir.join(format!("cap_{}.paf", tag));
        let status = Command::new(minimap2)
            .args(["-x", "sr", "-t", &threads.to_string()])
            .arg(&contigs_fa)
            .arg(reads)
            .arg("-o")
            .arg(&paf)
            .stderr(std::process::Stdio::null())
            .status()
            .context("Failed to run minimap2 for interior cap")?;
        if !status.success() {
            anyhow::bail!("minimap2 (interior cap) exited with {:?}", status.code());
        }

        // Best alignment per read (by matches), plus classify interior vs terminal.
        // best: read -> (matches, target, target_len, tstart, tend)
        let mut best: HashMap<String, (usize, String, usize, usize, usize)> = HashMap::new();
        let reader = PafReader::open(&paf)?;
        for rec in reader {
            let r = rec?;
            let e = best.get(&r.query_name);
            if e.map_or(true, |(m, ..)| r.matches > *m) {
                best.insert(
                    r.query_name.clone(),
                    (r.matches, r.target_name, r.target_len, r.target_start, r.target_end),
                );
            }
        }

        // Interior reads only, sorted deterministically by (contig, start, read).
        let mut interior: Vec<(&str, usize, usize, &str)> = Vec::new(); // (contig, s, e, read)
        for (read, (_m, contig, tlen, ts, te)) in best.iter() {
            let is_terminal = *ts < end_zone || *te + end_zone > *tlen;
            if !is_terminal {
                interior.push((contig.as_str(), *ts, *te, read.as_str()));
            }
        }
        interior.sort_unstable();

        // Per-contig coverage cap: keep a read only if some base it covers is still
        // below `cap`; otherwise drop it.
        let mut cov: HashMap<&str, Vec<u16>> = HashMap::new();
        let mut drop_set = FxHashSet::default();
        let capv = cap as u16;
        for (contig, s, e, read) in interior {
            let tlen = best[read].2;
            let arr = cov.entry(contig).or_insert_with(|| vec![0u16; tlen]);
            let end = e.min(arr.len());
            let mut under = false;
            for c in &arr[s..end] {
                if *c < capv { under = true; break; }
            }
            if under {
                for c in &mut arr[s..end] { *c = c.saturating_add(1); }
            } else {
                drop_set.insert(read.to_string());
            }
        }
        Ok(drop_set)
    };

    let write_kept = |reads: &Path, drop_set: &FxHashSet<String>, out: &Path| -> Result<(usize, usize)> {
        let mut reader = FastqFile::open(reads)?;
        let mut w = BufWriter::new(File::create(out)?);
        let (mut kept, mut total) = (0usize, 0usize);
        while let Some(rec) = reader.read_next()? {
            total += 1;
            if !drop_set.contains(&rec.name) {
                writeln!(w, "@{}\n{}\n+\n{}", rec.name, rec.seq, rec.qual)?;
                kept += 1;
            }
        }
        Ok((kept, total))
    };

    let drop1 = build_drop_set(r1, "r1")?;
    let drop2 = build_drop_set(r2, "r2")?;
    let out_r1 = sample_dir.join("capped_R1.fq");
    let out_r2 = sample_dir.join("capped_R2.fq");
    let (k1, t1) = write_kept(r1, &drop1, &out_r1)?;
    let (k2, t2) = write_kept(r2, &drop2, &out_r2)?;
    eprintln!(
        "        Interior cap ({}x, end-zone {}bp): kept {}/{} reads ({:.0}%)",
        cap, end_zone, k1 + k2, t1 + t2,
        (k1 + k2) as f64 / (t1 + t2).max(1) as f64 * 100.0
    );
    Ok((out_r1, out_r2))
}

fn extend_contigs_strict(
    contigs: &[FastaRecord],
    r1: &Path,
    r2: &Path,
    sample_dir: &Path,
    args: &Args,
    shared_reads: Option<std::sync::Arc<Vec<String>>>,
) -> Result<Vec<FastaRecord>> {
    // Strict mode: higher coverage requirement, lower branching tolerance
    let config = ExtenderConfig {
        kmer_size: args.ext_kmer_size,
        extension_step: args.ext_length,
        min_coverage: 3,
        branching_threshold: 0.1,
        max_n_ratio: 0.02,
        // Only ~max_flanking bp of flanking is ever used downstream; cap each side
        // (with margin) so runaway contigs don't burn many extra re-scan rounds.
        max_extension_per_side: if args.max_extension > 0 { args.max_extension } else { (args.max_flanking * 2).max(1500) },
        ..Default::default()
    };

    // Optionally cap interior read depth to shrink the extender's memory use.
    let (cap_r1, cap_r2);
    let (r1, r2) = if args.cap_interior > 0 {
        let (a, b) = cap_interior_reads(
            contigs, r1, r2, sample_dir,
            args.cap_interior, args.end_zone, &args.minimap2, args.threads,
        )?;
        cap_r1 = a; cap_r2 = b;
        (cap_r1.as_path(), cap_r2.as_path())
    } else {
        (r1, r2)
    };

    let mut extender = ContigExtender::new(config);
    // Reuse the pre-loaded reads when interior wasn't capped (same read set);
    // otherwise load the (capped) reads for this pass.
    match (&shared_reads, args.cap_interior) {
        (Some(reads), 0) => extender.set_reads(reads.clone()),
        _ => extender.load_reads(r1, r2)?,
    }

    let results = extender.extend_all_hybrid(contigs)?;

    let extended_path = sample_dir.join("contigs_strict.fasta");
    write_extended_contigs(&results, &extended_path)?;

    Ok(results.into_iter()
        .map(|r| FastaRecord { name: r.name, seq: r.extended_seq })
        .collect())
}

/// Flexible extension (aggressive) - for unresolved cases
fn extend_contigs_flexible(
    contigs: &[FastaRecord],
    r1: &Path,
    r2: &Path,
    sample_dir: &Path,
    args: &Args,
    shared_reads: Option<std::sync::Arc<Vec<String>>>,
) -> Result<Vec<FastaRecord>> {
    // Flexible mode: lower coverage, higher branching tolerance
    let config = ExtenderConfig {
        kmer_size: args.ext_kmer_size,
        extension_step: args.ext_length,
        min_coverage: 2,
        branching_threshold: 0.2,
        max_n_ratio: 0.05,
        max_extension_per_side: if args.max_extension > 0 { args.max_extension } else { (args.max_flanking * 2).max(1500) },
        ..Default::default()
    };

    let mut extender = ContigExtender::new(config);
    // Reuse the pre-loaded reads (flexible always uses the full filtered set).
    match &shared_reads {
        Some(reads) => extender.set_reads(reads.clone()),
        None => extender.load_reads(r1, r2)?,
    }

    let results = extender.extend_all_hybrid(contigs)?;

    let extended_path = sample_dir.join("contigs_flexible.fasta");
    write_extended_contigs(&results, &extended_path)?;

    Ok(results.into_iter()
        .map(|r| FastaRecord { name: r.name, seq: r.extended_seq })
        .collect())
}

fn run_minimap2_reads(r1: &Path, r2: &Path, db: &Path, output_dir: &Path, minimap2: &str, threads: usize) -> Result<PathBuf> {
    let paf_r1 = output_dir.join("alignment_r1.paf");
    let paf_r2 = output_dir.join("alignment_r2.paf");
    let paf_merged = output_dir.join("alignment.paf");

    // Run R1 and R2 alignment in parallel (each uses threads/2)
    let threads_per_job = (threads / 2).max(1);

    let r1_owned = r1.to_path_buf();
    let r2_owned = r2.to_path_buf();
    let db_owned = db.to_path_buf();
    let paf_r1_owned = paf_r1.clone();
    let paf_r2_owned = paf_r2.clone();
    let minimap2_owned = minimap2.to_string();
    let threads_str = threads_per_job.to_string();

    let handle_r1 = std::thread::spawn({
        let db = db_owned.clone();
        let minimap2 = minimap2_owned.clone();
        let threads_str = threads_str.clone();
        move || {
            Command::new(&minimap2)
                .args(["-x", "sr", "-t", &threads_str, "-c"])
                .arg(&db).arg(&r1_owned).arg("-o").arg(&paf_r1_owned)
                .stderr(std::process::Stdio::null())
                .status()
        }
    });

    let handle_r2 = std::thread::spawn({
        let db = db_owned;
        let minimap2 = minimap2_owned;
        move || {
            let threads_str = threads_per_job.to_string();
            Command::new(&minimap2)
                .args(["-x", "sr", "-t", &threads_str, "-c"])
                .arg(&db).arg(&r2_owned).arg("-o").arg(&paf_r2_owned)
                .stderr(std::process::Stdio::null())
                .status()
        }
    });

    handle_r1.join().map_err(|_| anyhow::anyhow!("R1 alignment thread panicked"))?.context("Failed to run minimap2 on R1")?;
    handle_r2.join().map_err(|_| anyhow::anyhow!("R2 alignment thread panicked"))?.context("Failed to run minimap2 on R2")?;

    let mut merged = File::create(&paf_merged)?;
    if paf_r1.exists() {
        merged.write_all(&fs::read(&paf_r1)?)?;
    }
    if paf_r2.exists() {
        merged.write_all(&fs::read(&paf_r2)?)?;
    }

    Ok(paf_merged)
}

/// Derive a FASTA reference for strobealign/bwa-mem2 from --ref-fasta, or by
/// swapping a .mmi extension on --arg-db for a sibling .fa/.fas/.fasta.
/// First regular file in `dir` whose extension matches one of `exts` (sorted for
/// deterministic pick). Used by --db-dir discovery.
fn find_db_file(dir: &Path, exts: &[&str]) -> Option<PathBuf> {
    let mut hits: Vec<PathBuf> = fs::read_dir(dir).ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .filter(|p| p.extension()
            .and_then(|e| e.to_str())
            .map(|e| exts.iter().any(|x| x.eq_ignore_ascii_case(e)))
            .unwrap_or(false))
        .collect();
    hits.sort();
    hits.into_iter().next()
}

/// Locate the flanking DB in `dir`: a `flanking.fdb` file, any `*.fdb` file, or a
/// `*.fdb/` directory containing `flanking.fdb`.
/// Locate the flanking DB in `dir`.
///
/// Returns `Ok(Some(path))` for a single unambiguous match, `Ok(None)` when nothing
/// is found, or `Err` when several `*.fdb` files are present — the user must then pick
/// one with `-f/--flanking-db` rather than have ARGenus silently guess (e.g. a folder
/// holding both `flanking_1kbp.fdb` and `flanking_5kbp.fdb`).
fn find_flanking_db(dir: &Path) -> Result<Option<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .filter(|p| p.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("fdb"))
            .unwrap_or(false))
        .collect();
    files.sort();
    if files.len() == 1 {
        return Ok(Some(files.pop().unwrap()));
    }
    if files.len() > 1 {
        let names: Vec<String> = files.iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()).map(String::from))
            .collect();
        anyhow::bail!(
            "found {} flanking databases in {} ({}); \
             pass the one you want explicitly with -f/--flanking-db",
            files.len(), dir.display(), names.join(", ")
        );
    }
    // No `*.fdb` file — try the builder's `*.fdb/` directory layout.
    let mut subdirs: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && p.extension().and_then(|e| e.to_str()) == Some("fdb"))
        .collect();
    subdirs.sort();
    for sd in subdirs {
        let inner = sd.join("flanking.fdb");
        if inner.is_file() {
            return Ok(Some(inner));
        }
    }
    Ok(None)
}

/// True if `p` names a FASTA (optionally gzip-compressed): *.fa/.fas/.fasta/.fna,
/// optionally with a trailing .gz. minimap2 and strobealign both read gzipped FASTA.
fn is_fasta_path(p: &Path) -> bool {
    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
    let base = name.strip_suffix(".gz").unwrap_or(&name);
    [".fa", ".fas", ".fasta", ".fna"].iter().any(|e| base.ends_with(e))
}

/// First FASTA (plain or .gz) in `dir`, sorted for a deterministic pick.
fn find_fasta_file(dir: &Path) -> Option<PathBuf> {
    let mut hits: Vec<PathBuf> = fs::read_dir(dir).ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && is_fasta_path(p))
        .collect();
    hits.sort();
    hits.into_iter().next()
}

fn resolve_ref_fasta(args: &Args) -> Result<PathBuf> {
    if let Some(p) = &args.ref_fasta {
        if p.exists() {
            return Ok(p.clone());
        }
        anyhow::bail!("--ref-fasta does not exist: {:?}", p);
    }
    if let Some(db) = &args.arg_db {
        // (a) --arg-db is itself a FASTA (plain or .gz): use it directly.
        if is_fasta_path(db) && db.exists() {
            return Ok(db.clone());
        }
        // (b) derive from a .mmi (etc.) stem: AMR_PanRes.mmi -> AMR_PanRes.fa[.gz]
        let stem = db.with_extension("");
        for ext in ["fa", "fas", "fasta", "fna", "fa.gz", "fas.gz", "fasta.gz", "fna.gz"] {
            let cand = PathBuf::from(format!("{}.{}", stem.display(), ext));
            if cand.exists() {
                return Ok(cand);
            }
        }
    }
    anyhow::bail!(
        "--mapper {} needs a FASTA reference. Provide --ref-fasta <FILE> \
         (strobealign/bwa-mem2 cannot read a minimap2 .mmi).",
        args.mapper
    );
}

/// Ensure bwa-mem2 index sidecar files exist next to `fasta`; build once if not.
fn ensure_bwamem2_index(bwa_mem2: &str, fasta: &Path) -> Result<()> {
    let marker = PathBuf::from(format!("{}.bwt.2bit.64", fasta.display()));
    if marker.exists() {
        return Ok(());
    }
    let output = Command::new(bwa_mem2)
        .arg("index")
        .arg(fasta)
        .output()
        .context("Failed to run bwa-mem2 index")?;
    if !output.status.success() {
        anyhow::bail!(
            "bwa-mem2 index failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// Read filter using strobealign (paired). Emits SAM, converts to PAF, returns PAF path.
/// The PAF is semantically identical to minimap2's (matches/block_len), so the
/// existing parse_paf_filter contract holds unchanged.
fn run_strobealign_reads(
    r1: &Path, r2: &Path, ref_fasta: &Path, output_dir: &Path,
    strobealign: &str, paftools: Option<&Path>, threads: usize,
) -> Result<PathBuf> {
    let sam_path = output_dir.join("alignment.sam");
    let paf_path = output_dir.join("alignment.paf");

    let sam_file = File::create(&sam_path)?;
    let status = Command::new(strobealign)
        .args(["-t", &threads.to_string()])
        .arg(ref_fasta).arg(r1).arg(r2)
        .stdout(std::process::Stdio::from(sam_file))
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to run strobealign")?;
    if !status.success() {
        anyhow::bail!("strobealign exited with status {:?}", status.code());
    }

    convert_sam_to_paf(&sam_path, &paf_path, paftools)?;
    Ok(paf_path)
}

/// Read filter using bwa-mem2 (paired). Emits SAM, converts to PAF, returns PAF path.
fn run_bwamem2_reads(
    r1: &Path, r2: &Path, ref_fasta: &Path, output_dir: &Path,
    bwa_mem2: &str, paftools: Option<&Path>, threads: usize,
) -> Result<PathBuf> {
    let sam_path = output_dir.join("alignment.sam");
    let paf_path = output_dir.join("alignment.paf");

    ensure_bwamem2_index(bwa_mem2, ref_fasta)?;

    let sam_file = File::create(&sam_path)?;
    let status = Command::new(bwa_mem2)
        .arg("mem")
        .args(["-t", &threads.to_string()])
        .arg(ref_fasta).arg(r1).arg(r2)
        .stdout(std::process::Stdio::from(sam_file))
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to run bwa-mem2 mem")?;
    if !status.success() {
        anyhow::bail!("bwa-mem2 mem exited with status {:?}", status.code());
    }

    convert_sam_to_paf(&sam_path, &paf_path, paftools)?;
    Ok(paf_path)
}

/// Convert SAM to PAF. Uses paftools.sh sam2paf when a path is given, otherwise a
/// built-in converter with identical identity/block-length semantics.
fn convert_sam_to_paf(sam: &Path, paf: &Path, paftools: Option<&Path>) -> Result<()> {
    if let Some(pt) = paftools {
        let out = File::create(paf)?;
        let status = Command::new(pt)
            .arg("sam2paf")
            .arg(sam)
            .stdout(std::process::Stdio::from(out))
            .stderr(std::process::Stdio::null())
            .status()
            .context("Failed to run paftools.sh sam2paf")?;
        if status.success() {
            return Ok(());
        }
        eprintln!("paftools.sh sam2paf failed; falling back to built-in converter");
    }
    sam_to_paf_builtin(sam, paf)
}

/// Built-in SAM->PAF converter.
///
/// Mirrors `paftools.sh sam2paf` for the fields the read filter consumes:
///   block_len (col 11) = sum of CIGAR M/I/D/=/X   (aligned columns + indels)
///   matches   (col 10) = aligned columns (M/=/X) - mismatches,
///                        where mismatches = NM - (inserted + deleted bases)
/// so parse_paf_filter's identity = matches/block_len matches minimap2 exactly.
/// Records without an alignment (FLAG 0x4) or without an NM tag are skipped.
fn sam_to_paf_builtin(sam: &Path, paf: &Path) -> Result<()> {
    let reader = BufReader::with_capacity(1 << 20, File::open(sam)?);
    let mut writer = BufWriter::new(File::create(paf)?);
    // Reference lengths from @SQ headers (SN -> LN).
    let mut ref_len: HashMap<String, usize> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        if line.starts_with('@') {
            if let Some(rest) = line.strip_prefix("@SQ\t") {
                let mut sn = None;
                let mut ln = None;
                for f in rest.split('\t') {
                    if let Some(v) = f.strip_prefix("SN:") { sn = Some(v.to_string()); }
                    else if let Some(v) = f.strip_prefix("LN:") { ln = v.parse::<usize>().ok(); }
                }
                if let (Some(sn), Some(ln)) = (sn, ln) {
                    ref_len.insert(sn, ln);
                }
            }
            continue;
        }

        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 11 {
            continue;
        }
        let flag: u32 = f[1].parse().unwrap_or(0);
        if flag & 0x4 != 0 {
            continue; // unmapped
        }
        let cigar = f[5];
        if cigar == "*" {
            continue;
        }
        // NM tag (edit distance) from optional fields.
        let mut nm: Option<i64> = None;
        for tag in &f[11..] {
            if let Some(v) = tag.strip_prefix("NM:i:") {
                nm = v.parse::<i64>().ok();
                break;
            }
        }
        let nm = match nm {
            Some(v) => v,
            None => continue, // cannot compute identity without NM
        };

        // Walk the CIGAR.
        let (mut m_ops, mut ins, mut del): (i64, i64, i64) = (0, 0, 0);
        let (mut q_lead_clip, mut q_trail_clip): (usize, usize) = (0, 0);
        let mut ref_consumed: usize = 0;
        let mut num = 0usize;
        let mut seen_aln = false;
        let bytes = cigar.as_bytes();
        for &b in bytes {
            if b.is_ascii_digit() {
                num = num * 10 + (b - b'0') as usize;
            } else {
                match b {
                    b'M' | b'=' | b'X' => { m_ops += num as i64; ref_consumed += num; seen_aln = true; }
                    b'I' => { ins += num as i64; seen_aln = true; }
                    b'D' | b'N' => { del += num as i64; ref_consumed += num; seen_aln = true; }
                    b'S' | b'H' => {
                        if seen_aln { q_trail_clip = num; } else { q_lead_clip = num; }
                    }
                    _ => {}
                }
                num = 0;
            }
        }

        let block_len = m_ops + ins + del;
        if block_len <= 0 {
            continue;
        }
        let mismatches = nm - (ins + del);
        let matches = (m_ops - mismatches).max(0);

        let qname = f[0];
        let strand = if flag & 0x10 != 0 { '-' } else { '+' };
        let rname = f[2];
        let pos: usize = f[3].parse::<usize>().unwrap_or(1);
        let mapq: &str = f[4];
        let t_start = pos.saturating_sub(1);
        let t_end = t_start + ref_consumed;
        let t_len = ref_len.get(rname).copied().unwrap_or(t_end.max(1));

        // Query coordinates on the forward read (soft/hard clips define the span).
        let q_aln_len = (m_ops + ins) as usize;
        let q_len = q_lead_clip + q_aln_len + q_trail_clip;
        // PAF query coords are on the original (forward) read.
        let (q_start, q_end) = if strand == '+' {
            (q_lead_clip, q_lead_clip + q_aln_len)
        } else {
            (q_trail_clip, q_trail_clip + q_aln_len)
        };

        writeln!(
            writer,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            qname, q_len.max(1), q_start, q_end, strand,
            rname, t_len, t_start, t_end, matches, block_len, mapq
        )?;
    }
    writer.flush()?;
    Ok(())
}

fn parse_paf_filter(paf_path: &Path, min_identity: f64, min_align_len: usize) -> Result<FxHashSet<String>> {
    let mut matching = FxHashSet::default();
    let min_identity_pct = min_identity * 100.0;

    let reader = PafReader::open(paf_path)?;
    for record in reader {
        let rec = record?;
        let identity = rec.calculate_identity();
        if identity >= min_identity_pct && rec.block_len >= min_align_len {
            matching.insert(rec.query_name);
        }
    }

    Ok(matching)
}

fn extract_read_pairs(r1: &Path, r2: &Path, matching: &FxHashSet<String>, output_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let filtered_r1 = output_dir.join("filtered_R1.fq");
    let filtered_r2 = output_dir.join("filtered_R2.fq");

    let normalized: FxHashSet<String> = matching.iter()
        .map(|name| {
            let name = name.split_whitespace().next().unwrap_or(name);
            if name.ends_with("/1") || name.ends_with("/2") {
                name[..name.len() - 2].to_string()
            } else {
                name.to_string()
            }
        })
        .collect();

    let mut r1_reader = FastqFile::open(r1)?;
    let mut r2_reader = FastqFile::open(r2)?;
    let mut out1 = BufWriter::new(File::create(&filtered_r1)?);
    let mut out2 = BufWriter::new(File::create(&filtered_r2)?);

    loop {
        let rec1 = r1_reader.read_next()?;
        let rec2 = r2_reader.read_next()?;

        match (rec1, rec2) {
            (Some(r1), Some(r2)) => {
                let name1 = normalize_read_name(&r1.name);
                let name2 = normalize_read_name(&r2.name);

                if normalized.contains(&name1) || normalized.contains(&name2) ||
                   matching.contains(&r1.name) || matching.contains(&r2.name) {
                    writeln!(out1, "@{}\n{}\n+\n{}", r1.name, r1.seq, r1.qual)?;
                    writeln!(out2, "@{}\n{}\n+\n{}", r2.name, r2.seq, r2.qual)?;
                }
            }
            _ => break,
        }
    }

    Ok((filtered_r1, filtered_r2))
}

fn normalize_read_name(name: &str) -> String {
    let name = name.split_whitespace().next().unwrap_or(name);
    if name.ends_with("/1") || name.ends_with("/2") {
        name[..name.len() - 2].to_string()
    } else {
        name.to_string()
    }
}

fn run_megahit(r1: &Path, r2: &Path, output_dir: &Path, megahit: &str, threads: usize) -> Result<PathBuf> {
    let megahit_dir = output_dir.join("megahit");

    // MEGAHIT requires output dir to not exist
    if megahit_dir.exists() {
        fs::remove_dir_all(&megahit_dir)?;
    }

    let output = Command::new(megahit)
        .arg("-1").arg(r1)
        .arg("-2").arg(r2)
        .arg("-o").arg(&megahit_dir)
        .arg("-t").arg(threads.to_string())
        .arg("--min-contig-len").arg("200")
        .output()
        .context("Failed to run MEGAHIT")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("MEGAHIT failed (exit code: {:?})", output.status.code());
        eprintln!("stderr: {}", stderr);
        eprintln!("stdout: {}", stdout);
    }

    Ok(megahit_dir)
}

fn load_and_filter_contigs(path: &Path, min_len: usize) -> Result<Vec<FastaRecord>> {
    let reader = FastaReader::open(path)?;
    Ok(reader.filter_map(|r| r.ok()).filter(|r| r.seq.len() >= min_len).collect())
}

/// Write contigs to FASTA file
fn write_contigs_simple(contigs: &[FastaRecord], path: &Path) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    for contig in contigs {
        writeln!(writer, ">{}", contig.name)?;
        writeln!(writer, "{}", contig.seq)?;
    }
    Ok(())
}

/// Reverse-complement a DNA string (non-ACGT mapped to N).
fn revcomp_str(seq: &str) -> String {
    seq.chars().rev().map(|c| match c.to_ascii_uppercase() {
        'A' => 'T', 'T' => 'A', 'G' => 'C', 'C' => 'G', _ => 'N',
    }).collect()
}

/// Selection of per-locus outputs to emit (cross-product of class × part × state).
struct EmitSel {
    on: bool,
    classes: Vec<String>,
    parts: Vec<String>,
    states: Vec<String>,
}

impl EmitSel {
    fn resolve(args: &Args) -> Result<Self> {
        let on = !args.emit_class.is_empty() || !args.emit_part.is_empty() || !args.emit_state.is_empty();
        let norm = |v: &[String], allowed: &[&str], name: &str| -> Result<Vec<String>> {
            if v.is_empty() { return Ok(allowed.iter().map(|s| s.to_string()).collect()); }
            for x in v {
                if !allowed.contains(&x.as_str()) {
                    anyhow::bail!("invalid --emit-{} value '{}' (allowed: {})", name, x, allowed.join(", "));
                }
            }
            Ok(v.to_vec())
        };
        Ok(EmitSel {
            on,
            classes: norm(&args.emit_class, &["resolved", "flanknomatch", "genenotindb"], "class")?,
            parts: norm(&args.emit_part, &["gene", "flank"], "part")?,
            states: norm(&args.emit_state, &["asm", "reads"], "state")?,
        })
    }
    fn wants(&self, class: &str, part: &str, state: &str) -> bool {
        self.on
            && self.classes.iter().any(|c| c == class)
            && self.parts.iter().any(|p| p == part)
            && self.states.iter().any(|s| s == state)
    }
}

/// Flanking-DB outcome class for a locus.
fn locus_class(res: &GenusResult) -> &'static str {
    if res.genus.is_some() {
        "resolved"
    } else if res.top_matches.first().map(|(g, _)| g == "gene_not_in_db").unwrap_or(false) {
        "genenotindb"
    } else {
        "flanknomatch"
    }
}

/// Placeholder for an absent header field (chosen so R/pandas read it as missing).
const NA: &str = "NA";

/// `flankdb` header field: DB match info when present, else why not.
fn flankdb_field(res: &GenusResult) -> String {
    match locus_class(res) {
        "resolved" => res.top_matches.first().map(|(g, s)| format!("{}:{:.1}", g, s)).unwrap_or_else(|| NA.to_string()),
        "genenotindb" => "gene_absent".to_string(),
        _ => "no_match".to_string(),
    }
}

/// Uniform 12-column, `|`-delimited FASTA header (TSV-friendly; "." = NA).
/// Columns: sample|contig|part|arg|argclass|pct_id|pct_cov|region|flank_len|genus|flankdb|read
#[allow(clippy::too_many_arguments)]
fn locus_hdr(sample: &str, contig: &str, part: &str, arg: &str, argclass: &str,
             pct_id: &str, pct_cov: &str, region: &str, flank_len: &str,
             genus: &str, flankdb: &str, read: &str) -> String {
    format!(">{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            sample, contig, part, arg, argclass, pct_id, pct_cov, region, flank_len, genus, flankdb, read)
}

const LOCUS_COLUMNS: &str = "sample\tcontig\tpart\targ\targclass\tpct_id\tpct_cov\tregion\tflank_len\tgenus\tflankdb\tread";

/// Emit per-locus FASTA outputs (assembled gene/flank sequences) for the selected
/// class × part × state combinations. Uses the given classification results and the
/// contigs they were called on so gene and flanking share one coordinate frame.
/// DB-redundant duplicates on the same contig region are collapsed. Read-state
/// outputs are produced separately by `emit_locus_reads`.
fn emit_locus_asm(
    sample_name: &str,
    arg_hits: &[ArgHit],
    contigs: &[FastaRecord],
    results: &[GenusResult],
    sample_dir: &Path,
    sel: &EmitSel,
) -> Result<()> {
    const MIN_FLANK: usize = 50;
    let contig_map: HashMap<&str, &str> = contigs.iter()
        .map(|c| (c.name.split_whitespace().next().unwrap_or(&c.name), c.seq.as_str()))
        .collect();

    // Open only the selected {class}.{part}.asm files.
    let mut files: HashMap<(String, String), BufWriter<File>> = HashMap::new();
    for class in &sel.classes {
        for part in &sel.parts {
            if sel.wants(class, part, "asm") {
                let path = sample_dir.join(format!("{}.{}.asm.fasta", class, part));
                files.insert((class.clone(), part.clone()), BufWriter::new(File::create(path)?));
            }
        }
    }
    // Column-schema sidecar.
    if sel.on {
        let mut c = BufWriter::new(File::create(sample_dir.join("loci_outputs.columns.tsv"))?);
        writeln!(c, "{}", LOCUS_COLUMNS)?;
    }
    if files.is_empty() { return Ok(()); }

    let mut seen: std::collections::HashSet<(String, usize, usize)> = std::collections::HashSet::new();
    for hit in arg_hits {
        let res = match results.iter().find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig) {
            Some(r) => r, None => continue,
        };
        let class = locus_class(res).to_string();
        if !sel.classes.iter().any(|c| *c == class) { continue; }

        let contig_key = hit.contig.split_whitespace().next().unwrap_or(&hit.contig);
        if !seen.insert((contig_key.to_string(), hit.contig_start, hit.contig_end)) { continue; }

        let genus = res.genus.clone().unwrap_or_else(|| NA.to_string());
        let flankdb = flankdb_field(res);

        // gene part
        if let Some(w) = files.get_mut(&(class.clone(), "gene".to_string())) {
            if let Some(cseq) = contig_map.get(contig_key) {
                let (s, e) = (hit.contig_start.min(cseq.len()), hit.contig_end.min(cseq.len()));
                if e > s {
                    let mut gene = cseq[s..e].to_string();
                    if hit.strand == '-' { gene = revcomp_str(&gene); }
                    let hdr = locus_hdr(sample_name, contig_key, "gene", &hit.arg_name, &hit.arg_class,
                                        &format!("{:.1}", hit.identity), &format!("{:.1}", hit.coverage),
                                        &format!("{}-{}", s, e), NA, &genus, NA, NA);
                    writeln!(w, "{}\n{}", hdr, gene)?;
                }
            }
        }
        // flank part (upstream + downstream)
        if let Some(w) = files.get_mut(&(class.clone(), "flank".to_string())) {
            if res.upstream_seq.len() >= MIN_FLANK {
                let hdr = locus_hdr(sample_name, contig_key, "flank_up", &hit.arg_name, &hit.arg_class,
                                    NA, NA, NA, &res.upstream_seq.len().to_string(), &genus, &flankdb, NA);
                writeln!(w, "{}\n{}", hdr, res.upstream_seq)?;
            }
            if res.downstream_seq.len() >= MIN_FLANK {
                let hdr = locus_hdr(sample_name, contig_key, "flank_down", &hit.arg_name, &hit.arg_class,
                                    NA, NA, NA, &res.downstream_seq.len().to_string(), &genus, &flankdb, NA);
                writeln!(w, "{}\n{}", hdr, res.downstream_seq)?;
            }
        }
    }
    Ok(())
}

/// Per-locus zone on a contig used to bin mapped reads.
struct LocusZone {
    gene: (usize, usize),
    up: (usize, usize),
    down: (usize, usize),
    arg: String,
    argclass: String,
    class: String,
    genus: String,
    flankdb: String,
}

/// Emit reads-state per-locus outputs: map filtered reads to the contigs, bin each
/// read by the region it overlaps (gene vs flanking; overlap-both → flank), and write
/// the read sequences with the uniform header. Uses minimap2 `-a` so read sequences
/// come straight from the SAM (no fastq re-streaming).
#[allow(clippy::too_many_arguments)]
fn emit_locus_reads(
    sample_name: &str,
    arg_hits: &[ArgHit],
    contigs: &[FastaRecord],
    results: &[GenusResult],
    r1: &Path,
    r2: &Path,
    sample_dir: &Path,
    sel: &EmitSel,
    args: &Args,
) -> Result<()> {
    // Open selected {class}.{part}.reads files.
    let mut files: HashMap<(String, String), BufWriter<File>> = HashMap::new();
    for class in &sel.classes {
        for part in &sel.parts {
            if sel.wants(class, part, "reads") {
                let path = sample_dir.join(format!("{}.{}.reads.fasta", class, part));
                files.insert((class.clone(), part.clone()), BufWriter::new(File::create(path)?));
            }
        }
    }
    if files.is_empty() { return Ok(()); }

    let clen: HashMap<&str, usize> = contigs.iter()
        .map(|c| (c.name.split_whitespace().next().unwrap_or(&c.name), c.seq.len())).collect();

    // Build per-contig zones for selected classes (dedup by contig region).
    let maxf = args.max_flanking;
    let mut zones: HashMap<String, Vec<LocusZone>> = HashMap::new();
    let mut seen: std::collections::HashSet<(String, usize, usize)> = std::collections::HashSet::new();
    for hit in arg_hits {
        let res = match results.iter().find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig) {
            Some(r) => r, None => continue,
        };
        let class = locus_class(res).to_string();
        if !sel.classes.iter().any(|c| *c == class) { continue; }
        let ckey = hit.contig.split_whitespace().next().unwrap_or(&hit.contig).to_string();
        if !seen.insert((ckey.clone(), hit.contig_start, hit.contig_end)) { continue; }
        let cl = *clen.get(ckey.as_str()).unwrap_or(&0);
        let (s, e) = (hit.contig_start.min(cl), hit.contig_end.min(cl));
        zones.entry(ckey).or_default().push(LocusZone {
            gene: (s, e),
            up: (s.saturating_sub(maxf), s),
            down: (e, (e + maxf).min(cl)),
            arg: hit.arg_name.clone(),
            argclass: hit.arg_class.clone(),
            class,
            genus: res.genus.clone().unwrap_or_else(|| NA.to_string()),
            flankdb: flankdb_field(res),
        });
    }
    if zones.is_empty() { return Ok(()); }

    // Map filtered reads to the contigs (SAM has the read sequences).
    let contigs_fa = sample_dir.join("emit_reads_contigs.fasta");
    write_contigs_simple(contigs, &contigs_fa)?;
    let sam_path = sample_dir.join("emit_reads.sam");
    let sam_file = File::create(&sam_path)?;
    Command::new(&args.minimap2)
        .args(["-a", "-x", "sr", "-t", &args.threads.to_string()])
        .arg(&contigs_fa).arg(r1).arg(r2)
        .stdout(std::process::Stdio::from(sam_file))
        .stderr(std::process::Stdio::null())
        .status().context("minimap2 read->contig mapping failed")?;

    let ovl = |a: (usize, usize), rs: usize, re: usize| a.1 > a.0 && rs.max(a.0) < re.min(a.1);
    let reader = std::io::BufReader::new(File::open(&sam_path)?);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with('@') { continue; }
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 11 { continue; }
        let (rname, seq) = (c[2], c[9]);
        if rname == "*" || seq == "*" || seq.is_empty() { continue; }
        let pos: usize = match c[3].parse::<usize>() { Ok(p) if p > 0 => p - 1, _ => continue };
        let (rs, re) = (pos, pos + seq.len());
        let locs = match zones.get(rname) { Some(v) => v, None => continue };
        for z in locs {
            // Overlap-both → flank; check flanks first.
            let (part_dim, part_lbl) = if ovl(z.up, rs, re) {
                ("flank", "flank_up")
            } else if ovl(z.down, rs, re) {
                ("flank", "flank_down")
            } else if ovl(z.gene, rs, re) {
                ("gene", "gene")
            } else { continue };
            if let Some(w) = files.get_mut(&(z.class.clone(), part_dim.to_string())) {
                let hdr = locus_hdr(sample_name, rname, part_lbl, &z.arg, &z.argclass,
                                    NA, NA, NA, NA, &z.genus, &z.flankdb, c[0]);
                writeln!(w, "{}\n{}", hdr, seq)?;
            }
            break; // assign each read to one locus
        }
    }
    let _ = fs::remove_file(&contigs_fa);
    let _ = fs::remove_file(&sam_path);
    Ok(())
}

/// Classify a pre-assembled contig FASTA: ARG detection + genus classification +
/// unresolved exports, without the read-filter/assembly stages. Used to A/B-test
/// alternative assemblies (e.g. per-locus reassembly) against the baseline.
fn run_classify_contigs_mode(contigs_fa: &Path, args: &Args) -> Result<()> {
    let sample_name = contigs_fa.file_stem().and_then(|s| s.to_str()).unwrap_or("classify").to_string();
    let sample_dir = args.outdir.join(&sample_name);
    fs::create_dir_all(&sample_dir)?;

    if args.verbose {
        eprintln!("[classify-contigs] Loading {}", contigs_fa.display());
    }
    let contigs = load_and_filter_contigs(contigs_fa, args.min_contig_len)?;
    if contigs.is_empty() {
        anyhow::bail!("No contigs >= {} bp in {}", args.min_contig_len, contigs_fa.display());
    }
    if args.verbose {
        eprintln!("[classify-contigs] {} contigs; detecting ARGs...", contigs.len());
    }

    let contigs_path = sample_dir.join("contigs_input.fasta");
    write_contigs_simple(&contigs, &contigs_path)?;

    let arg_hits = detect_args_blast(&contigs_path, args.arg_db.as_ref().unwrap(), &sample_dir,
                                     &args.blastn, &args.makeblastdb,
                                     args.arg_identity, args.arg_coverage, args.threads)?;
    let unique_args = deduplicate_args(arg_hits);
    if args.verbose {
        eprintln!("[classify-contigs] ARGs detected: {}", unique_args.len());
    }
    if unique_args.is_empty() {
        output_results(&[], args)?;
        return Ok(());
    }

    let genus_results = classify_genera(&unique_args, &contigs, args)?;

    let results: Vec<ResultRow> = unique_args.iter().map(|hit| {
        let g = genus_results.iter()
            .find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig)
            .cloned().unwrap_or_default();
        let top_matches_str = g.top_matches.iter()
            .map(|(gg, s)| format!("{}:{:.1}", gg, s)).collect::<Vec<_>>().join(";");
        let specificity = if g.specificity <= 1.0 { g.specificity * 100.0 } else { g.specificity };
        let genus_call = format_genus_call(&g);
        let species_call = format_species_call(&g);
        ResultRow {
            sample: sample_name.clone(),
            contig_id: hit.contig.clone(),
            arg_name: hit.arg_name.clone(),
            arg_class: hit.arg_class.clone(),
            genus: genus_call,
            confidence: g.confidence,
            specificity,
            identity: hit.identity,
            coverage: hit.coverage,
            contig_len: hit.contig_len,
            arg_start: hit.contig_start,
            arg_end: hit.contig_end,
            upstream_len: g.upstream_len,
            downstream_len: g.downstream_len,
            extension_method: "classify".to_string(),
            top_matches: top_matches_str,
            snp_status: format!("{}", g.snp_status),
            context: g.context.clone(),
            species: species_call,
            credible_set: if g.credible_set_grouped.is_empty() {
                "NA".to_string()
            } else {
                g.credible_set_grouped.clone()
            },
            support: g.support,
            resolution_distance: g.resolution_distance,
            limited_by: g.limited_by.clone(),
            resolution_rank: g.resolution_rank.clone(),
            resolution_taxon: g.resolution_taxon.clone(),
        }
    }).collect();

    let emit_sel = EmitSel::resolve(args)?;
    if emit_sel.on {
        if let Err(e) = emit_locus_asm(&sample_name, &unique_args, &contigs, &genus_results, &sample_dir, &emit_sel) {
            eprintln!("[classify-contigs] (warning) locus asm emit failed: {}", e);
        }
        if emit_sel.states.iter().any(|s| s == "reads") {
            eprintln!("[classify-contigs] note: 'reads' state needs the read pipeline; skipped in --classify-contigs");
        }
        if args.verbose { eprintln!("[classify-contigs] Per-locus outputs written ({} classes)", emit_sel.classes.len()); }
    }

    output_results(&results, args)?;
    if args.verbose {
        let resolved = results.iter().filter(|r| r.genus != "Unknown").count();
        eprintln!("[classify-contigs] Done: {}/{} loci resolved to a genus", resolved, results.len());
    }
    Ok(())
}

/// Detect ARGs by aligning assembled contigs against the ARG reference FASTA with
/// BLAST (dc-megablast), the standard for assembly-based ARG detection (ResFinder
/// FASTA path, abricate, AMRFinderPlus all use blastn). Replaces the earlier minimap2
/// step: minimap2 is a best-placement mapper whose chaining silently drops short genes
/// packed in integrons/cassettes (e.g. dfrA12) and divergent hits; a benchmark over
/// 1000+ GTDB genomes showed minimap2 recovering only ~62% (asm20) / 97% (sensitive) of
/// BLAST's loci at equal cost. dc-megablast's spaced (discontiguous) seed keeps
/// sensitivity across the ~80–95% cross-species regime that plain megablast misses.
///
/// HSPs of the same reference sitting close on a contig are merged into one locus before
/// the coverage/identity filter, so a gene split into several HSPs is not lost. Redundant
/// hits (PanRes maps one gene to many near-identical refs) are collapsed later by
/// deduplicate_args().
fn detect_args_blast(
    contigs: &Path,
    arg_db_fasta: &Path,
    output_dir: &Path,
    blastn: &str,
    makeblastdb: &str,
    min_identity: f64,
    min_coverage: f64,
    threads: usize,
) -> Result<Vec<ArgHit>> {
    use std::collections::HashMap;
    let min_identity_pct = min_identity * 100.0;
    let min_coverage_pct = min_coverage * 100.0;

    // Build a nucleotide BLAST DB from the ARG reference FASTA (PanRes ~0.05s). Rebuilt
    // per run into the sample dir to avoid stale/locked shared-DB files.
    let db_prefix = output_dir.join("argdb_blast");
    let status = Command::new(makeblastdb)
        .args(["-in", arg_db_fasta.to_str().unwrap(), "-dbtype", "nucl",
               "-out", db_prefix.to_str().unwrap()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("Failed to run makeblastdb on {}", arg_db_fasta.display()))?;
    if !status.success() {
        anyhow::bail!("makeblastdb failed on {} — BLAST detection needs the ARG FASTA (e.g. PanRes_genes_v1.0.0.fa), not an .mmi", arg_db_fasta.display());
    }

    // -max_target_seqs is intentionally huge: PanRes is highly redundant, so one
    // chromosome matches thousands of near-identical entries and a low cap silently
    // truncates real genes (a low cap caused ~4x undercounting in benchmarking).
    let out = output_dir.join("contigs_to_argdb.blast6");
    let perc = format!("{}", min_identity_pct);
    let status = Command::new(blastn)
        .args(["-task", "dc-megablast",
               "-query", contigs.to_str().unwrap(),
               "-db", db_prefix.to_str().unwrap(),
               "-outfmt", "6 qseqid qlen sseqid slen pident length qstart qend sstart send",
               "-perc_identity", &perc,
               "-max_target_seqs", "100000",
               "-evalue", "1e-10",
               "-num_threads", &threads.to_string(),
               "-out", out.to_str().unwrap()])
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to run blastn (dc-megablast)")?;
    if !status.success() {
        anyhow::bail!("blastn (dc-megablast) failed");
    }

    // Group HSPs by (contig, reference), then merge those close on the contig into one
    // physical locus before applying the coverage/identity filter.
    struct Hsp { qs: usize, qe: usize, ss: usize, se: usize, fwd: bool, matches: f64, alen: usize }
    let mut groups: HashMap<(String, String), (usize, usize, Vec<Hsp>)> = HashMap::new();
    let content = std::fs::read_to_string(&out)
        .with_context(|| format!("Failed to read {}", out.display()))?;
    for line in content.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 10 { continue; }
        let qlen: usize = f[1].parse().unwrap_or(0);
        let slen: usize = f[3].parse().unwrap_or(0);
        let pident: f64 = f[4].parse().unwrap_or(0.0);
        let length: usize = f[5].parse().unwrap_or(0);
        let qstart: usize = f[6].parse().unwrap_or(0);
        let qend: usize = f[7].parse().unwrap_or(0);
        let sstart: usize = f[8].parse().unwrap_or(0);
        let send: usize = f[9].parse().unwrap_or(0);
        if slen == 0 || length == 0 { continue; }
        let entry = groups.entry((f[0].to_string(), f[2].to_string()))
            .or_insert((qlen, slen, Vec::new()));
        entry.2.push(Hsp {
            qs: qstart.min(qend), qe: qstart.max(qend),
            ss: sstart.min(send), se: sstart.max(send),
            fwd: sstart <= send,
            matches: pident / 100.0 * length as f64,
            alen: length,
        });
    }

    let mut hits = Vec::new();
    for ((contig, refname), (qlen, slen, mut hsps)) in groups {
        hsps.sort_by_key(|h| h.qs);
        let mut i = 0;
        while i < hsps.len() {
            let mut j = i + 1;
            let mut cluster_end = hsps[i].qe;
            while j < hsps.len() && hsps[j].qs.saturating_sub(cluster_end) < slen {
                cluster_end = cluster_end.max(hsps[j].qe);
                j += 1;
            }
            let cluster = &hsps[i..j];
            // union of reference-covered intervals → true coverage across merged HSPs
            let mut ivs: Vec<(usize, usize)> = cluster.iter().map(|h| (h.ss, h.se)).collect();
            ivs.sort();
            let (mut cs, mut ce) = ivs[0];
            let mut cov = 0usize;
            for &(s, e) in &ivs[1..] {
                if s <= ce + 1 { ce = ce.max(e); }
                else { cov += ce - cs + 1; cs = s; ce = e; }
            }
            cov += ce - cs + 1;
            let refcov = cov as f64 / slen as f64 * 100.0;
            let sum_matches: f64 = cluster.iter().map(|h| h.matches).sum();
            let sum_alen: usize = cluster.iter().map(|h| h.alen).sum();
            let identity = if sum_alen > 0 { sum_matches / sum_alen as f64 * 100.0 } else { 0.0 };
            if refcov >= min_coverage_pct && identity >= min_identity_pct {
                let gstart = cluster.iter().map(|h| h.qs).min().unwrap();
                let gend = cluster.iter().map(|h| h.qe).max().unwrap();
                let fwd = cluster.iter().filter(|h| h.fwd).count() * 2 >= cluster.len();
                let parts: Vec<&str> = refname.split('|').collect();
                let arg_name = parts.first().unwrap_or(&"").to_string();
                hits.push(ArgHit {
                    members: vec![arg_name.clone()],
                    arg_name,
                    arg_class: parts.get(1).unwrap_or(&"UNKNOWN").to_string(),
                    contig: contig.clone(),
                    contig_len: qlen,
                    identity,
                    coverage: refcov,
                    contig_start: gstart.saturating_sub(1), // BLAST 1-based → 0-based half-open (matches old minimap2 coords)
                    contig_end: gend,
                    strand: if fwd { '+' } else { '-' },
                });
            }
            i = j;
        }
    }
    // Deterministic order: groups come from a HashMap (random iteration order), so sort
    // before returning. Otherwise, when several redundant PanRes refs tie at one locus
    // (same coords/identity/coverage), which one deduplicate_args keeps as the
    // representative would vary run-to-run. Final key = arg_name for a stable tiebreak.
    hits.sort_by(|a, b| a.contig.cmp(&b.contig)
        .then(a.contig_start.cmp(&b.contig_start))
        .then(a.contig_end.cmp(&b.contig_end))
        .then(a.arg_name.cmp(&b.arg_name)));
    Ok(hits)
}

/// Collapse redundant ARG hits that map to the SAME physical locus into one row.
///
/// A redundant, multi-source reference DB (e.g. PanRes aggregates ResFinder + CARD +
/// AMRFinder + MEGARes + …) makes one gene copy align to many near-identical reference
/// entries; each surfaces as a separate hit at ~the same contig coordinates. Keying on
/// the reference/gene name (as before) does NOT collapse these — every variant is a
/// distinct name. We instead group by contig + overlapping ARG span using the TRUE
/// contig coordinates (not the max-flanking-capped upstream/downstream lengths, which
/// are unreliable for this) and keep the single best representative per locus.
///
/// Distinct genes on one contig (e.g. an integron cassette: sul1 + aadA2 + dfrA12) sit
/// at different, non-overlapping spans and therefore remain SEPARATE loci.
fn deduplicate_args(mut hits: Vec<ArgHit>) -> Vec<ArgHit> {
    // Representative preference within a locus: higher identity, then higher coverage,
    // then the SHORTER span. Shorter wins the final tie so a tight, gene-length allele
    // is chosen over a padded consensus reference (e.g. a MEGARes entry longer than the
    // gene), keeping the reported ARG_Start/End close to the true gene boundary. A real
    // partial fragment is already beaten earlier on coverage, so "shorter" here is safe.
    fn better(a: &ArgHit, b: &ArgHit) -> bool {
        use std::cmp::Ordering::{Greater, Less};
        match a.identity.partial_cmp(&b.identity) {
            Some(Greater) => return true,
            Some(Less) => return false,
            _ => {}
        }
        match a.coverage.partial_cmp(&b.coverage) {
            Some(Greater) => return true,
            Some(Less) => return false,
            _ => {}
        }
        let sa = a.contig_end.saturating_sub(a.contig_start);
        let sb = b.contig_end.saturating_sub(b.contig_start);
        sa < sb
    }

    // Finalize an open locus: keep the same representative as before, and additionally
    // record the member IDs of every ref TIED with it (equal identity AND coverage) —
    // the redundant PanRes representations of the same physical gene. Their flanking
    // reference sets are unioned at classification time so no source DB's evidence is
    // dropped. Lower-identity overlapping refs (more divergent genes) are not merged.
    fn finalize(group: Vec<ArgHit>) -> ArgHit {
        let mut best = 0;
        for i in 1..group.len() {
            if better(&group[i], &group[best]) { best = i; }
        }
        let (rid, rcov) = (group[best].identity, group[best].coverage);
        let mut members: Vec<String> = Vec::new();
        for h in &group {
            if (h.identity - rid).abs() < 1e-9 && (h.coverage - rcov).abs() < 1e-9 {
                for m in &h.members {
                    if !members.contains(m) { members.push(m.clone()); }
                }
            }
        }
        members.sort();
        let mut out = group.into_iter().nth(best).unwrap();
        out.members = members;
        out
    }

    hits.sort_by(|a, b| {
        a.contig.cmp(&b.contig)
            .then(a.contig_start.cmp(&b.contig_start))
            .then(b.contig_end.cmp(&a.contig_end))
    });

    let mut result: Vec<ArgHit> = Vec::new();
    let mut group: Vec<ArgHit> = Vec::new(); // all hits of the currently open locus
    let mut rep: Option<ArgHit> = None;      // best-so-far of the open locus (overlap test)
    for hit in hits {
        // Same locus iff same contig and ≥50% reciprocal overlap (of the shorter span)
        // with the current representative. Same-gene variants share ~identical spans;
        // a neighbouring cassette gene barely overlaps → opens a new locus.
        let same = match &rep {
            Some(r) if r.contig == hit.contig => {
                let ov = hit.contig_start.max(r.contig_start);
                let oe = hit.contig_end.min(r.contig_end);
                let overlap = oe.saturating_sub(ov);
                let shorter = (hit.contig_end.saturating_sub(hit.contig_start))
                    .min(r.contig_end.saturating_sub(r.contig_start))
                    .max(1);
                overlap * 2 >= shorter
            }
            _ => false,
        };
        if !same {
            if !group.is_empty() { result.push(finalize(std::mem::take(&mut group))); }
            rep = None;
        }
        if rep.as_ref().map_or(true, |r| better(&hit, r)) {
            rep = Some(hit.clone());
        }
        group.push(hit);
    }
    if !group.is_empty() {
        result.push(finalize(group));
    }

    result.sort_by(|a, b| b.coverage.partial_cmp(&a.coverage).unwrap_or(std::cmp::Ordering::Equal));
    result
}

#[cfg(test)]
mod sam2paf_tests {
    use super::*;
    use std::io::Write as _;

    /// Parse a PAF file into (query, matches, block_len) tuples.
    fn read_paf(path: &Path) -> Vec<(String, usize, usize)> {
        let mut out = Vec::new();
        for line in fs::read_to_string(path).unwrap().lines() {
            let f: Vec<&str> = line.split('\t').collect();
            out.push((f[0].to_string(), f[9].parse().unwrap(), f[10].parse().unwrap()));
        }
        out
    }

    #[test]
    fn builtin_sam_to_paf_matches_reference_semantics() {
        let dir = std::env::temp_dir();
        let sam = dir.join("argenus_test_in.sam");
        let paf = dir.join("argenus_test_out.paf");
        let mut w = File::create(&sam).unwrap();
        writeln!(w, "@SQ\tSN:gene1\tLN:1000").unwrap();
        // A: 150M, NM=3 -> block_len=150, matches=147 (identity 98%).
        writeln!(w, "readA\t0\tgene1\t101\t60\t150M\t*\t0\t0\tACGT\t*\tNM:i:3").unwrap();
        // B: 100M, NM=30 -> block_len=100, matches=70 (identity 70%).
        writeln!(w, "readB\t0\tgene1\t1\t60\t100M\t*\t0\t0\tACGT\t*\tNM:i:30").unwrap();
        // C: unmapped (FLAG 4) -> dropped.
        writeln!(w, "readC\t4\t*\t0\t0\t*\t*\t0\t0\tACGT\t*").unwrap();
        // D: 48M2I50M, NM=2 (2 inserted bases) -> m_ops=98, mm=2-2=0, matches=98, block_len=100.
        writeln!(w, "readD\t0\tgene1\t5\t60\t48M2I50M\t*\t0\t0\tACGT\t*\tNM:i:2").unwrap();
        // E: 10S140M reverse, NM=0 -> block_len=140, matches=140.
        writeln!(w, "readE\t16\tgene1\t20\t60\t10S140M\t*\t0\t0\tACGT\t*\tNM:i:0").unwrap();
        // F: mapped, no NM tag -> dropped.
        writeln!(w, "readF\t0\tgene1\t1\t60\t100M\t*\t0\t0\tACGT\t*").unwrap();
        drop(w);

        sam_to_paf_builtin(&sam, &paf).unwrap();
        let recs = read_paf(&paf);
        let get = |q: &str| recs.iter().find(|r| r.0 == q).cloned();

        assert_eq!(get("readA"), Some(("readA".into(), 147, 150)));
        assert_eq!(get("readB"), Some(("readB".into(), 70, 100)));
        assert_eq!(get("readC"), None, "unmapped read must be dropped");
        assert_eq!(get("readD"), Some(("readD".into(), 98, 100)));
        assert_eq!(get("readE"), Some(("readE".into(), 140, 140)));
        assert_eq!(get("readF"), None, "record without NM tag must be dropped");

        let a = get("readA").unwrap();
        assert!(((a.1 as f64 / a.2 as f64) * 100.0 - 98.0).abs() < 1e-9);
    }
}
