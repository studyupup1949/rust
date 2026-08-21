//! PAF to FDB Converter
//!
//! Converts PAF alignment results to Flanking Database (FDB) format
//! by extracting flanking sequences from genome files.
//!
//! Outputs:
//!   - flanking_db.tsv: FDB with metadata and sequences
//!   - flanking.fas: FASTA for minimap2 indexing
//!   - flanking.mmi: minimap2 index (optional, requires minimap2)
//!
//! Usage:
//!   cargo run --release --bin paf_to_fdb -- \
//!     -p input.paf \
//!     -g genomes_dir \
//!     -c genome_catalog.tsv \
//!     -o output_dir \
//!     -n 1050 \
//!     -t 32

use anyhow::{Context, Result};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// PAF hit for flanking extraction
#[derive(Clone, Debug)]
struct PafHit {
    contig_id: String,
    genome_file: String,
    query_start: usize,
    query_end: usize,
    strand: char,
    gene_name: String,
}

impl PafHit {
    fn from_paf_line(line: &str) -> Option<Self> {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 12 {
            return None;
        }

        // Query: contig_id|genome_file.fna
        let query = fields[0];
        let (contig_id, genome_file) = if let Some(pipe_pos) = query.rfind('|') {
            (query[..pipe_pos].to_string(), query[pipe_pos + 1..].to_string())
        } else {
            return None;
        };

        let query_start: usize = fields[2].parse().ok()?;
        let query_end: usize = fields[3].parse().ok()?;
        let strand = fields[4].chars().next().unwrap_or('+');

        // Target: gene_name|class|drug|atc
        let target = fields[5];
        let gene_name = target.split('|').next()?.to_string();

        Some(PafHit {
            contig_id,
            genome_file,
            query_start,
            query_end,
            strand,
            gene_name,
        })
    }
}

/// Flanking entry for output (Upstream + ARG + Downstream as single sequence)
struct FlankingEntry {
    gene: String,
    contig: String,
    genus: String,
    start: usize,
    end: usize,
    strand: char,
    sequence: String,  // Upstream + ARG + Downstream
}

/// Genome catalog for taxonomy lookup
struct GenomeCatalog {
    accession_to_genus: FxHashMap<String, String>,
}

impl GenomeCatalog {
    fn load(path: &Path) -> Result<Self> {
        let file = File::open(path).context("Failed to open genome catalog")?;
        let reader = BufReader::new(file);
        let mut accession_to_genus = FxHashMap::default();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if i == 0 && line.starts_with("accession") {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 4 {
                let accession = fields[0].to_string();
                let genus = fields[3].to_string();
                accession_to_genus.insert(accession, genus);
            }
        }

        eprintln!("Loaded {} entries from genome catalog", accession_to_genus.len());
        Ok(GenomeCatalog { accession_to_genus })
    }

    fn get_genus(&self, genome_file: &str) -> &str {
        let base = genome_file.trim_end_matches(".fna");

        if let Some(genus) = self.accession_to_genus.get(base) {
            return genus;
        }

        if let Some(dot_pos) = base.rfind('.') {
            let no_version = &base[..dot_pos];
            if let Some(genus) = self.accession_to_genus.get(no_version) {
                return genus;
            }
        }

        let with_dot = base.replace('_', ".");
        if let Some(genus) = self.accession_to_genus.get(&with_dot) {
            return genus;
        }

        "Unknown"
    }
}

/// Extract flanking sequences from a genome file
/// Returns: Upstream(flanking_length) + ARG + Downstream(flanking_length)
fn extract_flanking_from_genome(
    genome_path: &Path,
    hits: &[PafHit],
    catalog: &GenomeCatalog,
    flanking_length: usize,
) -> Result<Vec<FlankingEntry>> {
    // Read genome FASTA
    let file = File::open(genome_path)?;
    let reader = BufReader::new(file);

    // Parse FASTA: contig_id -> sequence
    let mut contigs: FxHashMap<String, String> = FxHashMap::default();
    let mut current_id: Option<String> = None;
    let mut current_seq = String::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(stripped) = line.strip_prefix('>') {
            if let Some(id) = current_id.take() {
                contigs.insert(id, std::mem::take(&mut current_seq));
            }
            let id = stripped.split_whitespace().next().unwrap_or("").to_string();
            current_id = Some(id);
        } else {
            current_seq.push_str(line.trim());
        }
    }
    if let Some(id) = current_id {
        contigs.insert(id, current_seq);
    }

    // Get genus from catalog
    let genome_file = genome_path.file_name().unwrap().to_str().unwrap();
    let genus = catalog.get_genus(genome_file).to_string();

    // Extract flanking for each hit
    let mut entries = Vec::new();
    for hit in hits {
        if let Some(seq) = contigs.get(&hit.contig_id) {
            let seq_len = seq.len();

            // Bounds check
            if hit.query_start >= seq_len || hit.query_end > seq_len {
                continue;
            }

            // Calculate extraction range: Upstream + ARG + Downstream
            let extract_start = hit.query_start.saturating_sub(flanking_length);
            let extract_end = std::cmp::min(hit.query_end + flanking_length, seq_len);

            // Extract full sequence (Upstream + ARG + Downstream)
            let full_sequence = seq[extract_start..extract_end].to_string();

            // For negative strand, reverse complement the entire sequence
            let final_sequence = if hit.strand == '-' {
                reverse_complement(&full_sequence)
            } else {
                full_sequence
            };

            entries.push(FlankingEntry {
                gene: hit.gene_name.clone(),
                contig: hit.contig_id.clone(),
                genus: genus.clone(),
                start: hit.query_start,
                end: hit.query_end,
                strand: hit.strand,
                sequence: final_sequence,
            });
        }
    }

    Ok(entries)
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
            _ => 'N',
        })
        .collect()
}

/// Build minimap2 index from FASTA
fn build_mmi_index(fasta_path: &Path, mmi_path: &Path) -> Result<bool> {
    eprintln!("\n[4] Building minimap2 index...");

    let result = Command::new("minimap2")
        .args(["-d", mmi_path.to_str().unwrap(), fasta_path.to_str().unwrap()])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                eprintln!("    Created {}", mmi_path.display());
                Ok(true)
            } else {
                eprintln!("    Warning: minimap2 indexing failed");
                eprintln!("    stderr: {}", String::from_utf8_lossy(&output.stderr));
                Ok(false)
            }
        }
        Err(_) => {
            eprintln!("    Warning: minimap2 not found, skipping index creation");
            eprintln!("    Run manually: minimap2 -d {} {}", mmi_path.display(), fasta_path.display());
            Ok(false)
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let mut paf_path: Option<PathBuf> = None;
    let mut genomes_dir: Option<PathBuf> = None;
    let mut catalog_path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut flanking_length: usize = 1050;
    let mut threads: usize = 32;
    let mut skip_mmi = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--paf" => {
                paf_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "-g" | "--genomes" => {
                genomes_dir = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "-c" | "--catalog" => {
                catalog_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "-o" | "--output" => {
                output_dir = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "-n" | "--flanking-length" => {
                flanking_length = args[i + 1].parse()?;
                i += 2;
            }
            "-t" | "--threads" => {
                threads = args[i + 1].parse()?;
                i += 2;
            }
            "--skip-mmi" => {
                skip_mmi = true;
                i += 1;
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    let paf_path = paf_path.context("Missing -p/--paf argument")?;
    let genomes_dir = genomes_dir.context("Missing -g/--genomes argument")?;
    let catalog_path = catalog_path.context("Missing -c/--catalog argument")?;
    let output_dir = output_dir.context("Missing -o/--output argument")?;

    // Create output directory if needed
    std::fs::create_dir_all(&output_dir)?;

    let fdb_path = output_dir.join("flanking_db.tsv");
    let fasta_path = output_dir.join("flanking.fas");
    let mmi_path = output_dir.join("flanking.mmi");

    eprintln!("=== PAF to FDB Converter ===");
    eprintln!("PAF: {:?}", paf_path);
    eprintln!("Genomes: {:?}", genomes_dir);
    eprintln!("Catalog: {:?}", catalog_path);
    eprintln!("Output dir: {:?}", output_dir);
    eprintln!("Flanking length: {}bp", flanking_length);
    eprintln!("Threads: {}", threads);
    eprintln!();

    // Set thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()?;

    let start_time = Instant::now();

    // [1] Load genome catalog
    eprintln!("[1] Loading genome catalog...");
    let catalog = GenomeCatalog::load(&catalog_path)?;

    // [2] Parse PAF and group by genome file
    eprintln!("\n[2] Parsing PAF file...");
    let paf_file = File::open(&paf_path)?;
    let reader = BufReader::new(paf_file);

    let mut genome_hits: FxHashMap<String, Vec<PafHit>> = FxHashMap::default();
    let mut total_hits = 0usize;

    for line in reader.lines() {
        let line = line?;
        if let Some(hit) = PafHit::from_paf_line(&line) {
            genome_hits.entry(hit.genome_file.clone())
                .or_default()
                .push(hit);
            total_hits += 1;
        }
    }

    eprintln!("    Parsed {} hits from {} genomes", total_hits, genome_hits.len());

    // [3] Extract flanking sequences in parallel
    eprintln!("\n[3] Extracting flanking sequences (Upstream + ARG + Downstream)...");
    let processed = AtomicUsize::new(0);
    let extracted = AtomicUsize::new(0);
    let missing = AtomicUsize::new(0);
    let total_genomes = genome_hits.len();

    let genome_list: Vec<_> = genome_hits.into_iter().collect();

    let results: Vec<Vec<FlankingEntry>> = genome_list
        .par_iter()
        .filter_map(|(genome_file, hits)| {
            let genome_path = genomes_dir.join(genome_file);

            let result = if genome_path.exists() {
                match extract_flanking_from_genome(&genome_path, hits, &catalog, flanking_length) {
                    Ok(entries) => {
                        extracted.fetch_add(entries.len(), Ordering::Relaxed);
                        Some(entries)
                    }
                    Err(_) => None,
                }
            } else {
                missing.fetch_add(1, Ordering::Relaxed);
                None
            };

            let p = processed.fetch_add(1, Ordering::Relaxed) + 1;
            if p.is_multiple_of(1000) || p == total_genomes {
                let pct = p as f64 / total_genomes as f64 * 100.0;
                eprint!("\r    {}/{} genomes ({:.1}%) - {} entries extracted",
                       p, total_genomes, pct, extracted.load(Ordering::Relaxed));
            }

            result
        })
        .collect();

    eprintln!();

    // Write FDB (TSV) and FASTA simultaneously
    eprintln!("\n    Writing FDB and FASTA...");
    let fdb_file = File::create(&fdb_path)?;
    let fasta_file = File::create(&fasta_path)?;
    let mut fdb_writer = BufWriter::new(fdb_file);
    let mut fasta_writer = BufWriter::new(fasta_file);

    // FDB header
    writeln!(fdb_writer, "Gene\tContig\tGenus\tStart\tEnd\tStrand\tSequence")?;

    let mut total_written = 0usize;
    let mut entry_id = 0usize;

    for entries in results {
        for entry in entries {
            // Write FDB entry
            writeln!(
                fdb_writer,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                entry.gene,
                entry.contig,
                entry.genus,
                entry.start,
                entry.end,
                entry.strand,
                entry.sequence
            )?;

            // Write FASTA entry
            // Header: >ID|Gene|Contig|Genus|Start|End|Strand
            writeln!(
                fasta_writer,
                ">{}|{}|{}|{}|{}|{}|{}",
                entry_id,
                entry.gene,
                entry.contig,
                entry.genus,
                entry.start,
                entry.end,
                entry.strand
            )?;
            writeln!(fasta_writer, "{}", entry.sequence)?;

            total_written += 1;
            entry_id += 1;
        }
    }
    fdb_writer.flush()?;
    fasta_writer.flush()?;

    let missing_count = missing.load(Ordering::Relaxed);

    eprintln!("    FDB: {} ({} entries)", fdb_path.display(), total_written);
    eprintln!("    FASTA: {}", fasta_path.display());

    // [4] Build minimap2 index
    let mut mmi_created = false;
    if !skip_mmi {
        mmi_created = build_mmi_index(&fasta_path, &mmi_path)?;

        // Remove intermediate FASTA file after successful mmi creation
        if mmi_created {
            std::fs::remove_file(&fasta_path)?;
            eprintln!("    Removed intermediate FASTA file");
        }
    } else {
        eprintln!("\n[4] Skipping minimap2 index (--skip-mmi)");
    }

    let elapsed = start_time.elapsed().as_secs_f64();

    // Summary
    eprintln!();
    eprintln!("=== Complete ===");
    eprintln!("Time: {:.1}s", elapsed);
    eprintln!("Input hits: {}", total_hits);
    eprintln!("Genomes processed: {}", total_genomes - missing_count);
    eprintln!("Missing genomes: {}", missing_count);
    eprintln!("Flanking entries: {}", total_written);
    eprintln!();
    eprintln!("Output files:");
    eprintln!("  - {}", fdb_path.display());
    if mmi_created {
        eprintln!("  - {}", mmi_path.display());
    } else {
        eprintln!("  - {}", fasta_path.display());
    }

    Ok(())
}

fn print_help() {
    eprintln!("PAF to FDB Converter");
    eprintln!();
    eprintln!("Converts PAF alignment results to Flanking Database (FDB).");
    eprintln!("Extracts: Upstream(N bp) + ARG + Downstream(N bp) as connected sequence.");
    eprintln!();
    eprintln!("Usage: paf_to_fdb [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -p, --paf <FILE>            Input PAF file (from minimap2 alignment)");
    eprintln!("  -g, --genomes <DIR>         Directory containing genome FASTA files");
    eprintln!("  -c, --catalog <FILE>        Genome catalog TSV (accession -> genus mapping)");
    eprintln!("  -o, --output <DIR>          Output directory for FDB files");
    eprintln!("  -n, --flanking-length <N>   Flanking length in bp (default: 1050)");
    eprintln!("  -t, --threads <N>           Number of threads (default: 32)");
    eprintln!("      --skip-mmi              Skip minimap2 index generation");
    eprintln!("  -h, --help                  Show this help");
    eprintln!();
    eprintln!("Output files:");
    eprintln!("  flanking_db.tsv   FDB with metadata and sequences");
    eprintln!("  flanking.mmi      minimap2 index (if minimap2 available)");
    eprintln!("  flanking.fas      FASTA (only if --skip-mmi or minimap2 fails)");
}
