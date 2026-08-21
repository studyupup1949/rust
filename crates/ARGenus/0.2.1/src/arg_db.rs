//! ARG Reference Database Builder Module
//!
//! Downloads and builds antimicrobial resistance gene (ARG) reference databases
//! from NCBI AMRFinderPlus or CARD (Comprehensive Antibiotic Resistance Database).
//!
//! # Architecture
//! - Nucleotide-based sequences (not protein-based)
//! - Metadata embedded in FASTA headers
//! - Proper deduplication with taxa tracking
//!
//! # Output Files
//! - `AMR_NCBI.fas` or `AMR_CARD.fas`: FASTA sequences (for minimap2 alignment)
//! - `AMR_NCBI.mmi` or `AMR_CARD.mmi`: minimap2 pre-built index
//!
//! # Data Sources
//! - **NCBI**: Official AMRFinderPlus database with curated gene names
//! - **CARD**: Comprehensive database with ~94% NCBI gene name mapping
//!
//! # Example
//! ```no_run
//! use argenus::arg_db;
//! use std::path::Path;
//!
//! // Build from NCBI (recommended)
//! arg_db::build(Path::new("./db"), "ncbi", 32).unwrap();
//!
//! // Build from CARD (includes CARD-only genes marked with '^')
//! arg_db::build(Path::new("./db"), "card", 32).unwrap();
//! ```

use anyhow::{Context, Result};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const NCBI_FTP_BASE: &str = "https://ftp.ncbi.nlm.nih.gov/pathogen/Antimicrobial_resistance/AMRFinderPlus/database/latest";
const CARD_DATA_URL: &str = "https://card.mcmaster.ca/latest/data";
// PanRes files from Zenodo: https://doi.org/10.5281/zenodo.8055115
const PANRES_FASTA_URL: &str = "https://zenodo.org/records/8055116/files/PanRes_genes_v1.0.0.fa?download=1";
const PANRES_TSV_URL: &str = "https://zenodo.org/records/8055116/files/PanRes_data_v1.0.0.tsv?download=1";

// ============================================================================
// Data Structures
// ============================================================================

/// Gene metadata for FASTA header generation.
#[derive(Clone, Debug)]
struct GeneMeta {
    /// Gene family grouping (for CARD-only detection).
    gene_family: String,
    /// NDARO drug class (e.g., "BETA-LACTAM").
    drug_class: String,
    /// Chemical class of target antibiotic.
    chemical_class: String,
    /// ATC (Anatomical Therapeutic Chemical) code.
    atc_code: String,
}

/// ARG metadata collected during build.
#[derive(Clone, Debug)]
struct ArgMetaDb {
    /// Total number of genes.
    gene_count: usize,
    /// Total taxa entries across all genes (for statistics).
    #[allow(dead_code)]
    total_taxa_entries: usize,
    /// Gene metadata indexed by gene_id.
    genes: FxHashMap<String, GeneMeta>,
}

/// Catalogue entry parsed from NCBI ReferenceGeneCatalog.txt.
#[derive(Clone, Debug)]
struct CatalogueEntry {
    /// Protein accession (RefSeq or GenBank). Empty for nucleotide-only entries.
    protein_accession: String,
    /// Allele name (e.g., gyrA_S83F for point mutations).
    allele: String,
    /// Gene family name (e.g., gyrA).
    gene_family: String,
    /// NDARO drug class.
    ndaro_class: String,
    /// Subtype (e.g., "AMR", "POINT").
    subtype: String,
    /// Nucleotide accession for coordinate-based fetch (16S, etc.).
    nucleotide_accession: Option<String>,
    /// Coordinates for nucleotide-based fetch.
    nuc_start: Option<u64>,
    nuc_stop: Option<u64>,
    nuc_strand: Option<String>,
}

/// CARD gene entry with metadata.
#[derive(Clone, Debug)]
struct CardGeneEntry {
    card_name: String,
    ncbi_name: Option<String>,
    sequence: String,
    drug_class: String,
}

// ============================================================================
// Drug Class Mapping
// ============================================================================

/// Maps NDARO drug class to chemical class and ATC code.
///
/// ATC (Anatomical Therapeutic Chemical) classification provides
/// standardised drug categorisation for international use.
fn get_chemical_and_atc(ndaro_class: &str) -> (&'static str, &'static str) {
    match ndaro_class {
        "AMINOGLYCOSIDE" => ("AMINOGLYCOSIDE", "J01G"),
        "BETA-LACTAM" => ("BETA-LACTAM", "J01C/D"),
        "COLISTIN" => ("POLYMYXIN", "J01XB"),
        "EFFLUX" => ("MULTIDRUG", "J01"),
        "FOSFOMYCIN" => ("PHOSPHONIC_ACID", "J01XX01"),
        "FUSIDIC_ACID" => ("FUSIDANE", "J01XC"),
        "GLYCOPEPTIDE" => ("GLYCOPEPTIDE", "J01XA"),
        "LINCOSAMIDE" => ("LINCOSAMIDE", "J01FF"),
        "MACROLIDE" => ("MACROLIDE", "J01FA"),
        "MULTIDRUG" => ("MULTIDRUG", "J01"),
        "NITROIMIDAZOLE" => ("NITROIMIDAZOLE", "J01XD"),
        "NUCLEOSIDE" => ("NUCLEOSIDE", "J01XX"),
        "OXAZOLIDINONE" => ("OXAZOLIDINONE", "J01XX08"),
        "PHENICOL" => ("AMPHENICOL", "J01BA"),
        "PLEUROMUTILIN" => ("PLEUROMUTILIN", "J01XX"),
        "PSEUDOMONIC_ACID" => ("PSEUDOMONIC_ACID", "D06AX09"),
        "QUINOLONE" => ("FLUOROQUINOLONE", "J01MA"),
        "RIFAMYCIN" => ("RIFAMYCIN", "J04AB"),
        "STREPTOGRAMIN" => ("STREPTOGRAMIN", "J01FG"),
        "SULFONAMIDE" => ("SULFONAMIDE", "J01E"),
        "TETRACYCLINE" => ("TETRACYCLINE", "J01AA"),
        "THIOSTREPTON" => ("THIOSTREPTON", "J01XX"),
        "TRIMETHOPRIM" => ("TRIMETHOPRIM", "J01EA"),
        "TUBERACTINOMYCIN" => ("TUBERACTINOMYCIN", "J04AB"),
        _ => ("OTHER", "UNKNOWN"),
    }
}

/// Maps CARD drug class to NDARO-style class.
fn card_to_ndaro_class(card_class: &str) -> &'static str {
    let card_lower = card_class.to_lowercase();

    if card_lower.contains("aminoglycoside") { return "AMINOGLYCOSIDE"; }
    if card_lower.contains("beta-lactam") || card_lower.contains("cephalosporin")
       || card_lower.contains("carbapenem") || card_lower.contains("penam") { return "BETA-LACTAM"; }
    if card_lower.contains("colistin") || card_lower.contains("polymyxin") { return "COLISTIN"; }
    if card_lower.contains("fosfomycin") { return "FOSFOMYCIN"; }
    if card_lower.contains("fusidic") { return "FUSIDIC_ACID"; }
    if card_lower.contains("glycopeptide") || card_lower.contains("vancomycin") { return "GLYCOPEPTIDE"; }
    if card_lower.contains("lincosamide") { return "LINCOSAMIDE"; }
    if card_lower.contains("macrolide") { return "MACROLIDE"; }
    if card_lower.contains("nitroimidazole") { return "NITROIMIDAZOLE"; }
    if card_lower.contains("nucleoside") { return "NUCLEOSIDE"; }
    if card_lower.contains("oxazolidinone") { return "OXAZOLIDINONE"; }
    if card_lower.contains("phenicol") || card_lower.contains("chloramphenicol") { return "PHENICOL"; }
    if card_lower.contains("pleuromutilin") { return "PLEUROMUTILIN"; }
    if card_lower.contains("quinolone") || card_lower.contains("fluoroquinolone") { return "QUINOLONE"; }
    if card_lower.contains("rifamycin") || card_lower.contains("rifampin") { return "RIFAMYCIN"; }
    if card_lower.contains("streptogramin") { return "STREPTOGRAMIN"; }
    if card_lower.contains("sulfonamide") { return "SULFONAMIDE"; }
    if card_lower.contains("tetracycline") { return "TETRACYCLINE"; }
    if card_lower.contains("trimethoprim") { return "TRIMETHOPRIM"; }
    if card_lower.contains("mupirocin") || card_lower.contains("pseudomonic") { return "PSEUDOMONIC_ACID"; }

    "MULTIDRUG"
}

// ============================================================================
// File Download
// ============================================================================

/// Fetches a file from a URL to the specified output path.
///
/// Uses ureq with a 5-minute timeout for large files.
fn fetch_file(url: &str, output_path: &Path) -> Result<()> {
    eprintln!("  Downloading {}...", output_path.file_name().unwrap().to_string_lossy());

    let response = ureq::get(url)
        .timeout(Duration::from_secs(300))
        .call()
        .with_context(|| format!("Failed to download {}", url))?;

    let mut file = File::create(output_path)?;
    let mut reader = response.into_reader();
    std::io::copy(&mut reader, &mut file)?;

    Ok(())
}

/// Fetches and extracts CARD database archive.
///
/// CARD provides data as tar.bz2 containing:
/// - nucleotide_fasta_protein_homolog_model.fasta
/// - aro_index.tsv (metadata)
fn fetch_card_data(output_dir: &Path) -> Result<()> {
    use bzip2::read::BzDecoder;
    use tar::Archive;

    let archive_path = output_dir.join("card-data.tar.bz2");
    let extract_dir = output_dir.join("card_raw");

    eprintln!("  Downloading CARD data archive (~5MB compressed)...");
    fetch_file(CARD_DATA_URL, &archive_path)?;

    eprintln!("  Extracting CARD archive...");
    std::fs::create_dir_all(&extract_dir)?;

    let archive_file = File::open(&archive_path)?;
    let decoder = BzDecoder::new(archive_file);
    let mut archive = Archive::new(decoder);
    archive.unpack(&extract_dir)?;

    eprintln!("  CARD extraction complete.");
    Ok(())
}

// ============================================================================
// NCBI Parsing
// ============================================================================

/// Parses NCBI ReferenceGeneCatalog.txt for ARG entries.
///
/// Filters for type='AMR' and excludes susceptible reference sequences.
/// Returns entries keyed by protein accession (or allele for nucleotide-only entries).
fn parse_reference_catalogue(catalogue_path: &Path) -> Result<FxHashMap<String, CatalogueEntry>> {
    eprintln!("[2] Parsing ReferenceGeneCatalog.txt...");

    let file = File::open(catalogue_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Parse header to find column indices
    let header = lines.next().ok_or_else(|| anyhow::anyhow!("Empty catalogue file"))??;
    let columns: Vec<&str> = header.split('\t').collect();

    let find_col = |name: &str| -> Result<usize> {
        columns.iter().position(|&c| c == name)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", name))
    };

    let idx_allele = find_col("allele")?;
    let idx_gene_family = find_col("gene_family")?;
    let idx_type = find_col("type")?;
    let idx_subtype = find_col("subtype")?;
    let idx_class = find_col("class")?;
    let idx_refseq_prot = find_col("refseq_protein_accession")?;
    let idx_genbank_prot = find_col("genbank_protein_accession")?;
    let idx_refseq_nuc = find_col("refseq_nucleotide_accession")?;
    let idx_genbank_nuc = find_col("genbank_nucleotide_accession")?;
    let idx_genbank_strand = find_col("genbank_strand")?;
    let idx_genbank_start = find_col("genbank_start")?;
    let idx_genbank_stop = find_col("genbank_stop")?;

    let all_indices = [idx_allele, idx_gene_family, idx_type, idx_subtype, idx_class,
                       idx_refseq_prot, idx_genbank_prot, idx_refseq_nuc, idx_genbank_nuc,
                       idx_genbank_strand, idx_genbank_start, idx_genbank_stop];
    let min_columns = all_indices.into_iter().max().unwrap_or(0);

    let mut entries: FxHashMap<String, CatalogueEntry> = FxHashMap::default();
    let mut nuc_only_count = 0usize;

    for line in lines {
        let line = line?;
        let fields: Vec<&str> = line.split('\t').collect();

        if fields.len() <= min_columns {
            continue;
        }

        // Filter for type='AMR' only
        if fields[idx_type] != "AMR" {
            continue;
        }

        // Skip AMR-SUSCEPTIBLE (susceptible reference sequences)
        if fields[idx_subtype] == "AMR-SUSCEPTIBLE" {
            continue;
        }

        let refseq_prot = fields[idx_refseq_prot];
        let genbank_prot = fields[idx_genbank_prot];
        let refseq_nuc = fields[idx_refseq_nuc];
        let genbank_nuc = fields[idx_genbank_nuc];
        let genbank_strand = fields[idx_genbank_strand];
        let genbank_start = fields[idx_genbank_start].parse::<u64>().ok();
        let genbank_stop = fields[idx_genbank_stop].parse::<u64>().ok();

        let allele = fields[idx_allele].to_string();
        let gene_family = fields[idx_gene_family].to_string();
        let ndaro_class = fields[idx_class].to_string();
        let subtype = fields[idx_subtype].to_string();

        // Determine nucleotide accession (prefer RefSeq, fallback to GenBank)
        let nuc_acc = if !refseq_nuc.is_empty() {
            Some(refseq_nuc.to_string())
        } else if !genbank_nuc.is_empty() {
            Some(genbank_nuc.to_string())
        } else {
            None
        };

        // Create entry with nucleotide info
        let create_entry = |prot_acc: &str| CatalogueEntry {
            protein_accession: prot_acc.to_string(),
            allele: allele.clone(),
            gene_family: gene_family.clone(),
            ndaro_class: ndaro_class.clone(),
            subtype: subtype.clone(),
            nucleotide_accession: nuc_acc.clone(),
            nuc_start: genbank_start,
            nuc_stop: genbank_stop,
            nuc_strand: if !genbank_strand.is_empty() { Some(genbank_strand.to_string()) } else { None },
        };

        // Handle entries with protein accession
        // Key by allele (or gene_family if allele is empty) to preserve all POINT mutations
        // that share the same protein accession
        if !refseq_prot.is_empty() || !genbank_prot.is_empty() {
            let key = if !allele.is_empty() { allele.clone() } else { gene_family.clone() };
            let prot_acc = if !refseq_prot.is_empty() { refseq_prot } else { genbank_prot };
            if !entries.contains_key(&key) {
                entries.insert(key, create_entry(prot_acc));
            }
        }
        // Handle nucleotide-only entries (e.g., 16S rRNA mutations) - key by allele
        else if nuc_acc.is_some() && genbank_start.is_some() && genbank_stop.is_some() && !allele.is_empty() {
            let key = format!("NUC:{}", allele);
            if !entries.contains_key(&key) {
                entries.insert(key, CatalogueEntry {
                    protein_accession: String::new(),
                    allele: allele.clone(),
                    gene_family: gene_family.clone(),
                    ndaro_class: ndaro_class.clone(),
                    subtype: subtype.clone(),
                    nucleotide_accession: nuc_acc,
                    nuc_start: genbank_start,
                    nuc_stop: genbank_stop,
                    nuc_strand: if !genbank_strand.is_empty() { Some(genbank_strand.to_string()) } else { None },
                });
                nuc_only_count += 1;
            }
        }
    }

    let prot_count = entries.len() - nuc_only_count;
    eprintln!("    Found {} gene entries + {} nucleotide-only entries", prot_count, nuc_only_count);
    Ok(entries)
}

/// Parses AMR_CDS.fa file to extract gene sequences.
///
/// Maps protein accession to their nucleotide sequences for reliable matching.
fn parse_arg_sequences(fasta_path: &Path) -> Result<FxHashMap<String, String>> {
    eprintln!("[3] Parsing AMR_CDS.fa...");

    let file = File::open(fasta_path)?;
    let reader = BufReader::new(file);

    let mut sequences: FxHashMap<String, String> = FxHashMap::default();
    let mut current_prot_acc: Option<String> = None;
    let mut current_seq = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('>') {
            // Save previous sequence
            if let Some(prot_acc) = current_prot_acc.take() {
                if !current_seq.is_empty() {
                    sequences.insert(prot_acc, std::mem::take(&mut current_seq));
                }
            }

            // Parse protein accession from header (first field)
            // Format: >PROT_ACC|NUC_ACC|...|GENE_NAME|GENE_NAME|description
            let header = line.trim_start_matches('>');
            let parts: Vec<&str> = header.split('|').collect();
            if let Some(first) = parts.first() {
                current_prot_acc = Some(first.to_string());
            }
            current_seq.clear();
        } else {
            current_seq.push_str(line.trim());
        }
    }

    // Save last sequence
    if let Some(prot_acc) = current_prot_acc {
        if !current_seq.is_empty() {
            sequences.insert(prot_acc, current_seq);
        }
    }

    eprintln!("    Loaded {} sequences from AMR_CDS.fa", sequences.len());
    Ok(sequences)
}

/// Fetches missing sequences from NCBI using E-utilities API.
///
/// Strategy:
/// 1. For protein accessions (WP_, NP_, etc.): Try db=protein with rettype=fasta_cds_na
/// 2. For WP_ accessions that fail: Use IPG (Identical Protein Groups) to find linked CDS
/// 3. For nucleotide accessions (NG_, NC_, etc.): Use db=nucleotide with rettype=fasta
///
/// Uses batch requests (max 200 IDs per request) to efficiently download.
fn fetch_missing_cds(accessions: &[String]) -> Result<FxHashMap<String, String>> {
    if accessions.is_empty() {
        return Ok(FxHashMap::default());
    }

    // Separate protein vs nucleotide accessions
    let protein_prefixes = ["WP_", "NP_", "YP_", "XP_", "AAA", "AAB", "AAC", "AAD", "AAE",
                           "AAF", "AAG", "AAH", "AAI", "AAK", "AAL", "AAM", "AAN", "AAO",
                           "CAA", "BAA", "EAA", "P", "Q"];

    let (protein_accs, nucleotide_accs): (Vec<_>, Vec<_>) = accessions.iter()
        .cloned()
        .partition(|acc| {
            protein_prefixes.iter().any(|p| acc.starts_with(p))
        });

    eprintln!("    Downloading missing sequences from NCBI...");
    eprintln!("      Protein accessions: {}", protein_accs.len());
    eprintln!("      Nucleotide accessions: {}", nucleotide_accs.len());

    let mut sequences: FxHashMap<String, String> = FxHashMap::default();
    let batch_size = 200;

    // Fetch protein CDS sequences (standard method)
    if !protein_accs.is_empty() {
        for (batch_idx, chunk) in protein_accs.chunks(batch_size).enumerate() {
            let ids = chunk.join(",");
            let url = format!(
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=protein&id={}&rettype=fasta_cds_na&retmode=text",
                ids
            );

            if batch_idx > 0 {
                std::thread::sleep(Duration::from_millis(350));
            }

            if let Ok(resp) = ureq::get(&url).timeout(Duration::from_secs(120)).call() {
                if let Ok(body) = resp.into_string() {
                    parse_fasta_response(&body, chunk, &mut sequences);
                }
            }
        }

        // For WP_ accessions that failed, try IPG method
        let wp_failed: Vec<_> = protein_accs.iter()
            .filter(|acc| acc.starts_with("WP_") && !sequences.contains_key(*acc))
            .cloned()
            .collect();

        if !wp_failed.is_empty() {
            eprintln!("      Fetching {} WP_ accessions via IPG...", wp_failed.len());
            fetch_wp_via_ipg(&wp_failed, &mut sequences);
        }
    }

    // Fetch nucleotide sequences (NG_, NC_, CP, etc.)
    if !nucleotide_accs.is_empty() {
        eprintln!("      Fetching nucleotide sequences...");
        for (batch_idx, chunk) in nucleotide_accs.chunks(batch_size).enumerate() {
            let ids = chunk.join(",");
            let url = format!(
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nucleotide&id={}&rettype=fasta&retmode=text",
                ids
            );

            if batch_idx > 0 {
                std::thread::sleep(Duration::from_millis(350));
            }

            if let Ok(resp) = ureq::get(&url).timeout(Duration::from_secs(120)).call() {
                if let Ok(body) = resp.into_string() {
                    parse_fasta_response(&body, chunk, &mut sequences);
                }
            }

            if (batch_idx + 1) % 5 == 0 {
                eprintln!("        Downloaded {}/{} nucleotide accessions...",
                         (batch_idx + 1) * batch_size.min(chunk.len()),
                         nucleotide_accs.len());
            }
        }
    }

    eprintln!("    Downloaded {} sequences total", sequences.len());
    Ok(sequences)
}

/// Fetches CDS for WP_ accessions using IPG (Identical Protein Groups).
///
/// WP_ accessions are non-redundant proteins without direct CDS links.
/// IPG provides linked nucleotide accessions with coordinates.
///
/// Two-phase approach:
/// 1. Batch fetch IPG data using epost+efetch (much faster than individual queries)
/// 2. Fetch CDS sequences using coordinates (sequential due to coordinate-based queries)
fn fetch_wp_via_ipg(wp_accs: &[String], sequences: &mut FxHashMap<String, String>) {
    if wp_accs.is_empty() {
        return;
    }

    // Phase 1: Batch fetch IPG data using epost + efetch
    eprintln!("        Fetching IPG data for {} WP_ accessions...", wp_accs.len());

    // Build lookup set for filtering
    let wp_set: FxHashSet<&str> = wp_accs.iter().map(|s| s.as_str()).collect();
    let mut coord_map: FxHashMap<String, (String, usize, usize, char)> = FxHashMap::default();
    let batch_size = 50; // Smaller batches to avoid huge IPG responses

    for (batch_idx, chunk) in wp_accs.chunks(batch_size).enumerate() {
        if batch_idx > 0 {
            std::thread::sleep(Duration::from_millis(500));
        }

        // Step 1: epost to submit IDs
        let ids = chunk.join(",");
        let epost_url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/epost.fcgi?db=protein&id={}",
            ids
        );

        let epost_resp = match ureq::get(&epost_url).timeout(Duration::from_secs(60)).call() {
            Ok(r) => r,
            Err(_) => continue,
        };

        let epost_body = match epost_resp.into_string() {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Parse WebEnv and QueryKey from XML response
        let webenv = epost_body
            .split("<WebEnv>").nth(1)
            .and_then(|s| s.split("</WebEnv>").next())
            .map(|s| s.to_string());
        let query_key = epost_body
            .split("<QueryKey>").nth(1)
            .and_then(|s| s.split("</QueryKey>").next())
            .map(|s| s.to_string());

        let (webenv, query_key) = match (webenv, query_key) {
            (Some(w), Some(q)) => (w, q),
            _ => continue,
        };

        std::thread::sleep(Duration::from_millis(350));

        // Step 2: efetch with IPG format
        let efetch_url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=protein&query_key={}&WebEnv={}&rettype=ipg&retmode=text",
            query_key, webenv
        );

        let efetch_resp = match ureq::get(&efetch_url).timeout(Duration::from_secs(120)).call() {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Use reader to handle large responses
        let reader = BufReader::new(efetch_resp.into_reader());
        let mut line_count = 0;

        // Parse IPG data - Format: Id\tSource\tNucleotide Accession\tStart\tStop\tStrand\tProtein\t...
        // Column 6 (index 6) is the Protein accession we need to match
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };

            line_count += 1;

            // Skip header line
            if line_count == 1 {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 7 {
                let source = fields[1];
                let nuc_acc = fields[2];
                let start: Option<usize> = fields[3].parse().ok();
                let stop: Option<usize> = fields[4].parse().ok();
                let strand_char = fields[5].chars().next();
                let prot_acc = fields[6];

                // Only process if this protein accession is in our requested list
                if !wp_set.contains(prot_acc) {
                    continue;
                }

                // Skip if already found
                if coord_map.contains_key(prot_acc) {
                    continue;
                }

                if let (Some(s), Some(e), Some(c)) = (start, stop, strand_char) {
                    // Prefer RefSeq (NC_, NZ_)
                    if source == "RefSeq" && (nuc_acc.starts_with("NC_") || nuc_acc.starts_with("NZ_")) {
                        coord_map.insert(prot_acc.to_string(), (nuc_acc.to_string(), s, e, c));
                    }
                    // Use INSDC as fallback if no RefSeq yet
                    else if source == "INSDC" {
                        coord_map.insert(prot_acc.to_string(), (nuc_acc.to_string(), s, e, c));
                    }
                }
            }
        }

        let total_batches = wp_accs.chunks(batch_size).count();
        eprintln!("        IPG batch {}/{}: found {} coordinates so far",
                 batch_idx + 1, total_batches, coord_map.len());
    }

    eprintln!("        Found coordinates for {}/{} WP_ accessions", coord_map.len(), wp_accs.len());

    // Fallback for failed accessions: individual IPG queries without filtering
    let failed: Vec<_> = wp_accs.iter().filter(|acc| !coord_map.contains_key(*acc)).cloned().collect();
    if !failed.is_empty() {
        eprintln!("        Retrying {} failed accessions individually...", failed.len());
        for (idx, wp_acc) in failed.iter().enumerate() {
            if idx > 0 {
                std::thread::sleep(Duration::from_millis(400));
            }

            // Individual IPG query - take first valid coordinate
            let ipg_url = format!(
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=protein&id={}&rettype=ipg&retmode=text",
                wp_acc
            );

            if let Ok(resp) = ureq::get(&ipg_url).timeout(Duration::from_secs(30)).call() {
                let reader = BufReader::new(resp.into_reader());
                for (line_idx, line_result) in reader.lines().enumerate() {
                    if line_idx == 0 { continue; } // Skip header

                    if let Ok(line) = line_result {
                        let fields: Vec<&str> = line.split('\t').collect();
                        if fields.len() >= 6 {
                            let source = fields[1];
                            let nuc_acc = fields[2];
                            let start: Option<usize> = fields[3].parse().ok();
                            let stop: Option<usize> = fields[4].parse().ok();
                            let strand_char = fields[5].chars().next();

                            // Take first valid coordinate (prefer RefSeq, then INSDC)
                            if let (Some(s), Some(e), Some(c)) = (start, stop, strand_char) {
                                if source == "RefSeq" && (nuc_acc.starts_with("NC_") || nuc_acc.starts_with("NZ_")) {
                                    coord_map.insert(wp_acc.clone(), (nuc_acc.to_string(), s, e, c));
                                    break;
                                } else if source == "INSDC" && !coord_map.contains_key(wp_acc) {
                                    coord_map.insert(wp_acc.clone(), (nuc_acc.to_string(), s, e, c));
                                    // Don't break yet - keep looking for RefSeq
                                }
                            }
                        }
                    }
                }
            }
        }

        let still_failed: Vec<_> = failed.iter().filter(|acc| !coord_map.contains_key(*acc)).collect();
        if !still_failed.is_empty() {
            eprintln!("        Still failed ({} - likely deprecated):", still_failed.len());
            for acc in &still_failed {
                eprintln!("          - {}", acc);
            }
        } else {
            eprintln!("        All {} retried successfully", failed.len());
        }
    }

    eprintln!("        Final coordinates: {}/{} WP_ accessions", coord_map.len(), wp_accs.len());

    // Phase 2: Fetch CDS sequences using coordinates
    let coord_list: Vec<_> = wp_accs.iter()
        .filter_map(|wp| coord_map.get(wp).map(|(n, s, e, c)| (wp.clone(), n.clone(), *s, *e, *c)))
        .collect();

    for (idx, (wp_acc, nuc_acc, start, stop, strand)) in coord_list.iter().enumerate() {
        if idx > 0 {
            std::thread::sleep(Duration::from_millis(340));
        }

        let strand_param = if *strand == '-' { "2" } else { "1" };
        let fetch_url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id={}&rettype=fasta&seq_start={}&seq_stop={}&strand={}",
            nuc_acc, start, stop, strand_param
        );

        if let Ok(resp) = ureq::get(&fetch_url).timeout(Duration::from_secs(30)).call() {
            if let Ok(body) = resp.into_string() {
                let mut seq = String::new();
                for line in body.lines() {
                    if !line.starts_with('>') {
                        seq.push_str(line.trim());
                    }
                }
                if !seq.is_empty() {
                    sequences.insert(wp_acc.clone(), seq);
                }
            }
        }

        if (idx + 1) % 100 == 0 {
            eprintln!("        CDS fetch: {}/{}", idx + 1, coord_list.len());
        }
    }
}

/// Fetches nucleotide sequences by coordinates for nucleotide-only entries.
///
/// For entries like 16S rRNA mutations that have no protein accession but have
/// nucleotide accession with start/stop coordinates.
fn fetch_by_coordinates(entries: &[(String, String, u64, u64, String)], sequences: &mut FxHashMap<String, String>) {
    if entries.is_empty() {
        return;
    }

    eprintln!("      Fetching {} nucleotide-only sequences by coordinates...", entries.len());

    for (idx, (key, nuc_acc, start, stop, strand)) in entries.iter().enumerate() {
        // Rate limiting
        if idx > 0 {
            std::thread::sleep(Duration::from_millis(350));
        }

        let strand_param = if strand == "-" { "2" } else { "1" };
        let fetch_url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=nuccore&id={}&rettype=fasta&seq_start={}&seq_stop={}&strand={}",
            nuc_acc, start, stop, strand_param
        );

        if let Ok(resp) = ureq::get(&fetch_url).timeout(Duration::from_secs(30)).call() {
            if let Ok(body) = resp.into_string() {
                // Parse FASTA
                let mut seq = String::new();
                for line in body.lines() {
                    if !line.starts_with('>') {
                        seq.push_str(line.trim());
                    }
                }
                if !seq.is_empty() {
                    sequences.insert(key.clone(), seq);
                }
            }
        }

        // Progress update
        if (idx + 1) % 50 == 0 {
            eprintln!("        Coordinate fetch progress: {}/{}", idx + 1, entries.len());
        }
    }
}

/// Parses FASTA response and maps sequences to known accessions.
fn parse_fasta_response(body: &str, known_accs: &[String], sequences: &mut FxHashMap<String, String>) {
    let mut current_acc: Option<String> = None;
    let mut current_seq = String::new();

    for line in body.lines() {
        if line.starts_with('>') {
            // Save previous sequence
            if let Some(acc) = current_acc.take() {
                if !current_seq.is_empty() {
                    sequences.insert(acc, std::mem::take(&mut current_seq));
                }
            }

            let header = line.trim_start_matches('>');

            // Try to match known accessions in header
            for acc in known_accs {
                if header.contains(acc.as_str()) || header.starts_with(acc.as_str()) {
                    current_acc = Some(acc.clone());
                    break;
                }
            }
            current_seq.clear();
        } else {
            current_seq.push_str(line.trim());
        }
    }

    // Save last sequence
    if let Some(acc) = current_acc {
        if !current_seq.is_empty() {
            sequences.insert(acc, current_seq);
        }
    }
}

/// Builds protein accession → gene name mapping from NCBI catalogue.
///
/// Returns (protein_mapping, gene_name_set) for CARD gene name resolution.
fn build_protein_accession_map(catalogue_path: &Path) -> Result<(FxHashMap<String, String>, FxHashSet<String>)> {
    eprintln!("    Building protein accession → gene name mapping...");

    let file = File::open(catalogue_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = lines.next().ok_or_else(|| anyhow::anyhow!("Empty catalogue file"))??;
    let columns: Vec<&str> = header.split('\t').collect();

    let find_col = |name: &str| -> Result<usize> {
        columns.iter().position(|&c| c == name)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", name))
    };

    let idx_allele = find_col("allele")?;
    let idx_gene_family = find_col("gene_family")?;
    let idx_refseq_prot = find_col("refseq_protein_accession")?;
    let idx_genbank_prot = find_col("genbank_protein_accession")?;
    let idx_type = find_col("type")?;
    let idx_subtype = find_col("subtype")?;

    let mut mapping: FxHashMap<String, String> = FxHashMap::default();
    let mut gene_names: FxHashSet<String> = FxHashSet::default();

    for line in lines {
        let line = line?;
        let fields: Vec<&str> = line.split('\t').collect();

        if fields.len() <= idx_genbank_prot {
            continue;
        }

        if fields[idx_type] != "AMR" || fields[idx_subtype] == "AMR-SUSCEPTIBLE" {
            continue;
        }

        let gene_name = if !fields[idx_allele].is_empty() {
            fields[idx_allele]
        } else {
            fields[idx_gene_family]
        };

        gene_names.insert(gene_name.to_string());

        if !fields[idx_refseq_prot].is_empty() {
            mapping.insert(fields[idx_refseq_prot].to_string(), gene_name.to_string());
        }
        if !fields[idx_genbank_prot].is_empty() {
            mapping.insert(fields[idx_genbank_prot].to_string(), gene_name.to_string());
        }
    }

    eprintln!("    Built mapping for {} protein accessions ({} unique genes)",
             mapping.len(), gene_names.len());
    Ok((mapping, gene_names))
}

// ============================================================================
// CARD Parsing
// ============================================================================

/// Parses CARD database files and maps to NCBI gene names.
///
/// Attempts to resolve CARD gene names to NCBI names via:
/// 1. Protein accession mapping (most reliable)
/// 2. Case-insensitive name matching
/// 3. "bla" prefix matching for beta-lactamases
fn parse_card_entries(
    card_dir: &Path,
    ncbi_prot_mapping: &FxHashMap<String, String>,
    ncbi_name_set: &FxHashSet<String>,
) -> Result<Vec<CardGeneEntry>> {
    eprintln!("[3] Parsing CARD database...");

    // Build case-insensitive name lookup
    let ncbi_name_lower: FxHashMap<String, String> = ncbi_name_set
        .iter()
        .map(|n| (n.to_lowercase(), n.clone()))
        .collect();

    // Parse ARO index for metadata
    let aro_path = card_dir.join("aro_index.tsv");
    let mut aro_metadata: FxHashMap<String, (String, String)> = FxHashMap::default();

    if aro_path.exists() {
        let file = File::open(&aro_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        lines.next(); // Skip header

        for line in lines {
            let line = line?;
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 10 {
                let aro_acc = fields[0].to_string();
                let prot_acc = fields[6].to_string();
                let drug_class = fields[9].to_string();
                aro_metadata.insert(aro_acc, (drug_class, prot_acc));
            }
        }
        eprintln!("    Loaded {} ARO metadata entries", aro_metadata.len());
    }

    // Parse CARD FASTA (protein homolog model)
    let fasta_path = card_dir.join("nucleotide_fasta_protein_homolog_model.fasta");
    let mut entries: Vec<CardGeneEntry> = Vec::new();

    if fasta_path.exists() {
        let file = File::open(&fasta_path)?;
        let reader = BufReader::new(file);

        let mut current_entry: Option<CardGeneEntry> = None;
        let mut current_seq = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('>') {
                // Save previous entry
                if let Some(mut entry) = current_entry.take() {
                    entry.sequence = std::mem::take(&mut current_seq);
                    if !entry.sequence.is_empty() {
                        entries.push(entry);
                    }
                }

                // Parse CARD header
                let header = line.trim_start_matches('>');
                let parts: Vec<&str> = header.split('|').collect();

                if parts.len() >= 6 {
                    let aro_acc = parts[4].to_string();
                    let gene_part = parts[5];

                    let card_name = if let Some(idx) = gene_part.find('[') {
                        gene_part[..idx].trim().to_string()
                    } else {
                        gene_part.trim().to_string()
                    };

                    let (drug_class, prot_acc) = aro_metadata
                        .get(&aro_acc)
                        .cloned()
                        .unwrap_or_else(|| ("UNKNOWN".to_string(), String::new()));

                    // Try to map to NCBI gene name
                    let ncbi_name = ncbi_prot_mapping.get(&prot_acc).cloned()
                        .or_else(|| {
                            let bla_name = format!("bla{}", card_name);
                            if ncbi_name_set.contains(&bla_name) {
                                return Some(bla_name);
                            }
                            let lower = card_name.to_lowercase();
                            if let Some(ncbi) = ncbi_name_lower.get(&lower) {
                                return Some(ncbi.clone());
                            }
                            let bla_lower = format!("bla{}", lower);
                            ncbi_name_lower.get(&bla_lower).cloned()
                        });

                    current_entry = Some(CardGeneEntry {
                        card_name,
                        ncbi_name,
                        sequence: String::new(),
                        drug_class,
                    });
                }
                current_seq.clear();
            } else {
                current_seq.push_str(line.trim());
            }
        }

        if let Some(mut entry) = current_entry {
            entry.sequence = current_seq;
            if !entry.sequence.is_empty() {
                entries.push(entry);
            }
        }
    }

    let mapped_count = entries.iter().filter(|e| e.ncbi_name.is_some()).count();
    eprintln!("    Loaded {} CARD sequences", entries.len());
    eprintln!("    NCBI mapping: {}/{} ({:.1}%)",
             mapped_count, entries.len(),
             100.0 * mapped_count as f64 / entries.len().max(1) as f64);

    Ok(entries)
}

// ============================================================================
// Database Building
// ============================================================================

/// Builds gene database from CARD entries.
fn build_from_card(card_entries: &[CardGeneEntry]) -> Result<(FxHashMap<String, String>, ArgMetaDb)> {
    eprintln!("[4] Building gene database from CARD...");

    let mut sequences: FxHashMap<String, String> = FxHashMap::default();
    let mut meta_genes: FxHashMap<String, GeneMeta> = FxHashMap::default();
    let mut seen_genes: FxHashSet<String> = FxHashSet::default();
    let mut card_only_count = 0;

    for entry in card_entries {
        let (gene_id, is_card_only) = if let Some(ncbi_name) = &entry.ncbi_name {
            (ncbi_name.clone(), false)
        } else {
            (entry.card_name.to_lowercase(), true)
        };

        if seen_genes.contains(&gene_id) {
            continue;
        }
        seen_genes.insert(gene_id.clone());

        if is_card_only {
            card_only_count += 1;
        }

        let ndaro_class = card_to_ndaro_class(&entry.drug_class);
        let (chemical_class, atc_code) = get_chemical_and_atc(ndaro_class);

        let gene_family = if is_card_only {
            format!("CARD:{}", entry.card_name)
        } else {
            entry.card_name.clone()
        };

        let meta = GeneMeta {
            gene_family,
            drug_class: ndaro_class.to_string(),
            chemical_class: chemical_class.to_string(),
            atc_code: atc_code.to_string(),
        };

        sequences.insert(gene_id.clone(), entry.sequence.clone());
        meta_genes.insert(gene_id, meta);
    }

    let meta_db = ArgMetaDb {
        gene_count: meta_genes.len(),
        total_taxa_entries: meta_genes.len(),
        genes: meta_genes,
    };

    eprintln!("    Built {} unique genes ({} CARD-only)", meta_db.gene_count, card_only_count);
    Ok((sequences, meta_db))
}

// ============================================================================
// PanRes Parsing
// ============================================================================

/// PanRes gene entry with metadata.
#[derive(Clone, Debug)]
struct PanResGeneEntry {
    pan_id: String,
    gene_name: String,
    gene_family: String,
    drug_class: String,
    sequence: String,
}

/// Parses PanRes database files.
///
/// PanRes is a combined ARG database from ARGprofiler project.
/// It merges multiple sources (NCBI, CARD, ResFinder, etc.) into one unified database.
fn parse_panres_entries(panres_dir: &Path) -> Result<Vec<PanResGeneEntry>> {
    eprintln!("[3] Parsing PanRes database...");

    let fasta_path = panres_dir.join("panres_genes.fa");
    let tsv_path = panres_dir.join("panres_data.tsv");

    // Parse metadata TSV first
    let mut metadata: FxHashMap<String, (String, String, String)> = FxHashMap::default();

    if tsv_path.exists() {
        let file = File::open(&tsv_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.starts_with("userGeneName") {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 5 {
                let pan_id = fields[0].to_string();
                let fa_header = if fields.len() > 4 { fields[4] } else { "" };

                // Extract gene name from fa_header (format: ...|gene_name|gene_family|...)
                let parts: Vec<&str> = fa_header.split('|').collect();
                let (gene_name, gene_family) = if parts.len() >= 7 {
                    (parts[5].to_string(), parts[6].to_string())
                } else {
                    (pan_id.clone(), pan_id.clone())
                };

                // Infer drug class from gene family or description
                let drug_class = infer_drug_class_from_gene(&gene_name, &gene_family);

                metadata.insert(pan_id, (gene_name, gene_family, drug_class));
            }
        }
        eprintln!("    Loaded {} metadata entries", metadata.len());
    }

    // Parse FASTA sequences
    let mut entries: Vec<PanResGeneEntry> = Vec::new();

    if fasta_path.exists() {
        let file = File::open(&fasta_path)?;
        let reader = BufReader::new(file);

        let mut current_id: Option<String> = None;
        let mut current_seq = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('>') {
                // Save previous entry
                if let Some(pan_id) = current_id.take() {
                    if !current_seq.is_empty() {
                        let (gene_name, gene_family, drug_class) = metadata
                            .get(&pan_id)
                            .cloned()
                            .unwrap_or_else(|| (pan_id.clone(), pan_id.clone(), "MULTIDRUG".to_string()));

                        entries.push(PanResGeneEntry {
                            pan_id: pan_id.clone(),
                            gene_name,
                            gene_family,
                            drug_class,
                            sequence: std::mem::take(&mut current_seq),
                        });
                    }
                }

                // Parse new header
                let header = line.trim_start_matches('>');
                current_id = Some(header.split_whitespace().next().unwrap_or(header).to_string());
                current_seq.clear();
            } else {
                current_seq.push_str(line.trim());
            }
        }

        // Save last entry
        if let Some(pan_id) = current_id {
            if !current_seq.is_empty() {
                let (gene_name, gene_family, drug_class) = metadata
                    .get(&pan_id)
                    .cloned()
                    .unwrap_or_else(|| (pan_id.clone(), pan_id.clone(), "MULTIDRUG".to_string()));

                entries.push(PanResGeneEntry {
                    pan_id,
                    gene_name,
                    gene_family,
                    drug_class,
                    sequence: current_seq,
                });
            }
        }
    }

    eprintln!("    Loaded {} PanRes sequences", entries.len());
    Ok(entries)
}

/// Infers drug class from gene name or family.
fn infer_drug_class_from_gene(gene_name: &str, gene_family: &str) -> String {
    let name_lower = gene_name.to_lowercase();
    let family_lower = gene_family.to_lowercase();

    // Beta-lactamases
    if name_lower.starts_with("bla") || family_lower.contains("beta-lactam")
        || name_lower.contains("oxa") || name_lower.contains("ctx")
        || name_lower.contains("tem") || name_lower.contains("shv")
        || name_lower.contains("kpc") || name_lower.contains("ndm")
        || name_lower.contains("vim") || name_lower.contains("imp") {
        return "BETA-LACTAM".to_string();
    }

    // Aminoglycosides
    if name_lower.starts_with("aac") || name_lower.starts_with("aph")
        || name_lower.starts_with("ant") || name_lower.starts_with("aad")
        || family_lower.contains("aminoglycoside") {
        return "AMINOGLYCOSIDE".to_string();
    }

    // Tetracyclines
    if name_lower.starts_with("tet") || family_lower.contains("tetracycline") {
        return "TETRACYCLINE".to_string();
    }

    // Macrolides
    if name_lower.starts_with("erm") || name_lower.starts_with("mef")
        || name_lower.starts_with("mph") || family_lower.contains("macrolide") {
        return "MACROLIDE".to_string();
    }

    // Quinolones
    if name_lower.starts_with("qnr") || name_lower.contains("gyr")
        || family_lower.contains("quinolone") {
        return "QUINOLONE".to_string();
    }

    // Sulfonamides
    if name_lower.starts_with("sul") || family_lower.contains("sulfonamide") {
        return "SULFONAMIDE".to_string();
    }

    // Trimethoprim
    if name_lower.starts_with("dfr") || family_lower.contains("trimethoprim") {
        return "TRIMETHOPRIM".to_string();
    }

    // Phenicols
    if name_lower.starts_with("cat") || name_lower.starts_with("cml")
        || name_lower.starts_with("flor") || family_lower.contains("phenicol") {
        return "PHENICOL".to_string();
    }

    // Glycopeptides
    if name_lower.starts_with("van") || family_lower.contains("glycopeptide") {
        return "GLYCOPEPTIDE".to_string();
    }

    // Colistin/Polymyxin
    if name_lower.starts_with("mcr") || family_lower.contains("colistin")
        || family_lower.contains("polymyxin") {
        return "COLISTIN".to_string();
    }

    // Rifamycin
    if name_lower.starts_with("arr") || name_lower.contains("rpo")
        || family_lower.contains("rifamycin") {
        return "RIFAMYCIN".to_string();
    }

    // Fosfomycin
    if name_lower.starts_with("fos") || family_lower.contains("fosfomycin") {
        return "FOSFOMYCIN".to_string();
    }

    "MULTIDRUG".to_string()
}

/// Builds gene database from PanRes entries.
fn build_from_panres(panres_entries: &[PanResGeneEntry]) -> Result<(FxHashMap<String, String>, ArgMetaDb)> {
    eprintln!("[4] Building gene database from PanRes...");

    let mut sequences: FxHashMap<String, String> = FxHashMap::default();
    let mut meta_genes: FxHashMap<String, GeneMeta> = FxHashMap::default();

    for entry in panres_entries {
        // Use pan_id as gene_id to keep all sequences (PanRes already deduplicated)
        let gene_id = entry.pan_id.clone();

        let (chemical_class, atc_code) = get_chemical_and_atc(&entry.drug_class);

        let meta = GeneMeta {
            gene_family: entry.gene_family.clone(),
            drug_class: entry.drug_class.clone(),
            chemical_class: chemical_class.to_string(),
            atc_code: atc_code.to_string(),
        };

        sequences.insert(gene_id.clone(), entry.sequence.clone());
        meta_genes.insert(gene_id, meta);
    }

    let meta_db = ArgMetaDb {
        gene_count: meta_genes.len(),
        total_taxa_entries: meta_genes.len(),
        genes: meta_genes,
    };

    eprintln!("    Built {} unique genes", meta_db.gene_count);
    Ok((sequences, meta_db))
}

/// Fetches PanRes database files from Zenodo.
fn fetch_panres_data(output_dir: &Path) -> Result<()> {
    let panres_dir = output_dir.join("panres_raw");
    std::fs::create_dir_all(&panres_dir)?;

    eprintln!("  Downloading PanRes database from Zenodo (~13MB)...");
    fetch_file(PANRES_FASTA_URL, &panres_dir.join("panres_genes.fa"))?;
    fetch_file(PANRES_TSV_URL, &panres_dir.join("panres_data.tsv"))?;

    eprintln!("  PanRes download complete.");
    Ok(())
}

/// Builds gene database from NCBI catalogue entries.
///
/// Uses protein accession as the key for matching catalogue entries to sequences.
/// Both acquired ARGs and POINT mutations are processed uniformly - sequences are
/// fetched by their accessions. POINT mutations use their own sequences directly;
/// SNP verification happens later during genus classification (snp.rs).
/// Downloads missing sequences from NCBI E-utilities when not found in AMR_CDS.fa.
fn build_from_ncbi(
    catalogue: &FxHashMap<String, CatalogueEntry>,
    cds_sequences: &FxHashMap<String, String>,
) -> Result<(FxHashMap<String, String>, ArgMetaDb)> {
    eprintln!("[4] Building gene database from NCBI...");

    // Separate nucleotide-only entries (keyed by "NUC:allele") from protein-based entries
    let mut gene_entries: FxHashMap<String, Vec<&CatalogueEntry>> = FxHashMap::default();
    let mut nuc_only_entries: Vec<(String, &CatalogueEntry)> = Vec::new();
    let mut point_count = 0;

    for (key, entry) in catalogue.iter() {
        if entry.subtype == "POINT" || entry.subtype == "POINT_DISRUPT" {
            point_count += 1;
        }

        // Nucleotide-only entries are keyed by "NUC:allele"
        if key.starts_with("NUC:") {
            nuc_only_entries.push((entry.allele.clone(), entry));
        } else {
            let gene_id = if !entry.allele.is_empty() {
                &entry.allele
            } else {
                &entry.gene_family
            };
            gene_entries.entry(gene_id.clone()).or_default().push(entry);
        }
    }

    eprintln!("    {} unique gene IDs ({} POINT mutations) + {} nucleotide-only entries",
             gene_entries.len(), point_count, nuc_only_entries.len());
    // Find sequences by protein accession
    let mut sequences: FxHashMap<String, String> = FxHashMap::default();
    let mut meta_genes: FxHashMap<String, GeneMeta> = FxHashMap::default();
    let mut missing_accessions: FxHashSet<String> = FxHashSet::default();
    // Map accession to ALL gene_ids that use it (multiple POINT mutations can share same accession)
    let mut acc_to_genes: FxHashMap<String, Vec<String>> = FxHashMap::default();

    for (gene_id, gene_entries_list) in &gene_entries {
        let mut found_sequence: Option<String> = None;

        // Try each protein accession in the entries
        for entry in gene_entries_list.iter() {
            if let Some(seq) = cds_sequences.get(&entry.protein_accession) {
                found_sequence = Some(seq.clone());
                break;
            }
        }

        if let Some(sequence) = found_sequence {
            let first_entry = gene_entries_list.first().unwrap();
            let (chemical_class, atc_code) = get_chemical_and_atc(&first_entry.ndaro_class);

            let meta = GeneMeta {
                gene_family: first_entry.gene_family.clone(),
                drug_class: first_entry.ndaro_class.clone(),
                chemical_class: chemical_class.to_string(),
                atc_code: atc_code.to_string(),
            };

            sequences.insert(gene_id.clone(), sequence);
            meta_genes.insert(gene_id.clone(), meta);
        } else {
            // Mark for download - track ALL genes that use this accession
            if let Some(first_entry) = gene_entries_list.first() {
                if !first_entry.protein_accession.is_empty() {
                    missing_accessions.insert(first_entry.protein_accession.clone());
                    acc_to_genes.entry(first_entry.protein_accession.clone())
                        .or_default()
                        .push(gene_id.clone());
                }
            }
        }
    }

    eprintln!("    Found {} genes in AMR_CDS.fa, {} unique accessions missing ({} genes)",
             sequences.len(), missing_accessions.len(),
             acc_to_genes.values().map(|v| v.len()).sum::<usize>());

    // Download missing protein-based sequences from NCBI
    if !missing_accessions.is_empty() {
        let acc_list: Vec<String> = missing_accessions.into_iter().collect();
        let downloaded = fetch_missing_cds(&acc_list)?;

        // Apply downloaded sequence to ALL genes that share this accession
        for (acc, seq) in downloaded {
            if let Some(gene_ids) = acc_to_genes.get(&acc) {
                for gene_id in gene_ids {
                    if let Some(entries_list) = gene_entries.get(gene_id) {
                        if let Some(first_entry) = entries_list.first() {
                            let (chemical_class, atc_code) = get_chemical_and_atc(&first_entry.ndaro_class);

                            let meta = GeneMeta {
                                gene_family: first_entry.gene_family.clone(),
                                drug_class: first_entry.ndaro_class.clone(),
                                chemical_class: chemical_class.to_string(),
                                atc_code: atc_code.to_string(),
                            };

                            sequences.insert(gene_id.clone(), seq.clone());
                            meta_genes.insert(gene_id.clone(), meta);
                        }
                    }
                }
            }
        }

        eprintln!("    After protein download: {} total genes", sequences.len());
    }

    // Fetch nucleotide-only sequences by coordinates (16S rRNA, etc.)
    if !nuc_only_entries.is_empty() {
        eprintln!("    Downloading nucleotide-only sequences...");

        // Build coordinate fetch list: (gene_id, nuc_acc, start, stop, strand)
        let coord_fetch_list: Vec<(String, String, u64, u64, String)> = nuc_only_entries.iter()
            .filter_map(|(gene_id, entry)| {
                if let (Some(nuc_acc), Some(start), Some(stop)) =
                    (&entry.nucleotide_accession, entry.nuc_start, entry.nuc_stop) {
                    let strand = entry.nuc_strand.clone().unwrap_or_else(|| "+".to_string());
                    Some((gene_id.clone(), nuc_acc.clone(), start, stop, strand))
                } else {
                    None
                }
            })
            .collect();

        // Fetch sequences
        let mut nuc_sequences: FxHashMap<String, String> = FxHashMap::default();
        fetch_by_coordinates(&coord_fetch_list, &mut nuc_sequences);

        // Add fetched sequences to results
        for (gene_id, entry) in &nuc_only_entries {
            if let Some(seq) = nuc_sequences.get(gene_id) {
                let (chemical_class, atc_code) = get_chemical_and_atc(&entry.ndaro_class);

                let meta = GeneMeta {
                    gene_family: entry.gene_family.clone(),
                    drug_class: entry.ndaro_class.clone(),
                    chemical_class: chemical_class.to_string(),
                    atc_code: atc_code.to_string(),
                };

                sequences.insert(gene_id.clone(), seq.clone());
                meta_genes.insert(gene_id.clone(), meta);
            }
        }

        eprintln!("    After nucleotide fetch: {} total genes", sequences.len());
    }

    let total_taxa: usize = gene_entries.iter()
        .filter(|(gene_id, _)| sequences.contains_key(*gene_id))
        .map(|(_, entries)| entries.len())
        .sum();

    let meta_db = ArgMetaDb {
        gene_count: meta_genes.len(),
        total_taxa_entries: total_taxa,
        genes: meta_genes,
    };

    eprintln!("    Built {} genes with {} taxa entries",
             meta_db.gene_count, meta_db.total_taxa_entries);
    Ok((sequences, meta_db))
}

// ============================================================================
// Public API
// ============================================================================

/// Builds the ARG reference database.
///
/// Downloads sequences from the specified source and creates:
/// - AMR_{source}.fas: FASTA file for alignment (metadata in headers)
/// - AMR_{source}.mmi: minimap2 index
///
/// # Arguments
/// * `output_dir` - Directory for output files
/// * `source` - Data source: "ncbi", "card", or "panres"
/// * `_threads` - Thread count (reserved for future use)
///
/// # Sources
/// - **ncbi**: NCBI AMRFinderPlus (recommended, curated gene names)
/// - **card**: CARD database (includes CARD-only genes marked with '^')
/// - **panres**: PanRes combined database from ARGprofiler (~14,000 genes)
pub fn build(output_dir: &Path, source: &str, _threads: usize) -> Result<()> {
    let source_suffix = match source {
        "ncbi" => "NCBI",
        "card" => "CARD",
        "panres" => "PanRes",
        _ => anyhow::bail!("Unknown source: {}. Use 'ncbi', 'card', or 'panres'", source),
    };

    std::fs::create_dir_all(output_dir)?;

    // Intermediate files (will be cleaned up)
    let catalogue_path = output_dir.join("ReferenceGeneCatalog.txt");
    let cds_path = output_dir.join("AMR_CDS.fa");
    let card_archive = output_dir.join("card-data.tar.bz2");
    let card_raw_dir = output_dir.join("card_raw");

    // Intermediate file (will be deleted after indexing)
    let out_fasta = output_dir.join(format!("AMR_{}.fas", source_suffix));
    // Output files with source suffix
    let out_mmi = output_dir.join(format!("AMR_{}.mmi", source_suffix));
    let out_tsv = output_dir.join(format!("AMR_{}.tsv", source_suffix));

    // Check if already complete
    if out_mmi.exists() && out_tsv.exists() {
        let mmi_size = std::fs::metadata(&out_mmi)?.len();
        if mmi_size > 1_000_000 {
            eprintln!("\n[RESUME] ARG database already exists, skipping build.");
            eprintln!("         Delete {} to rebuild.", out_mmi.display());
            return Ok(());
        }
    }

    // Build database based on source
    let (sequences, meta_db) = match source {
        "ncbi" => {
            eprintln!("\n[1] Downloading NCBI AMRFinderPlus database files...");
            if catalogue_path.exists() && std::fs::metadata(&catalogue_path)?.len() > 100_000 {
                eprintln!("    [RESUME] ReferenceGeneCatalog.txt exists, skipping...");
            } else {
                fetch_file(&format!("{}/ReferenceGeneCatalog.txt", NCBI_FTP_BASE), &catalogue_path)?;
            }
            if cds_path.exists() && std::fs::metadata(&cds_path)?.len() > 1_000_000 {
                eprintln!("    [RESUME] AMR_CDS.fa exists, skipping...");
            } else {
                fetch_file(&format!("{}/AMR_CDS.fa", NCBI_FTP_BASE), &cds_path)?;
            }

            let catalogue_entries = parse_reference_catalogue(&catalogue_path)?;
            let cds_sequences = parse_arg_sequences(&cds_path)?;
            build_from_ncbi(&catalogue_entries, &cds_sequences)?
        }
        "card" => {
            eprintln!("\n[1] Downloading reference files...");
            if !catalogue_path.exists() || std::fs::metadata(&catalogue_path)?.len() < 100_000 {
                eprintln!("    Downloading NCBI catalogue for gene name mapping...");
                fetch_file(&format!("{}/ReferenceGeneCatalog.txt", NCBI_FTP_BASE), &catalogue_path)?;
            } else {
                eprintln!("    [RESUME] NCBI catalogue exists for mapping...");
            }

            if !card_raw_dir.exists() || std::fs::read_dir(&card_raw_dir)?.count() < 5 {
                fetch_card_data(output_dir)?;
            } else {
                eprintln!("    [RESUME] CARD data exists, skipping...");
            }

            eprintln!("\n[2] Building gene name mapping...");
            let (ncbi_prot_mapping, ncbi_name_set) = build_protein_accession_map(&catalogue_path)?;
            let card_entries = parse_card_entries(&card_raw_dir, &ncbi_prot_mapping, &ncbi_name_set)?;
            build_from_card(&card_entries)?
        }
        "panres" => {
            let panres_raw_dir = output_dir.join("panres_raw");

            eprintln!("\n[1] Downloading PanRes database files...");
            if panres_raw_dir.exists() && panres_raw_dir.join("panres_genes.fa").exists() {
                eprintln!("    [RESUME] PanRes data exists, skipping...");
            } else {
                fetch_panres_data(output_dir)?;
            }

            eprintln!("\n[2] Parsing PanRes data...");
            let panres_entries = parse_panres_entries(&panres_raw_dir)?;
            build_from_panres(&panres_entries)?
        }
        _ => unreachable!(),
    };

    // Write output files
    eprintln!("\n[5] Writing output files...");

    let fasta_name = out_fasta.file_name().unwrap().to_str().unwrap();
    let tsv_name = out_tsv.file_name().unwrap().to_str().unwrap();
    eprintln!("    Writing {} (intermediate)...", fasta_name);
    eprintln!("    Writing {}...", tsv_name);
    {
        let mut fasta_writer = BufWriter::new(File::create(&out_fasta)?);
        let mut tsv_writer = BufWriter::new(File::create(&out_tsv)?);

        // TSV header
        writeln!(tsv_writer, "gene_name\tgene_family\tdrug_class\tchemical_class\tatc_code")?;

        let mut sorted_genes: Vec<_> = sequences.iter().collect();
        sorted_genes.sort_by_key(|(k, _)| *k);

        for (gene_id, sequence) in sorted_genes {
            if let Some(meta) = meta_db.genes.get(gene_id) {
                let display_name = if meta.gene_family.starts_with("CARD:") {
                    format!("{}^", gene_id)
                } else {
                    gene_id.clone()
                };

                // Write FASTA
                writeln!(fasta_writer, ">{}|{}|{}|{}",
                        display_name, meta.drug_class, meta.chemical_class, meta.atc_code)?;
                for chunk in sequence.as_bytes().chunks(80) {
                    writeln!(fasta_writer, "{}", std::str::from_utf8(chunk)?)?;
                }

                // Write TSV row
                writeln!(tsv_writer, "{}\t{}\t{}\t{}\t{}",
                        display_name, meta.gene_family, meta.drug_class, meta.chemical_class, meta.atc_code)?;
            }
        }
    }

    // Build minimap2 index
    eprintln!("\n[6] Building minimap2 index...");
    let mm2_result = Command::new("minimap2")
        .args(["-d", out_mmi.to_str().unwrap(), out_fasta.to_str().unwrap()])
        .output();

    let mmi_name = out_mmi.file_name().unwrap().to_str().unwrap();
    match mm2_result {
        Ok(output) => {
            if output.status.success() {
                eprintln!("    Created {}", mmi_name);
            } else {
                eprintln!("    Warning: minimap2 indexing failed");
                eprintln!("    {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("    Warning: minimap2 not found: {}", e);
            eprintln!("    Run manually: minimap2 -d {} {}", out_mmi.display(), out_fasta.display());
        }
    }

    // Clean up intermediate files
    eprintln!("\n[7] Cleaning up intermediate files...");
    let mut cleaned = 0;
    // Include FASTA in cleanup since we have the .mmi index
    let mmi_exists = out_mmi.exists();
    for path in [&catalogue_path, &cds_path, &card_archive] {
        if path.exists() && std::fs::remove_file(path).is_ok() {
            cleaned += 1;
        }
    }
    // Only delete FASTA if mmi was successfully created
    if mmi_exists && out_fasta.exists() && std::fs::remove_file(&out_fasta).is_ok() {
        cleaned += 1;
    }
    if card_raw_dir.exists() && std::fs::remove_dir_all(&card_raw_dir).is_ok() {
        cleaned += 1;
    }
    let panres_raw_dir = output_dir.join("panres_raw");
    if panres_raw_dir.exists() && std::fs::remove_dir_all(&panres_raw_dir).is_ok() {
        cleaned += 1;
    }
    if cleaned > 0 {
        eprintln!("    Removed {} intermediate file(s)", cleaned);
    }

    // Summary
    let mmi_size = std::fs::metadata(&out_mmi).map(|m| m.len()).unwrap_or(0);
    let tsv_size = std::fs::metadata(&out_tsv).map(|m| m.len()).unwrap_or(0);

    eprintln!("\n============================================================");
    eprintln!(" Build Complete!");
    eprintln!("============================================================");
    eprintln!("Files created:");
    if mmi_size > 0 {
        eprintln!("  {}: {:.2} MB ({} genes)", mmi_name, mmi_size as f64 / 1_000_000.0, meta_db.gene_count);
    }
    if tsv_size > 0 {
        eprintln!("  {}: {:.2} KB", tsv_name, tsv_size as f64 / 1_000.0);
    }

    if source == "card" {
        eprintln!("\nNote: Genes with '^' suffix are CARD-only (not in NCBI).");
    }

    Ok(())
}

/// Builds flanking sequence database.
///
/// Delegates to flanking_db module for the actual implementation.
pub fn build_flanking_db(
    output_dir: &Path,
    arg_db: &Path,
    threads: usize,
    email: &str,
    config: crate::flanking_db::FlankBuildConfig,
) -> Result<()> {
    crate::flanking_db::build(output_dir, arg_db, threads, email, config)
}

/// Builds ARG database from a pre-built unified FASTA file.
///
/// This function takes an existing unified ARG database (e.g., created from
/// ARO-normalized sequences from multiple sources) and creates:
/// - AMR_unified.mmi: minimap2 index for alignment
/// - AMR_unified.tsv: metadata file with gene annotations
///
/// Expected FASTA header format: >ARO_ID|gene_name|source|length
/// Example: >ARO:3000005|vanD|CARD|1032
///
/// # Arguments
/// * `output_dir` - Directory for output files
/// * `unified_fasta` - Path to the pre-built unified ARG FASTA file
/// * `_threads` - Thread count (reserved for future use)
pub fn build_from_unified(output_dir: &Path, unified_fasta: &Path, _threads: usize) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    let out_fasta = output_dir.join("AMR_unified.fas");
    let out_mmi = output_dir.join("AMR_unified.mmi");
    let out_tsv = output_dir.join("AMR_unified.tsv");

    // Check if already complete
    if out_mmi.exists() && out_tsv.exists() {
        let mmi_size = std::fs::metadata(&out_mmi)?.len();
        if mmi_size > 1_000_000 {
            eprintln!("\n[RESUME] Unified ARG database already exists, skipping build.");
            eprintln!("         Delete {} to rebuild.", out_mmi.display());
            return Ok(());
        }
    }

    eprintln!("\n[1] Reading unified ARG database...");
    eprintln!("    Input: {}", unified_fasta.display());

    // Parse unified FASTA and convert to ARGenus format
    let file = File::open(unified_fasta)?;
    let reader = BufReader::new(file);

    let mut sequences: Vec<(String, String, String, String, String)> = Vec::new(); // (aro_id, gene_name, source, drug_class, sequence)
    let mut current_header: Option<(String, String, String)> = None;
    let mut current_seq = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('>') {
            // Save previous sequence
            if let Some((aro_id, gene_name, source)) = current_header.take() {
                if !current_seq.is_empty() {
                    let drug_class = infer_drug_class_from_gene(&gene_name, &gene_name);
                    sequences.push((aro_id, gene_name, source, drug_class, std::mem::take(&mut current_seq)));
                }
            }

            // Parse header: >ARO_ID|gene_name|source|length
            let header = line.trim_start_matches('>');
            let parts: Vec<&str> = header.split('|').collect();

            let aro_id = parts.first().unwrap_or(&"").to_string();
            let gene_name = parts.get(1).unwrap_or(&"unknown").to_string();
            let source = parts.get(2).unwrap_or(&"unified").to_string();

            current_header = Some((aro_id, gene_name, source));
            current_seq.clear();
        } else {
            current_seq.push_str(line.trim());
        }
    }

    // Save last sequence
    if let Some((aro_id, gene_name, source)) = current_header {
        if !current_seq.is_empty() {
            let drug_class = infer_drug_class_from_gene(&gene_name, &gene_name);
            sequences.push((aro_id, gene_name, source, drug_class, current_seq));
        }
    }

    eprintln!("    Loaded {} sequences", sequences.len());

    // Write output FASTA with ARGenus format
    eprintln!("\n[2] Writing output files...");
    {
        let mut fasta_writer = BufWriter::new(File::create(&out_fasta)?);
        let mut tsv_writer = BufWriter::new(File::create(&out_tsv)?);

        writeln!(tsv_writer, "aro_id\tgene_name\tsource\tdrug_class")?;

        for (aro_id, gene_name, source, drug_class, sequence) in &sequences {
            let (chemical_class, atc_code) = get_chemical_and_atc(drug_class);

            // Write FASTA: >gene_name|drug_class|chemical_class|atc_code
            // Using gene_name as display ID for compatibility with existing detection pipeline
            writeln!(fasta_writer, ">{}|{}|{}|{}", gene_name, drug_class, chemical_class, atc_code)?;
            for chunk in sequence.as_bytes().chunks(80) {
                writeln!(fasta_writer, "{}", std::str::from_utf8(chunk)?)?;
            }

            // Write TSV
            writeln!(tsv_writer, "{}\t{}\t{}\t{}", aro_id, gene_name, source, drug_class)?;
        }
    }

    // Build minimap2 index
    eprintln!("\n[3] Building minimap2 index...");
    let mm2_result = Command::new("minimap2")
        .args(["-d", out_mmi.to_str().unwrap(), out_fasta.to_str().unwrap()])
        .output();

    match mm2_result {
        Ok(output) => {
            if output.status.success() {
                eprintln!("    Created AMR_unified.mmi");
            } else {
                eprintln!("    Warning: minimap2 indexing failed");
                eprintln!("    {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("    Warning: minimap2 not found: {}", e);
            eprintln!("    Run manually: minimap2 -d {} {}", out_mmi.display(), out_fasta.display());
        }
    }

    // Clean up intermediate FASTA if mmi was created successfully
    if out_mmi.exists() && out_fasta.exists() {
        let _ = std::fs::remove_file(&out_fasta);
        eprintln!("    Removed intermediate FASTA file");
    }

    // Summary
    let mmi_size = std::fs::metadata(&out_mmi).map(|m| m.len()).unwrap_or(0);
    let tsv_size = std::fs::metadata(&out_tsv).map(|m| m.len()).unwrap_or(0);

    eprintln!("\n============================================================");
    eprintln!(" Build Complete!");
    eprintln!("============================================================");
    eprintln!("Files created:");
    if mmi_size > 0 {
        eprintln!("  AMR_unified.mmi: {:.2} MB ({} genes)", mmi_size as f64 / 1_000_000.0, sequences.len());
    }
    if tsv_size > 0 {
        eprintln!("  AMR_unified.tsv: {:.2} KB", tsv_size as f64 / 1_000.0);
    }

    eprintln!("\nUsage:");
    eprintln!("  argenus -a {}/AMR_unified.mmi -f flanking.fdb -1 R1.fq -2 R2.fq -o output/", output_dir.display());

    Ok(())
}
