mod seqio;
mod paf;
mod extender;
mod classifier;
mod snp;
mod flanking_db;
mod arg_db;
mod fdb;

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

fn parse_arg_identity(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| format!("Invalid number: {}", s))?;
    if !(0.8..=1.0).contains(&val) {
        Err(format!("ARG identity must be between 0.8 and 1.0, got {}", val))
    } else {
        Ok(val)
    }
}

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
  Reads → minimap2 filter → MEGAHIT assembly → Extension → ARG detection → Genus classification

ALIGNMENT TIE-BREAKING (for equal-score hits):
  Priority: Score (higher first) → Gene length (higher first) → MapQ (higher first)
            → Divergence (lower first) → Gap count (lower first) → Gene name (alphabetical)

OUTPUT FILES:
  results.tsv          Main output with detected ARGs and genus assignments
    Columns: Sample, ARG_Name, ARG_Class, Genus, Confidence, Specificity,
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
For more information, visit: https://github.com/your-repo/argenus
"#)]
struct Args {

    #[arg(short = '1', long, value_name = "FILE(S)", help_heading = "Input")]
    r1: Option<String>,

    #[arg(short = '2', long, value_name = "FILE(S)", help_heading = "Input")]
    r2: Option<String>,

    #[arg(short = 'l', long, value_name = "PATH", help_heading = "Input")]
    samples: Option<PathBuf>,

    #[arg(short = 'b', long = "build-db", value_name = "TYPE", help_heading = "Database")]
    build_db: Option<String>,

    #[arg(short = 'x', long, value_name = "SOURCE", default_value = "ncbi", help_heading = "Database")]
    source: String,

    #[arg(long = "unified-db", value_name = "FILE", help_heading = "Database")]
    unified_db: Option<PathBuf>,

    #[arg(short = 'a', long = "arg-db", value_name = "FILE", help_heading = "Database")]
    arg_db: Option<PathBuf>,

    #[arg(short = 'f', long = "flanking-db", value_name = "FILE", help_heading = "Database")]
    flanking_db: Option<PathBuf>,

    #[arg(short = 'e', long, value_name = "EMAIL", help_heading = "Database")]
    email: Option<String>,

    #[arg(short = 'p', long = "flanking-length", value_name = "BP", default_value = "1050", help_heading = "Database")]
    flanking_length: usize,

    #[arg(short = 'q', long = "queue-buffer", value_name = "GB", default_value = "30", help_heading = "Database")]
    queue_buffer_gb: u32,

    #[arg(short = 'd', long = "plsdb-dir", value_name = "DIR", help_heading = "Database")]
    plsdb_dir: Option<PathBuf>,

    #[arg(short = 'z', long = "skip-plsdb", help_heading = "Database")]
    skip_plsdb: bool,

    #[arg(short = 'o', long, value_name = "DIR", default_value = ".", help_heading = "Output")]
    outdir: PathBuf,

    #[arg(short = 'u', long, help_heading = "Output")]
    keep_temp: bool,

    #[arg(short = 'v', long, help_heading = "Output")]
    verbose: bool,

    #[arg(long = "all-hits", help_heading = "Output")]
    all_hits: bool,

    #[arg(short = 'i', long = "arg-identity", value_name = "FLOAT",
          default_value = "0.80", value_parser = parse_arg_identity, help_heading = "ARG Detection")]
    arg_identity: f64,

    #[arg(short = 'c', long = "arg-coverage", value_name = "FLOAT",
          default_value = "0.70", value_parser = parse_arg_coverage, help_heading = "ARG Detection")]
    arg_coverage: f64,

    #[arg(short = 'r', long, value_name = "PERCENT", default_value = "95", help_heading = "Genus Classification")]
    resolution: f64,

    #[arg(short = 'n', long, value_name = "BP", default_value = "1050", help_heading = "Genus Classification")]
    max_flanking: usize,

    #[arg(short = 'g', long, value_name = "BP", default_value = "200", help_heading = "Assembly")]
    min_contig_len: usize,

    #[arg(short = 'k', long, value_name = "SIZE", default_value = "62", help_heading = "Assembly")]
    ext_kmer_size: usize,

    #[arg(short = 'j', long, value_name = "BP", default_value = "100", help_heading = "Assembly")]
    ext_length: usize,

    #[arg(short = 'm', long, value_name = "FLOAT", default_value = "0.80", help_heading = "Read Filtering")]
    identity: f64,

    #[arg(short = 'w', long, value_name = "BP", default_value = "50", help_heading = "Read Filtering")]
    min_align_len: usize,

    #[arg(short = 't', long, value_name = "NUM", default_value = "0", help_heading = "Runtime")]
    threads: usize,

    #[arg(short = 's', long, value_name = "NUM", default_value = "8", help_heading = "Runtime")]
    threads_per_sample: usize,

    #[arg(short = 'y', long, help_heading = "Runtime")]
    yes: bool,

    #[arg(skip)]
    minimap2: String,

    #[arg(skip)]
    megahit: String,
}

fn find_executable(name: &str) -> Result<PathBuf> {

    let path = Path::new(name);
    if path.is_absolute() && path.exists() {
        return Ok(path.to_path_buf());
    }

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

#[derive(Debug, Clone)]
struct Sample {
    name: String,
    r1: PathBuf,
    r2: PathBuf,
}

#[derive(Debug, Clone)]
struct ResultRow {
    sample: String,
    arg_name: String,
    arg_class: String,
    genus: String,
    confidence: f64,
    specificity: f64,
    identity: f64,
    coverage: f64,
    contig_len: usize,
    upstream_len: usize,
    downstream_len: usize,
    extension_method: String,
    top_matches: String,
    snp_status: String,
}

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
}

fn validate_arg_db_file(path: &Path) -> Result<(bool, bool)> {
    use std::io::{BufRead, BufReader, Read};

    let mut file = std::fs::File::open(path)?;

    let mut header = [0u8; 16];
    let bytes_read = file.read(&mut header)?;

    if bytes_read < 4 {
        anyhow::bail!(
            "ARG database file too small: {}\n\
             Expected FASTA or minimap2 index file",
            path.display()
        );
    }

    if header[..bytes_read].contains(&0u8) {
        return Ok((false, true));
    }

    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

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

    let seq_line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => anyhow::bail!("Failed to read sequence: {}", e),
        None => anyhow::bail!("Invalid FASTA: no sequence after header in {}", path.display()),
    };

    if seq_line.trim().is_empty() {
        anyhow::bail!("Invalid FASTA: empty sequence in {}", path.display());
    }

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

fn handle_build_db(db_type: &str, source: &str, output_dir: &Path, threads: usize, email: Option<&str>, arg_db: Option<&Path>, unified_db: Option<&Path>, config: crate::flanking_db::FlankBuildConfig) -> Result<()> {
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

            if !arg_db.exists() {
                anyhow::bail!(
                    "AMR database not found: {}\n\
                     Build it first with: argenus -b arg -o ./db",
                    arg_db.display()
                );
            }

            let (is_fasta, is_mmi) = validate_arg_db_file(arg_db)?;

            if !is_fasta && !is_mmi {
                anyhow::bail!(
                    "Invalid ARG database file: {}\n\
                     File must be FASTA (valid sequences) or minimap2 index (.mmi)\n\
                     Build with: argenus -b arg -o ./db",
                    arg_db.display()
                );
            }

            let arg_db = if is_mmi {

                arg_db.to_path_buf()
            } else {

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

            let email = match email {
                Some(e) => e,
                None => {
                    anyhow::bail!(
                        "--email is required for flanking database build.\n\
                         Example: argenus -b flank -a ./db/AMR_NCBI.mmi -o ./db -e your@email.com\n\n\
                         NCBI requires email for API access. Register at:\n\
                         https://www.ncbi.nlm.nih.gov/account/"
                    );
                }
            };

            eprintln!("============================================================");
            eprintln!(" ARGenus Database Builder - Flanking Sequence Database");
            eprintln!("============================================================");
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

            arg_db::build_flanking_db(output_dir, &arg_db, threads, email, config)
        }
        _ => {
            anyhow::bail!(
                "Unknown database type '{}'. Use 'arg' or 'flank'.\n\
                 Examples:\n  \
                   argenus -b arg -o ./db    # Build AMR reference database\n  \
                   argenus -b flank -o ./db  # Build flanking sequence database",
                db_type
            );
        }
    }
}

fn main() -> Result<()> {
    let mut args = Args::parse();
    let start_time = Instant::now();

    if args.threads == 0 {
        args.threads = num_cpus::get();
    }

    if let Some(db_type) = &args.build_db {
        let config = crate::flanking_db::FlankBuildConfig {
            flanking_length: args.flanking_length,
            queue_buffer_gb: args.queue_buffer_gb,
            plsdb: crate::flanking_db::PlsdbOptions {
                dir: args.plsdb_dir.clone(),
                skip: args.skip_plsdb,
            },
        };
        return handle_build_db(db_type, &args.source, &args.outdir, args.threads, args.email.as_deref(), args.arg_db.as_deref(), args.unified_db.as_deref(), config);
    }

    if args.arg_db.is_none() {
        anyhow::bail!("--arg-db is required for analysis mode");
    }
    if args.flanking_db.is_none() {
        anyhow::bail!("--flanking-db is required for analysis mode");
    }

    args.minimap2 = find_executable("minimap2")?.to_string_lossy().to_string();
    args.megahit = find_executable("megahit")?.to_string_lossy().to_string();

    if args.verbose {
        eprintln!("Found minimap2: {}", args.minimap2);
        eprintln!("Found megahit: {}", args.megahit);
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .ok();

    let samples = parse_samples(&args)?;
    if samples.is_empty() {
        anyhow::bail!("No samples provided. Use -1/-2 or --samples");
    }

    let max_concurrent = (args.threads / args.threads_per_sample).max(1);

    if args.verbose {
        eprintln!("Processing {} sample(s) with {} threads ({} concurrent, {} threads/sample)",
                  samples.len(), args.threads, max_concurrent, args.threads_per_sample);
    }

    fs::create_dir_all(&args.outdir)?;

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

    let final_results = Arc::try_unwrap(all_results)
        .expect("All threads should have finished")
        .into_inner()
        .unwrap();
    output_results(&final_results, &args)?;

    if !args.keep_temp {

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

fn find_samples_in_dir(dir: &Path) -> Result<Vec<Sample>> {
    use std::collections::BTreeSet;

    let mut sample_ids: BTreeSet<String> = BTreeSet::new();

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

        for suffix in &r1_suffixes {
            if filename.ends_with(suffix) {
                let id = filename.strip_suffix(suffix).unwrap();
                sample_ids.insert(id.to_string());
                break;
            }
        }
    }

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

fn find_fastq_pair(base_dir: &Path, id: &str) -> Result<(PathBuf, PathBuf)> {

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

            samples = find_samples_in_dir(samples_path)?;
        } else {

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

        let r1_files: Vec<&str> = r1_str.split(',').collect();
        let r2_files: Vec<&str> = r2_str.split(',').collect();

        if r1_files.len() != r2_files.len() {
            anyhow::bail!("Number of R1 and R2 files must match");
        }

        for (r1, r2) in r1_files.iter().zip(r2_files.iter()) {
            let r1_path = PathBuf::from(r1.trim());
            let r2_path = PathBuf::from(r2.trim());

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

    if !sample.r1.exists() {
        anyhow::bail!("R1 file not found: {:?}", sample.r1);
    }
    if !sample.r2.exists() {
        anyhow::bail!("R2 file not found: {:?}", sample.r2);
    }

    if args.verbose {
        eprintln!("  [1/6] Aligning reads to ARG database...");
    }
    let paf_path = run_minimap2_reads(&sample.r1, &sample.r2, args.arg_db.as_ref().unwrap(), &sample_dir, &args.minimap2, args.threads)?;
    let matching_reads = parse_paf_filter(&paf_path, args.identity, args.min_align_len)?;

    if args.verbose {
        eprintln!("        Reads passing filter: {}", matching_reads.len());
    }

    if matching_reads.is_empty() {
        return Ok(Vec::new());
    }

    let (filtered_r1, filtered_r2) = extract_read_pairs(&sample.r1, &sample.r2, &matching_reads, &sample_dir)?;

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

    if args.verbose {
        eprintln!("  [3/6] Extending contigs (strict)...");
    }
    let mut strict_contigs = extend_contigs_strict(&contigs, &filtered_r1, &filtered_r2, &sample_dir, args)?;

    for (i, c) in strict_contigs.iter_mut().enumerate() {
        c.name = format!("contig_{}", i + 1);
    }

    let contigs_path = sample_dir.join("contigs_strict.fasta");
    write_contigs_simple(&strict_contigs, &contigs_path)?;

    if args.verbose {
        eprintln!("  [4/6] Detecting ARGs...");
    }
    let paf_contigs = run_minimap2_contigs(&contigs_path, args.arg_db.as_ref().unwrap(), &sample_dir, &args.minimap2, args.threads)?;
    let arg_hits = detect_args(&paf_contigs, args.arg_identity, args.arg_coverage)?;
    let unique_args = deduplicate_args(arg_hits);

    if unique_args.is_empty() {
        return Ok(Vec::new());
    }

    if args.verbose {
        eprintln!("        ARGs detected: {}", unique_args.len());
    }

    if args.verbose {
        eprintln!("  [5/6] Classifying genera (strict)...");
    }

    let genus_results = classify_genera(&unique_args, &strict_contigs, args)?;

    let min_flanking_for_resolve = 100;
    let unresolved_args: Vec<&ArgHit> = unique_args.iter()
        .filter(|hit| {
            genus_results.iter()
                .find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig)
                .map(|g| {

                    g.genus.is_none() ||
                    (g.upstream_len < min_flanking_for_resolve && g.downstream_len < min_flanking_for_resolve)
                })
                .unwrap_or(true)
        })
        .collect();

    let mut flexible_results: HashMap<String, GenusResult> = HashMap::new();

    if !unresolved_args.is_empty() {
        if args.verbose {
            eprintln!("  [6/6] Extending {} unresolved contigs with Rust (flexible)...", unresolved_args.len());
        }

        let unresolved_contig_names: FxHashSet<String> = unresolved_args.iter()
            .map(|h| h.contig.clone())
            .collect();

        let contigs_to_extend: Vec<FastaRecord> = strict_contigs.iter()
            .filter(|c| unresolved_contig_names.contains(&c.name))
            .cloned()
            .collect();

        if !contigs_to_extend.is_empty() {

            let flexible_contigs = extend_contigs_flexible(&contigs_to_extend, &filtered_r1, &filtered_r2, &sample_dir, args)?;

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

    let results: Vec<ResultRow> = unique_args.iter()
        .map(|hit| {
            let key = format!("{}:{}", hit.arg_name, hit.contig);

            let (genus_info, ext_method) = if let Some(flex_result) = flexible_results.get(&key) {
                if flex_result.genus.is_some() {
                    (flex_result.clone(), "flexible")
                } else {

                    let strict = genus_results.iter()
                        .find(|g| g.arg_name == hit.arg_name && g.contig_name == hit.contig)
                        .cloned()
                        .unwrap_or_default();
                    (strict, "strict")
                }
            } else {

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

            let specificity = if genus_info.specificity <= 1.0 {
                genus_info.specificity * 100.0
            } else {
                genus_info.specificity
            };

            ResultRow {
                sample: sample.name.clone(),
                arg_name: hit.arg_name.clone(),
                arg_class: hit.arg_class.clone(),
                genus: genus_info.genus.unwrap_or_else(|| "Unknown".to_string()),
                confidence: genus_info.confidence,
                specificity,
                identity: hit.identity,
                coverage: hit.coverage,
                contig_len: hit.contig_len,
                upstream_len: genus_info.upstream_len,
                downstream_len: genus_info.downstream_len,
                extension_method: ext_method.to_string(),
                top_matches: top_matches_str,
                snp_status: format!("{}", genus_info.snp_status),
            }
        })
        .collect();

    Ok(results)
}

fn classify_genera(
    arg_hits: &[ArgHit],
    contigs: &[FastaRecord],
    args: &Args,
) -> Result<Vec<GenusResult>> {

    let contig_map: HashMap<String, String> = contigs.iter()
        .map(|c| (c.name.split_whitespace().next().unwrap_or(&c.name).to_string(), c.seq.clone()))
        .collect();

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
            })
        })
        .collect();

    if positions.is_empty() {
        return Ok(Vec::new());
    }

    if !args.flanking_db.as_ref().unwrap().exists() {

        if args.verbose {
            eprintln!("        Flanking database not found, skipping genus classification");
        }
        let results: Vec<GenusResult> = positions.iter()
            .map(|pos| {
                let upstream_len = pos.arg_start.min(args.max_flanking);
                let downstream_len = (pos.contig_len - pos.arg_end).min(args.max_flanking);

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
                }
            })
            .collect();
        return Ok(results);
    }

    let mut classifier = GenusClassifier::new(
        args.flanking_db.as_ref().unwrap(),
        &args.minimap2,
        0.90,
        100,
        args.max_flanking,
    )?;

    classifier.classify_batch(&positions, args.threads)
}

fn output_results(results: &[ResultRow], args: &Args) -> Result<()> {
    let header = "Sample\tARG_Name\tARG_Class\tGenus\tConfidence\tSpecificity\tARG_Identity\tARG_Coverage\tContig_Len\tUpstream_Len\tDownstream_Len\tExtension_Method\tSNP_Status\tTop_Matches";

    let output_results: Vec<_> = if args.all_hits {
        results.iter().collect()
    } else {
        results.iter()
            .filter(|r| r.snp_status != "WildType" && r.snp_status != "NotCovered")
            .collect()
    };

    let excluded_count = results.len() - output_results.len();

    let output_path = args.outdir.join("results.tsv");
    let mut output = BufWriter::new(File::create(&output_path)?);

    writeln!(output, "{}", header)?;

    for r in &output_results {
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}",
            r.sample,
            r.arg_name,
            r.arg_class,
            r.genus,
            r.confidence,
            r.specificity,
            r.identity,
            r.coverage,
            r.contig_len,
            r.upstream_len,
            r.downstream_len,
            r.extension_method,
            r.snp_status,
            r.top_matches
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

fn extend_contigs_strict(
    contigs: &[FastaRecord],
    r1: &Path,
    r2: &Path,
    sample_dir: &Path,
    args: &Args,
) -> Result<Vec<FastaRecord>> {

    let config = ExtenderConfig {
        kmer_size: args.ext_kmer_size,
        extension_step: args.ext_length,
        min_coverage: 3,
        branching_threshold: 0.1,
        max_n_ratio: 0.02,
        ..Default::default()
    };

    let mut extender = ContigExtender::new(config);
    extender.load_reads(r1, r2)?;

    let results = extender.extend_all_hybrid(contigs)?;

    let extended_path = sample_dir.join("contigs_strict.fasta");
    write_extended_contigs(&results, &extended_path)?;

    Ok(results.into_iter()
        .map(|r| FastaRecord { name: r.name, seq: r.extended_seq })
        .collect())
}

fn extend_contigs_flexible(
    contigs: &[FastaRecord],
    r1: &Path,
    r2: &Path,
    sample_dir: &Path,
    args: &Args,
) -> Result<Vec<FastaRecord>> {

    let config = ExtenderConfig {
        kmer_size: args.ext_kmer_size,
        extension_step: args.ext_length,
        min_coverage: 2,
        branching_threshold: 0.2,
        max_n_ratio: 0.05,
        ..Default::default()
    };

    let mut extender = ContigExtender::new(config);
    extender.load_reads(r1, r2)?;

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

fn write_contigs_simple(contigs: &[FastaRecord], path: &Path) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    for contig in contigs {
        writeln!(writer, ">{}", contig.name)?;
        writeln!(writer, "{}", contig.seq)?;
    }
    Ok(())
}

fn run_minimap2_contigs(contigs: &Path, db: &Path, output_dir: &Path, minimap2: &str, threads: usize) -> Result<PathBuf> {
    let paf_path = output_dir.join("contigs_to_argdb.paf");

    Command::new(minimap2)
        .args(["-x", "asm20", "-t", &threads.to_string(), "-c"])
        .arg(db).arg(contigs).arg("-o").arg(&paf_path)
        .stderr(std::process::Stdio::null())
        .status().context("Failed to run minimap2")?;

    Ok(paf_path)
}

fn detect_args(paf_path: &Path, min_identity: f64, min_coverage: f64) -> Result<Vec<ArgHit>> {
    let mut hits = Vec::new();
    let min_identity_pct = min_identity * 100.0;
    let min_coverage_pct = min_coverage * 100.0;

    let reader = PafReader::open(paf_path)?;
    for record in reader {
        let rec = record?;
        let identity = rec.calculate_identity();
        let coverage = rec.calculate_coverage();

        if identity >= min_identity_pct && coverage >= min_coverage_pct {
            let parts: Vec<&str> = rec.target_name.split('|').collect();
            let arg_name = parts.first().unwrap_or(&"").to_string();
            let arg_class = parts.get(1).unwrap_or(&"UNKNOWN").to_string();

            hits.push(ArgHit {
                arg_name,
                arg_class,
                contig: rec.query_name,
                contig_len: rec.query_len,
                identity,
                coverage,
                contig_start: rec.query_start,
                contig_end: rec.query_end,
                strand: rec.strand,
            });
        }
    }

    Ok(hits)
}

fn deduplicate_args(hits: Vec<ArgHit>) -> Vec<ArgHit> {
    let mut best: HashMap<String, ArgHit> = HashMap::new();

    for hit in hits {
        let key = hit.arg_name.clone();
        if let Some(existing) = best.get(&key) {
            if hit.identity > existing.identity {
                best.insert(key, hit);
            }
        } else {
            best.insert(key, hit);
        }
    }

    let mut result: Vec<ArgHit> = best.into_values().collect();
    result.sort_by(|a, b| b.coverage.partial_cmp(&a.coverage).unwrap_or(std::cmp::Ordering::Equal));
    result
}
