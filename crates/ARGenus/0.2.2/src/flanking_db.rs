//! Flanking Database Builder Module
//!
//! Downloads prokaryotic genomes from NCBI and PLSDB, aligns ARG sequences,
//! and extracts flanking sequences for genus classification.
//!
//! # Data Sources
//! - **NCBI**: GenBank (bacteria + archaea), Complete Genome + Chromosome level
//! - **PLSDB**: Standalone circular complete plasmids (ASSEMBLY_UID == -1)
//!
//! # Pipeline Overview
//! 1. Download NCBI taxonomy database (taxdump)
//! 2. Fetch prokaryotic genome assemblies
//! 3. Download PLSDB plasmid sequences
//! 4. Align ARG database to genomes (minimap2)
//! 5. Extract flanking sequences with genus annotation
//! 6. Build compressed FDB index
//!
//! # Output Files
//! - `genome_catalogue.tsv`: Accession → genus mapping
//! - `flanking_db.tsv`: Raw flanking sequences
//! - `flanking.fdb`: Compressed indexed database
//!
//! # External Dependencies
//! - minimap2 (alignment)

use anyhow::{Context, Result};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use std::thread;

const NCBI_FTP_BASE: &str = "https://ftp.ncbi.nlm.nih.gov/genomes";
const NCBI_TAXDUMP_URL: &str = "https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz";
const NCBI_DATASETS_API: &str = "https://api.ncbi.nlm.nih.gov/datasets/v2";
const PLSDB_META_URL: &str = "https://ccb-microbe.cs.uni-saarland.de/plsdb2025/download_meta.tar.gz";
const PLSDB_FASTA_URL: &str = "https://ccb-microbe.cs.uni-saarland.de/plsdb2025/download_fasta";
// Default flanking length: 1000bp (configurable via -n flag)
const API_BATCH_SIZE: usize = 1000; // NCBI recommends <1000 per request

/// PLSDB options for flanking database build
#[derive(Clone, Debug, Default)]
pub struct PlsdbOptions {
    /// Pre-downloaded PLSDB directory (contains meta.tar.gz and sequences.fasta)
    pub dir: Option<PathBuf>,
    /// Skip PLSDB processing entirely
    pub skip: bool,
}

/// Configuration for flanking database build
#[derive(Clone, Debug)]
pub struct FlankBuildConfig {
    /// Flanking region length in bp (upstream + downstream)
    pub flanking_length: usize,
    /// Queue buffer size in GB for backpressure control
    pub queue_buffer_gb: u32,
    /// PLSDB options
    pub plsdb: PlsdbOptions,
}

impl Default for FlankBuildConfig {
    fn default() -> Self {
        Self {
            flanking_length: 1000,
            queue_buffer_gb: 30,
            plsdb: PlsdbOptions::default(),
        }
    }
}

/// Assembly metadata from NCBI
#[derive(Clone, Debug)]
pub struct AssemblyInfo {
    pub accession: String,
    pub taxid: String,
    pub species_taxid: String,
    pub organism_name: String,
}

/// Plasmid metadata from PLSDB
#[derive(Clone, Debug)]
pub struct PlasmidInfo {
    pub accession: String,
    pub taxonomy_uid: String,
    pub genus: String,
    pub species: String,
}

/// Parsed PAF alignment hit for tie-breaking
#[derive(Clone, Debug)]
struct PafHit {
    query_name: String,
    query_start: usize,
    query_end: usize,
    gene_name: String,
    gene_length: usize,
    score: i64,
    mapq: u8,
    divergence: f32,
    gap_count: usize,
    raw_line: String,
}

impl PafHit {
    fn from_paf_line(line: &str) -> Option<Self> {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 12 {
            return None;
        }

        let query_name = fields[0].to_string();
        let query_start: usize = fields[2].parse().ok()?;
        let query_end: usize = fields[3].parse().ok()?;
        let gene_name = fields[5].to_string();
        let gene_length: usize = fields[6].parse().ok()?;
        let mapq: u8 = fields[11].parse().unwrap_or(0);

        let matches: usize = fields[9].parse().unwrap_or(0);
        let block_len: usize = fields[10].parse().unwrap_or(0);

        let mut score: i64 = 0;
        let mut divergence: f32 = 1.0;

        for field in &fields[12..] {
            if let Some(val) = field.strip_prefix("AS:i:") {
                score = val.parse().unwrap_or(0);
            } else if let Some(val) = field.strip_prefix("de:f:") {
                divergence = val.parse().unwrap_or(1.0);
            }
        }

        let gap_count = block_len.saturating_sub(matches);

        Some(PafHit {
            query_name,
            query_start,
            query_end,
            gene_name,
            gene_length,
            score,
            mapq,
            divergence,
            gap_count,
            raw_line: line.to_string(),
        })
    }

    fn overlaps(&self, other: &PafHit) -> bool {
        if self.query_name != other.query_name {
            return false;
        }
        let start = self.query_start.max(other.query_start);
        let end = self.query_end.min(other.query_end);
        if start >= end {
            return false;
        }
        let overlap = end - start;
        let self_len = self.query_end - self.query_start;
        let other_len = other.query_end - other.query_start;
        overlap * 2 > self_len || overlap * 2 > other_len
    }
}

/// Compare hits: Score↓ → Gene length↓ → MapQ↓ → Divergence↑ → Gap↑ → Name
fn compare_paf_hits(a: &PafHit, b: &PafHit) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    b.score.cmp(&a.score)
        .then_with(|| b.gene_length.cmp(&a.gene_length))
        .then_with(|| b.mapq.cmp(&a.mapq))
        .then_with(|| a.divergence.partial_cmp(&b.divergence).unwrap_or(Ordering::Equal))
        .then_with(|| a.gap_count.cmp(&b.gap_count))
        .then_with(|| a.gene_name.cmp(&b.gene_name))
}

/// Deduplicate overlapping hits using tie-breaking rules
fn deduplicate_paf_hits(hits: Vec<PafHit>) -> Vec<PafHit> {
    if hits.is_empty() {
        return hits;
    }

    let mut groups: Vec<Vec<PafHit>> = Vec::new();
    for hit in hits {
        let mut found = false;
        for group in &mut groups {
            if group.iter().any(|h| h.overlaps(&hit)) {
                group.push(hit.clone());
                found = true;
                break;
            }
        }
        if !found {
            groups.push(vec![hit]);
        }
    }

    groups
        .into_iter()
        .map(|mut g| {
            g.sort_by(compare_paf_hits);
            g.into_iter().next().unwrap()
        })
        .collect()
}

// ============================================================================
// Adaptive Thread Bucketing
// ============================================================================

/// Genome size category for adaptive thread allocation
/// Larger genomes benefit from more minimap2 threads
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenomeSizeCategory {
    /// Small genomes (<5MB): 4 threads for minimap2
    Small,
    /// Medium genomes (5-20MB): 6 threads for minimap2
    Medium,
    /// Large genomes (>20MB): 8 threads for minimap2
    Large,
}

/// Size thresholds in bytes
const SMALL_GENOME_THRESHOLD: u64 = 5 * 1024 * 1024;   // 5MB
const LARGE_GENOME_THRESHOLD: u64 = 20 * 1024 * 1024;  // 20MB

impl GenomeSizeCategory {
    /// Determine category from file size in bytes
    pub fn from_size(size_bytes: u64) -> Self {
        if size_bytes < SMALL_GENOME_THRESHOLD {
            GenomeSizeCategory::Small
        } else if size_bytes < LARGE_GENOME_THRESHOLD {
            GenomeSizeCategory::Medium
        } else {
            GenomeSizeCategory::Large
        }
    }

    /// Get recommended minimap2 thread count for this category
    pub fn thread_count(&self, max_threads: usize) -> usize {
        let recommended = match self {
            GenomeSizeCategory::Small => 4,
            GenomeSizeCategory::Medium => 6,
            GenomeSizeCategory::Large => 8,
        };
        // Don't exceed available threads
        recommended.min(max_threads)
    }

    /// Get category name for logging
    pub fn name(&self) -> &'static str {
        match self {
            GenomeSizeCategory::Small => "small (<5MB)",
            GenomeSizeCategory::Medium => "medium (5-20MB)",
            GenomeSizeCategory::Large => "large (>20MB)",
        }
    }
}

/// Bucket genome files by size category
/// Returns a map of category -> list of genome paths
fn bucket_genomes_by_size(genome_files: &[PathBuf]) -> FxHashMap<GenomeSizeCategory, Vec<PathBuf>> {
    let mut buckets: FxHashMap<GenomeSizeCategory, Vec<PathBuf>> = FxHashMap::default();

    for path in genome_files {
        let size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        let category = GenomeSizeCategory::from_size(size);
        buckets.entry(category).or_default().push(path.clone());
    }

    buckets
}

// ============================================================================
// Genome Catalogue
// ============================================================================

/// Unified genome catalogue entry
/// Maps accession to taxonomy information
#[derive(Clone, Debug)]
pub struct CatalogEntry {
    pub accession: String,      // GCF_/GCA_ or NZ_/CP accession
    pub taxid: String,          // NCBI taxonomy ID
    pub species_taxid: String,  // Species-level taxonomy ID
    pub genus: String,          // Genus name (parsed from organism name)
    pub species: String,        // Species name
    pub organism_name: String,  // Full organism name
    pub source: String,         // "refseq", "genbank", or "plsdb"
}

/// Genome catalog for accession-to-genus lookup
pub struct GenomeCatalog {
    entries: FxHashMap<String, CatalogEntry>,
}

impl Default for GenomeCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl GenomeCatalog {
    pub fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
        }
    }

    /// Add an assembly to the catalog with taxonomy-based genus lookup
    pub fn add_assembly_with_taxonomy(
        &mut self,
        asm: &AssemblyInfo,
        source: &str,
        taxonomy: Option<&TaxonomyDB>,
    ) {
        let (parsed_genus, species) = parse_organism_name(&asm.organism_name);

        // Resolve genus with smart fallback (uncultured, taxonomy, higher ranks)
        let genus = resolve_genus(&asm.organism_name, &asm.taxid, &parsed_genus, taxonomy);

        let entry = CatalogEntry {
            accession: asm.accession.clone(),
            taxid: asm.taxid.clone(),
            species_taxid: asm.species_taxid.clone(),
            genus,
            species,
            organism_name: asm.organism_name.clone(),
            source: source.to_string(),
        };

        // Index by full accession and base accession (without version)
        self.entries.insert(asm.accession.clone(), entry.clone());
        if let Some(base) = asm.accession.split('.').next() {
            self.entries.insert(base.to_string(), entry);
        }
    }

    /// Add a PLSDB plasmid to the catalog with taxonomy-based genus lookup
    pub fn add_plasmid_with_taxonomy(
        &mut self,
        plasmid: &PlasmidInfo,
        taxonomy: Option<&TaxonomyDB>,
    ) {
        // Construct organism name from genus + species
        let organism_name = format!("{} {}", plasmid.genus, plasmid.species);

        // Resolve genus with smart fallback (uncultured, taxonomy, higher ranks)
        let genus = resolve_genus(&organism_name, &plasmid.taxonomy_uid, &plasmid.genus, taxonomy);

        let entry = CatalogEntry {
            accession: plasmid.accession.clone(),
            taxid: plasmid.taxonomy_uid.clone(),
            species_taxid: String::new(),
            genus,
            species: plasmid.species.clone(),
            organism_name,
            source: "plsdb".to_string(),
        };

        // Index by full accession and variants
        self.entries.insert(plasmid.accession.clone(), entry.clone());

        // Also index by base accession (without NZ_ prefix and version)
        let base = plasmid.accession
            .strip_prefix("NZ_")
            .unwrap_or(&plasmid.accession);
        let no_ver = base.split('.').next().unwrap_or(base);
        self.entries.insert(no_ver.to_string(), entry.clone());

        // Also index by underscore-replaced version (for filename-derived lookups)
        // e.g., NZ_CP157204.1 -> NZ_CP157204_1
        let underscore_key = plasmid.accession.replace('.', "_");
        self.entries.insert(underscore_key, entry);
    }

    /// Look up genus by accession (tries multiple variants)
    pub fn get_genus(&self, accession: &str) -> Option<&str> {
        // Try exact match first
        if let Some(entry) = self.entries.get(accession) {
            return Some(&entry.genus);
        }

        // Try without version
        if let Some(base) = accession.split('.').next() {
            if let Some(entry) = self.entries.get(base) {
                return Some(&entry.genus);
            }
        }

        // Try without NZ_ prefix
        let stripped = accession.strip_prefix("NZ_").unwrap_or(accession);
        if let Some(entry) = self.entries.get(stripped) {
            return Some(&entry.genus);
        }
        let stripped_base = stripped.split('.').next().unwrap_or(stripped);
        if let Some(entry) = self.entries.get(stripped_base) {
            return Some(&entry.genus);
        }

        // Try with underscore-to-dot conversion (for PLSDB filenames like NZ_CP157204_1)
        // Pattern: NZ_ABC123_1 -> NZ_ABC123.1
        if let Some(last_underscore) = accession.rfind('_') {
            let suffix = &accession[last_underscore + 1..];
            // Check if suffix looks like a version number (digits only)
            if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                let with_dot = format!("{}.{}", &accession[..last_underscore], suffix);
                if let Some(entry) = self.entries.get(&with_dot) {
                    return Some(&entry.genus);
                }
                // Also try without NZ_ prefix
                let stripped_with_dot = with_dot.strip_prefix("NZ_").unwrap_or(&with_dot);
                if let Some(entry) = self.entries.get(stripped_with_dot) {
                    return Some(&entry.genus);
                }
            }
        }

        None
    }

    /// Save catalog to TSV file
    pub fn save(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "accession\ttaxid\tspecies_taxid\tgenus\tspecies\torganism_name\tsource")?;

        // Deduplicate entries (same accession may be indexed multiple ways)
        let mut seen: FxHashSet<&str> = FxHashSet::default();
        for entry in self.entries.values() {
            if seen.contains(entry.accession.as_str()) {
                continue;
            }
            seen.insert(&entry.accession);

            writeln!(
                writer,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                entry.accession,
                entry.taxid,
                entry.species_taxid,
                entry.genus,
                entry.species,
                entry.organism_name.replace('\t', " "),
                entry.source
            )?;
        }

        Ok(())
    }

    /// Get entry count (unique accessions)
    pub fn len(&self) -> usize {
        let mut seen: FxHashSet<&str> = FxHashSet::default();
        for entry in self.entries.values() {
            seen.insert(&entry.accession);
        }
        seen.len()
    }

    /// Check if catalog is empty (required by clippy for len())
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Parse organism name to extract genus and species
fn parse_organism_name(organism: &str) -> (String, String) {
    let parts: Vec<&str> = organism.split_whitespace().collect();

    let genus = parts.first()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let species = if parts.len() >= 2 {
        // Join remaining parts as species (handles "sp." and strain info)
        parts[1..].join(" ")
    } else {
        "Unknown".to_string()
    };

    (clean_genus(&genus), species)
}

/// Clean genus name: remove parentheses and citations
fn clean_genus(genus: &str) -> String {
    // Remove everything after '(' including the parenthesis
    // e.g., "Methylacidiphilum (ex Ratnadevi et al. 2023)" -> "Methylacidiphilum"
    let cleaned = if let Some(idx) = genus.find('(') {
        genus[..idx].trim()
    } else {
        genus.trim()
    };

    cleaned.to_string()
}

/// Determine genus with smart fallback logic
/// Priority:
/// 1. If organism contains "uncultured" -> "uncultured"
/// 2. Use taxonomy database to find genus or higher rank
/// 3. Fall back to parsed genus from organism name
fn resolve_genus(
    organism_name: &str,
    taxid_str: &str,
    _parsed_genus: &str,
    taxonomy: Option<&TaxonomyDB>,
) -> String {
    // Check for uncultured
    let organism_lower = organism_name.to_lowercase();
    if organism_lower.contains("uncultured") {
        return "uncultured".to_string();
    }

    // Try taxonomy database
    if let Some(tax_db) = taxonomy {
        if let Ok(taxid) = taxid_str.parse::<u32>() {
            // First try to get genus directly
            if let Some(genus) = tax_db.get_genus(taxid) {
                return clean_genus(&genus);
            }

            // If no genus, try to get higher rank (family, order, class)
            if let Some((name, _rank)) = tax_db.get_genus_or_higher(taxid) {
                return clean_genus(&name);
            }
        }
    }

    // Fall back: use full organism name (cleaned) when taxonomy lookup fails
    // This preserves more information (e.g., "Dehalococcoidia bacterium" instead of just "Dehalococcoidia")
    let cleaned = clean_genus(organism_name);
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned
    }
}

/// NCBI Taxonomy database for taxid → genus lookup
/// Parses names.dmp and nodes.dmp from taxdump
pub struct TaxonomyDB {
    /// taxid → scientific name
    names: FxHashMap<u32, String>,
    /// taxid → (parent_taxid, rank)
    nodes: FxHashMap<u32, (u32, String)>,
}

impl Default for TaxonomyDB {
    fn default() -> Self {
        Self::new()
    }
}

impl TaxonomyDB {
    pub fn new() -> Self {
        Self {
            names: FxHashMap::default(),
            nodes: FxHashMap::default(),
        }
    }

    /// Load taxonomy from taxdump directory
    pub fn load(taxdump_dir: &Path) -> Result<Self> {
        let mut db = Self::new();

        let names_path = taxdump_dir.join("names.dmp");
        let nodes_path = taxdump_dir.join("nodes.dmp");

        if !names_path.exists() || !nodes_path.exists() {
            anyhow::bail!("Taxonomy files not found in {}", taxdump_dir.display());
        }

        // Parse names.dmp
        // Format: tax_id | name_txt | unique name | name class |
        eprintln!("    Loading names.dmp...");
        let file = File::open(&names_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split("\t|\t").collect();
            if fields.len() < 4 {
                continue;
            }

            // Only keep scientific names
            let name_class = fields[3].trim_end_matches("\t|");
            if name_class != "scientific name" {
                continue;
            }

            let taxid: u32 = match fields[0].parse() {
                Ok(id) => id,
                Err(_) => continue,
            };
            let name = fields[1].to_string();

            db.names.insert(taxid, name);
        }

        // Parse nodes.dmp
        // Format: tax_id | parent_tax_id | rank | ...
        eprintln!("    Loading nodes.dmp...");
        let file = File::open(&nodes_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split("\t|\t").collect();
            if fields.len() < 3 {
                continue;
            }

            let taxid: u32 = match fields[0].parse() {
                Ok(id) => id,
                Err(_) => continue,
            };
            let parent_taxid: u32 = match fields[1].parse() {
                Ok(id) => id,
                Err(_) => continue,
            };
            let rank = fields[2].to_string();

            db.nodes.insert(taxid, (parent_taxid, rank));
        }

        eprintln!("    Loaded {} taxa, {} nodes", db.names.len(), db.nodes.len());
        Ok(db)
    }

    /// Get genus name for a taxid by traversing up the tree
    pub fn get_genus(&self, taxid: u32) -> Option<String> {
        let mut current = taxid;
        let mut visited = 0;

        // Traverse up to 50 levels (safety limit)
        while visited < 50 {
            if let Some((parent, rank)) = self.nodes.get(&current) {
                if rank == "genus" {
                    return self.names.get(&current).cloned();
                }

                // Check if we're at the root (parent == self)
                if *parent == current {
                    return None;
                }

                current = *parent;
                visited += 1;
            } else {
                return None;
            }
        }

        None
    }

    /// Get genus or higher rank (family, order, class) if genus not found
    /// Returns (name, rank) tuple
    pub fn get_genus_or_higher(&self, taxid: u32) -> Option<(String, String)> {
        let mut current = taxid;
        let mut visited = 0;

        // Priority ranks to return if genus not found
        let target_ranks = ["genus", "family", "order", "class", "phylum"];
        let mut best_match: Option<(String, String, usize)> = None; // (name, rank, priority)

        // Traverse up to 50 levels (safety limit)
        while visited < 50 {
            if let Some((parent, rank)) = self.nodes.get(&current) {
                // Check if this rank is one we want
                if let Some(priority) = target_ranks.iter().position(|r| r == rank) {
                    if let Some(name) = self.names.get(&current) {
                        // Return immediately if genus found
                        if rank == "genus" {
                            return Some((name.clone(), rank.clone()));
                        }
                        // Otherwise, keep track of best (highest priority = lowest index)
                        if best_match.is_none() || priority < best_match.as_ref().unwrap().2 {
                            best_match = Some((name.clone(), rank.clone(), priority));
                        }
                    }
                }

                // Check if we're at the root (parent == self)
                if *parent == current {
                    break;
                }

                current = *parent;
                visited += 1;
            } else {
                break;
            }
        }

        best_match.map(|(name, rank, _)| (name, rank))
    }
}

/// Build state for resume/tracking
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BuildState {
    pub started_at: String,
    pub last_updated: String,
    pub current_step: String,
    pub completed_steps: Vec<String>,
    pub genomes_downloaded: usize,
    pub plsdb_extracted: usize,
    pub alignments_done: bool,
    pub flanking_extracted: bool,
}

impl Default for BuildState {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildState {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            started_at: now.clone(),
            last_updated: now,
            current_step: String::new(),
            completed_steps: Vec::new(),
            genomes_downloaded: 0,
            plsdb_extracted: 0,
            alignments_done: false,
            flanking_extracted: false,
        }
    }

    pub fn load(path: &Path) -> Option<Self> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn update_step(&mut self, step: &str) {
        self.current_step = step.to_string();
        self.last_updated = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    pub fn complete_step(&mut self, step: &str) {
        if !self.completed_steps.contains(&step.to_string()) {
            self.completed_steps.push(step.to_string());
        }
        self.last_updated = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    pub fn is_completed(&self, step: &str) -> bool {
        self.completed_steps.contains(&step.to_string())
    }
}

/// Builder configuration
pub struct FlankingDbBuilder {
    output_dir: PathBuf,
    threads: usize,
    amr_db_path: PathBuf,
    /// NCBI email for API access (required, enables higher rate limits)
    email: String,
    /// Flanking build configuration
    config: FlankBuildConfig,
}

impl FlankingDbBuilder {
    pub fn new(amr_db: &Path, output_dir: &Path, threads: usize, email: &str, config: FlankBuildConfig) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
            threads,
            amr_db_path: amr_db.to_path_buf(),
            email: email.to_string(),
            config,
        }
    }

    /// Run the complete pipeline with resume support
    pub fn build(&self) -> Result<()> {
        eprintln!("\n============================================================");
        eprintln!(" ARGenus Flanking Database Builder (Rust)");
        eprintln!("============================================================");
        eprintln!("Output directory: {}", self.output_dir.display());
        eprintln!("NCBI Email: {}", self.email);
        eprintln!("Download method: NCBI Datasets API (batch)");
        eprintln!("Threads: {}", self.threads);
        eprintln!("Flanking length: {} bp", self.config.flanking_length);
        eprintln!("Queue buffer: {} GB", self.config.queue_buffer_gb);
        eprintln!("Alignment: minimap2 (AMR indexed)");
        eprintln!();

        std::fs::create_dir_all(&self.output_dir)?;
        let genomes_dir = self.output_dir.join("genomes");
        std::fs::create_dir_all(&genomes_dir)?;
        // Use user-provided PLSDB directory or create default
        let plsdb_dir = self.config.plsdb.dir.clone().unwrap_or_else(|| self.output_dir.join("plsdb"));
        if !self.config.plsdb.skip {
            std::fs::create_dir_all(&plsdb_dir)?;
        }
        let taxonomy_dir = self.output_dir.join("taxonomy");
        let temp_dir = self.output_dir.join("temp");
        std::fs::create_dir_all(&temp_dir)?;
        let state_path = self.output_dir.join("build_state.json");

        // Load or create build state
        let mut state = BuildState::load(&state_path).unwrap_or_else(|| {
            eprintln!("Starting fresh build...");
            BuildState::new()
        });

        if !state.completed_steps.is_empty() {
            eprintln!("Resuming from previous state:");
            eprintln!("  Started: {}", state.started_at);
            eprintln!("  Last updated: {}", state.last_updated);
            eprintln!("  Completed steps: {}", state.completed_steps.join(", "));
            eprintln!("  Genomes downloaded: {}", state.genomes_downloaded);
            eprintln!("  PLSDB extracted: {}", state.plsdb_extracted);
            eprintln!();
        }

        // Initialize unified catalog
        let mut catalog = GenomeCatalog::new();
        let catalog_path = self.output_dir.join("genome_catalog.tsv");

        // Step 1: Download NCBI taxdump (for authoritative taxonomy)
        if !state.is_completed("taxonomy_download") {
            state.update_step("taxonomy_download");
            state.save(&state_path)?;
            eprintln!("[1/9] Downloading NCBI taxonomy database...");
            self.download_taxdump(&taxonomy_dir)?;
            state.complete_step("taxonomy_download");
            state.save(&state_path)?;
        } else {
            eprintln!("[1/9] Taxonomy database already downloaded, skipping...");
        }

        // Step 2: Load taxonomy database
        eprintln!("[2/9] Loading taxonomy database...");
        let taxonomy = match TaxonomyDB::load(&taxonomy_dir) {
            Ok(db) => {
                eprintln!("    Taxonomy database loaded successfully");
                Some(db)
            }
            Err(e) => {
                eprintln!("    Warning: Failed to load taxonomy: {}", e);
                eprintln!("    Genus will be parsed from organism names instead");
                None
            }
        };

        // Step 3: Download NCBI assembly summary (GenBank only)
        eprintln!("[3/9] Downloading NCBI assembly summaries (GenBank)...");
        let assemblies = self.download_assembly_summaries()?;
        eprintln!("    GenBank assemblies: {} (Complete Genome + Chromosome)", assemblies.len());

        // Add assemblies to catalog (with taxonomy-based genus if available)
        for asm in &assemblies {
            catalog.add_assembly_with_taxonomy(asm, "genbank", taxonomy.as_ref());
        }

        // Step 4: Download PLSDB (skip if --skip-plsdb or --plsdb-dir provided)
        let standalone_plasmids = if self.config.plsdb.skip {
            eprintln!("[4/9] PLSDB skipped (--skip-plsdb)");
            eprintln!("[5/9] PLSDB metadata skipped");
            Vec::new()
        } else if self.config.plsdb.dir.is_some() {
            // User provided pre-downloaded PLSDB directory
            eprintln!("[4/9] Using pre-downloaded PLSDB: {}", plsdb_dir.display());
            // Validate required files exist
            let nuccore_csv = plsdb_dir.join("nuccore.csv");
            let fasta_path = plsdb_dir.join("sequences.fasta");
            if !nuccore_csv.exists() {
                // Check if meta.tar.gz needs extraction
                let meta_tar = plsdb_dir.join("meta.tar.gz");
                if meta_tar.exists() {
                    eprintln!("    Extracting meta.tar.gz...");
                    std::process::Command::new("tar")
                        .args(["-xzf", meta_tar.to_str().unwrap(), "-C", plsdb_dir.to_str().unwrap()])
                        .status()
                        .with_context(|| "Failed to extract PLSDB metadata")?;
                } else {
                    anyhow::bail!("PLSDB directory missing nuccore.csv and meta.tar.gz: {}", plsdb_dir.display());
                }
            }
            if !fasta_path.exists() {
                anyhow::bail!("PLSDB directory missing sequences.fasta: {}", plsdb_dir.display());
            }
            state.complete_step("plsdb_download");
            state.save(&state_path)?;

            // Step 5: Load PLSDB metadata
            eprintln!("[5/9] Loading PLSDB metadata...");
            let plasmids = self.load_plsdb_plasmids(&plsdb_dir)?;
            eprintln!("    Standalone circular+complete plasmids: {} (not in any assembly)",
                     plasmids.len());
            plasmids
        } else {
            // Download from server
            if !state.is_completed("plsdb_download") {
                state.update_step("plsdb_download");
                state.save(&state_path)?;
                eprintln!("[4/9] Downloading PLSDB database...");
                self.download_plsdb(&plsdb_dir)?;
                state.complete_step("plsdb_download");
                state.save(&state_path)?;
            } else {
                eprintln!("[4/9] PLSDB database already downloaded, skipping...");
            }

            // Step 5: Load PLSDB metadata (standalone plasmids only)
            eprintln!("[5/9] Loading PLSDB metadata...");
            let plasmids = self.load_plsdb_plasmids(&plsdb_dir)?;
            eprintln!("    Standalone circular+complete plasmids: {} (not in any assembly)",
                     plasmids.len());
            plasmids
        };

        // Add PLSDB plasmids to catalog (with taxonomy-based genus)
        for plasmid in &standalone_plasmids {
            catalog.add_plasmid_with_taxonomy(plasmid, taxonomy.as_ref());
        }

        // Step 6: Save unified catalog
        eprintln!("[6/9] Saving unified genome catalog...");
        catalog.save(&catalog_path)?;
        eprintln!("    Catalog entries: {}", catalog.len());
        eprintln!("    Saved to: {}", catalog_path.display());

        // Step 7-9: Batch pipeline (download+align per batch, no combined FASTA)
        // Saves ~350GB disk by avoiding combined FASTA
        let paf_output = self.output_dir.join("all_alignments.paf");
        let merged_hits = self.output_dir.join("merged_alignment_hits.tsv");
        let output_tsv = self.output_dir.join("all_flanking_sequences.tsv");

        if !state.is_completed("alignment_flanking") {
            // Step 7: Download NCBI genomes and align in batches
            state.update_step("genome_download");
            state.save(&state_path)?;

            eprintln!("[7/9] Batch processing: Download + Align NCBI genomes...");
            eprintln!("    Producer-consumer pattern with {} GB queue buffer", self.config.queue_buffer_gb);

            let downloaded = self.download_and_align_batches(
                &assemblies,
                &genomes_dir,
                &temp_dir,
                &paf_output,
                &catalog,
            )?;
            state.genomes_downloaded = downloaded;
            state.complete_step("genome_download");
            state.save(&state_path)?;
            eprintln!("    Processed {} genomes", downloaded);

            // Step 8: Process PLSDB sequences (batch align)
            if self.config.plsdb.skip {
                eprintln!("[8/9] PLSDB processing skipped (--skip-plsdb)");
                state.plsdb_extracted = 0;
            } else {
                state.update_step("plsdb_extract");
                state.save(&state_path)?;
                eprintln!("[8/9] Processing PLSDB sequences...");
                let plsdb_count = self.align_plsdb(
                    &standalone_plasmids,
                    &plsdb_dir,
                    &temp_dir,
                    &paf_output,
                )?;
                state.plsdb_extracted = plsdb_count;
                state.complete_step("plsdb_extract");
                state.save(&state_path)?;
            }

            // Step 9: Convert PAF to merged format and extract flanking
            state.update_step("alignment_flanking");
            state.save(&state_path)?;

            // Step 9a: Convert PAF to merged hits format
            eprintln!("[9/9] Processing alignment results...");
            self.convert_paf_to_merged(&paf_output, &merged_hits)?;
            state.alignments_done = true;
            state.save(&state_path)?;

            // Step 9b: Extract flanking sequences from individual genome files
            eprintln!("    Extracting flanking sequences (flanking_length: {} bp)...",
                self.config.flanking_length);
            self.extract_flanking_sequences(
                &merged_hits,
                &genomes_dir,
                &output_tsv,
                &catalog,
            )?;
            // Also extract from PLSDB (if not skipped)
            if !self.config.plsdb.skip {
                self.extract_flanking_from_plsdb(
                    &merged_hits,
                    &plsdb_dir,
                    &output_tsv,
                    &catalog,
                )?;
            }
            state.flanking_extracted = true;
            state.complete_step("alignment_flanking");
            state.save(&state_path)?;

            // Cleanup: only delete temp files on success (keep genome files for resume)
            eprintln!("    Cleaning up PAF file...");
            std::fs::remove_file(&paf_output).ok();
        } else {
            eprintln!("[7-9] Batch pipeline already completed, skipping...");
        }

        // Cleanup temp directory
        if temp_dir.exists() {
            eprintln!("\nCleaning up temp files...");
            std::fs::remove_dir_all(&temp_dir).ok();
        }

        // Step 10: Build FDB from TSV (external sort + zstd compression)
        let fdb_path = self.output_dir.join("flanking.fdb");
        if !state.is_completed("fdb_build") && output_tsv.exists() {
            state.update_step("fdb_build");
            state.save(&state_path)?;
            eprintln!("\n[10/10] Building FDB from TSV (external sort + zstd)...");

            // Use 1GB buffer for external sort, all available threads
            let buffer_size_mb = 1024;
            crate::fdb::build(&output_tsv, &fdb_path, buffer_size_mb, self.threads)?;

            state.complete_step("fdb_build");
            state.save(&state_path)?;
        } else if fdb_path.exists() {
            eprintln!("[10/10] FDB already built, skipping...");
        }

        eprintln!("\n============================================================");
        eprintln!(" Build Complete");
        eprintln!("============================================================");
        eprintln!("Output files:");
        eprintln!("  - {}", output_tsv.display());
        if fdb_path.exists() {
            let fdb_size = std::fs::metadata(&fdb_path).map(|m| m.len()).unwrap_or(0);
            eprintln!("  - {} ({:.1} MB)", fdb_path.display(), fdb_size as f64 / 1024.0 / 1024.0);
        }
        eprintln!("  - {}", catalog_path.display());

        Ok(())
    }

    /// Download GenBank-only assembly summaries (bacteria + archaea)
    /// GenBank is a superset of RefSeq with 0 FTP NA entries
    fn download_assembly_summaries(&self) -> Result<Vec<AssemblyInfo>> {
        let mut assemblies = Vec::new();

        // Download GenBank only (no RefSeq deduplication needed)
        for kingdom in ["bacteria", "archaea"] {
            let url = format!(
                "{}/genbank/{}/assembly_summary.txt",
                NCBI_FTP_BASE, kingdom
            );
            eprintln!("    Downloading genbank/{}...", kingdom);

            match self.download_and_parse_assembly_summary(&url, None) {
                Ok((mut asm, _)) => {
                    eprintln!("      Found {} assemblies (Complete Genome + Chromosome)", asm.len());
                    assemblies.append(&mut asm);
                }
                Err(e) => {
                    eprintln!("      Warning: Failed to download {}: {}", url, e);
                }
            }
        }

        eprintln!("    Total assemblies: {}", assemblies.len());
        Ok(assemblies)
    }

    /// Download PLSDB meta and fasta files
    fn download_plsdb(&self, plsdb_dir: &Path) -> Result<()> {
        let meta_tar = plsdb_dir.join("meta.tar.gz");
        let fasta_path = plsdb_dir.join("sequences.fasta");

        // Download meta archive if needed
        if !plsdb_dir.join("nuccore.csv").exists() {
            eprintln!("    Downloading PLSDB metadata...");
            self.download_file(PLSDB_META_URL, &meta_tar)?;

            // Extract tar.gz
            eprintln!("    Extracting metadata...");
            let status = Command::new("tar")
                .args(["-xzf", meta_tar.to_str().unwrap(), "-C", plsdb_dir.to_str().unwrap()])
                .status()
                .with_context(|| "Failed to extract PLSDB metadata")?;

            if !status.success() {
                anyhow::bail!("tar extraction failed");
            }

            // Clean up tar file
            std::fs::remove_file(&meta_tar).ok();
        } else {
            eprintln!("    PLSDB metadata already exists, skipping download...");
        }

        // Download FASTA if needed
        if !fasta_path.exists() {
            eprintln!("    Downloading PLSDB sequences (~7GB)...");
            self.download_file(PLSDB_FASTA_URL, &fasta_path)?;
        } else {
            eprintln!("    PLSDB sequences already exist, skipping download...");
        }

        Ok(())
    }

    /// Download and extract NCBI taxdump
    fn download_taxdump(&self, taxonomy_dir: &Path) -> Result<()> {
        let tar_path = taxonomy_dir.join("taxdump.tar.gz");
        let names_path = taxonomy_dir.join("names.dmp");

        // Skip if already extracted
        if names_path.exists() {
            eprintln!("    Taxdump already exists, skipping download...");
            return Ok(());
        }

        std::fs::create_dir_all(taxonomy_dir)?;

        // Download taxdump
        eprintln!("    Downloading NCBI taxdump (~60MB)...");
        self.download_file(NCBI_TAXDUMP_URL, &tar_path)?;

        // Extract
        eprintln!("    Extracting taxdump...");
        let status = Command::new("tar")
            .args(["-xzf", tar_path.to_str().unwrap(), "-C", taxonomy_dir.to_str().unwrap()])
            .status()
            .with_context(|| "Failed to extract taxdump")?;

        if !status.success() {
            anyhow::bail!("tar extraction failed");
        }

        // Clean up tar file
        std::fs::remove_file(&tar_path).ok();

        Ok(())
    }

    /// Download a file from URL with retry
    fn download_file(&self, url: &str, output_path: &Path) -> Result<()> {
        for attempt in 0..3 {
            match self.download_file_once(url, output_path) {
                Ok(_) => return Ok(()),
                Err(e) if attempt < 2 => {
                    eprintln!("      Download failed (attempt {}): {}", attempt + 1, e);
                    eprintln!("      Retrying in 5 seconds...");
                    std::thread::sleep(Duration::from_secs(5));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Download a file from URL (single attempt)
    fn download_file_once(&self, url: &str, output_path: &Path) -> Result<()> {
        let response = ureq::get(url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(7200)) // 2 hour timeout for dynamically generated files
            .call()
            .with_context(|| format!("Failed to download {}", url))?;

        let mut file = File::create(output_path)?;
        let mut reader = response.into_reader();

        // Stream download with progress
        let mut buffer = [0u8; 65536];
        let mut total = 0usize;
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    file.write_all(&buffer[..n])?;
                    total += n;
                    if total.is_multiple_of(100 * 1024 * 1024) {
                        eprintln!("      Downloaded {} MB...", total / (1024 * 1024));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        eprintln!("      Total: {} MB", total / (1024 * 1024));

        Ok(())
    }

    /// Download and parse assembly_summary.txt (streaming for large files)
    /// Returns (assemblies, paired_gca_accessions)
    /// If exclude_set is provided, skip assemblies in that set (for GenBank deduplication)
    fn download_and_parse_assembly_summary(
        &self,
        url: &str,
        exclude_set: Option<&FxHashSet<String>>,
    ) -> Result<(Vec<AssemblyInfo>, Vec<String>)> {
        let response = ureq::get(url)
            .timeout(Duration::from_secs(600))
            .call()
            .with_context(|| format!("Failed to download {}", url))?;

        let reader = BufReader::new(response.into_reader());
        let mut assemblies = Vec::new();
        let mut paired_gca = Vec::new();
        let mut skipped_duplicates = 0usize;

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 20 {
                continue;
            }

            let accession = fields[0];
            let assembly_level = fields[11];

            // Only include Complete Genome and Chromosome level
            if assembly_level != "Complete Genome" && assembly_level != "Chromosome" {
                continue;
            }

            let ftp_path = fields[19];
            if ftp_path == "na" || ftp_path.is_empty() {
                continue;
            }

            // Check for deduplication (GenBank vs RefSeq)
            if let Some(exclude) = exclude_set {
                // Strip version for matching (GCA_000005845.2 -> GCA_000005845)
                let acc_base = accession.split('.').next().unwrap_or(accession);
                if exclude.contains(accession) || exclude.contains(acc_base) {
                    skipped_duplicates += 1;
                    continue;
                }
            }

            // For RefSeq, collect paired GCA accessions with identical status
            // Field 17: gbrs_paired_asm (e.g., GCA_000005845.2)
            // Field 18: paired_asm_comp (identical, different, na)
            if fields.len() > 18 && accession.starts_with("GCF_") {
                let paired_asm = fields[17];
                let paired_comp = fields[18];
                if paired_asm != "na" && !paired_asm.is_empty() && paired_comp == "identical" {
                    paired_gca.push(paired_asm.to_string());
                    // Also add without version
                    if let Some(base) = paired_asm.split('.').next() {
                        paired_gca.push(base.to_string());
                    }
                }
            }

            assemblies.push(AssemblyInfo {
                accession: accession.to_string(),
                taxid: fields[5].to_string(),
                species_taxid: fields[6].to_string(),
                organism_name: fields[7].to_string(),
            });
        }

        if skipped_duplicates > 0 {
            eprintln!("      Skipped {} duplicates (have identical RefSeq)", skipped_duplicates);
        }

        Ok((assemblies, paired_gca))
    }

    /// Load PLSDB plasmids (circular + complete only)
    fn load_plsdb_plasmids(&self, plsdb_dir: &Path) -> Result<Vec<PlasmidInfo>> {
        let nuccore_path = plsdb_dir.join("nuccore.csv");
        let taxonomy_path = plsdb_dir.join("taxonomy.csv");

        // Load taxonomy mapping
        let mut taxonomy: FxHashMap<String, (String, String)> = FxHashMap::default();
        if taxonomy_path.exists() {
            let tax_file = File::open(&taxonomy_path)?;
            let tax_reader = BufReader::new(tax_file);

            for (idx, line) in tax_reader.lines().enumerate() {
                let line = line?;
                if idx == 0 {
                    continue; // Skip header
                }

                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() >= 10 {
                    let uid = fields[0].to_string();
                    let genus = fields[8].to_string();
                    let species = fields[9].to_string();
                    taxonomy.insert(uid, (genus, species));
                }
            }
        }

        // Load nuccore and filter
        let mut plasmids = Vec::new();
        let nuc_file = File::open(&nuccore_path)?;
        let nuc_reader = BufReader::new(nuc_file);

        for (idx, line) in nuc_reader.lines().enumerate() {
            let line = line?;
            if idx == 0 {
                continue; // Skip header
            }

            // Parse CSV with quotes handling
            let fields = parse_csv_line(&line);
            if fields.len() < 15 {
                continue;
            }

            let completeness = &fields[4];
            let topology = &fields[14];
            let assembly_uid = &fields[9];  // ASSEMBLY_UID

            // Filter: circular + complete only
            if completeness != "complete" {
                continue;
            }
            if topology != "circular" {
                continue;
            }

            // Filter: standalone plasmids only (not linked to any assembly)
            // Plasmids with ASSEMBLY_UID != "-1" are already included in GenBank assemblies
            if assembly_uid != "-1" {
                continue;
            }

            let taxonomy_uid = &fields[12];
            let (genus, species) = taxonomy.get(taxonomy_uid)
                .cloned()
                .unwrap_or_else(|| ("Unknown".to_string(), "Unknown".to_string()));

            plasmids.push(PlasmidInfo {
                accession: fields[1].clone(),
                taxonomy_uid: taxonomy_uid.clone(),
                genus,
                species,
            });
        }

        Ok(plasmids)
    }

    /// Batch processing: Download and align in batches without combined FASTA
    /// Producer-consumer pattern with backpressure based on queue_buffer_gb
    /// Saves ~350GB disk by avoiding combined FASTA
    fn download_and_align_batches(
        &self,
        assemblies: &[AssemblyInfo],
        genomes_dir: &Path,
        temp_dir: &Path,
        paf_output: &Path,
        _catalog: &GenomeCatalog,
    ) -> Result<usize> {
        let all_accessions_path = temp_dir.join("accessions_all.txt");
        let done_accessions_path = temp_dir.join("accessions_done.txt");

        // Step 1: Save all accessions if not exists
        let accessions: Vec<String> = assemblies.iter()
            .map(|a| a.accession.clone())
            .collect();

        if !all_accessions_path.exists() {
            eprintln!("    Saving {} accessions to temp file...", accessions.len());
            let mut file = File::create(&all_accessions_path)?;
            for acc in &accessions {
                writeln!(file, "{}", acc)?;
            }
        }

        // Step 2: Load already downloaded accessions and verify PAF integrity
        let mut done_set: FxHashSet<String> = FxHashSet::default();
        if done_accessions_path.exists() {
            let file = File::open(&done_accessions_path)?;
            let reader = BufReader::new(file);
            for acc in reader.lines().map_while(Result::ok) {
                done_set.insert(acc.trim().to_string());
            }

            // Verify PAF file integrity on resume
            if !done_set.is_empty() {
                if !paf_output.exists() {
                    eprintln!("    WARNING: PAF file missing but {} genomes marked as done", done_set.len());
                    eprintln!("    Clearing done list and reprocessing all genomes...");
                    done_set.clear();
                    std::fs::remove_file(&done_accessions_path).ok();
                } else {
                    // Verify PAF has content (basic sanity check)
                    let paf_size = std::fs::metadata(paf_output)?.len();
                    if paf_size == 0 {
                        eprintln!("    WARNING: PAF file is empty but {} genomes marked as done", done_set.len());
                        eprintln!("    Clearing done list and reprocessing all genomes...");
                        done_set.clear();
                        std::fs::remove_file(&done_accessions_path).ok();
                    } else {
                        // Verify genome files exist for done accessions
                        let mut missing_genomes = Vec::new();
                        for acc in &done_set {
                            let genome_path = genomes_dir.join(format!("{}.fna", acc));
                            if !genome_path.exists() {
                                missing_genomes.push(acc.clone());
                            }
                        }
                        if !missing_genomes.is_empty() {
                            eprintln!("    WARNING: {} genome files missing for done accessions", missing_genomes.len());
                            for acc in &missing_genomes {
                                done_set.remove(acc);
                            }
                            eprintln!("    Removed missing genomes from done list, will re-download");
                        }

                        if !done_set.is_empty() {
                            eprintln!("    Resume: {} already processed (PAF: {} MB)",
                                     done_set.len(), paf_size / 1024 / 1024);
                        }
                    }
                }
            }
        }

        // Step 3: Filter remaining accessions
        let remaining: Vec<&String> = accessions.iter()
            .filter(|a| !done_set.contains(*a))
            .collect();

        if remaining.is_empty() {
            eprintln!("    All {} genomes already processed", accessions.len());
            return Ok(accessions.len());
        }

        eprintln!("    Remaining: {}/{} genomes to process", remaining.len(), accessions.len());

        // Step 4: Calculate channel capacity based on queue_buffer_gb
        // Each batch ~3GB (1000 genomes × 3MB avg), queue_buffer_gb=30 → ~10 batches
        let bytes_per_batch = API_BATCH_SIZE * 3 * 1024 * 1024; // ~3GB per batch
        let queue_capacity = std::cmp::max(
            1,
            (self.config.queue_buffer_gb as usize * 1024 * 1024 * 1024) / bytes_per_batch
        );
        eprintln!("    Queue capacity: {} batches (based on {} GB buffer)", queue_capacity, self.config.queue_buffer_gb);

        // Step 5: Setup producer-consumer with bounded channel
        let batches: Vec<Vec<String>> = remaining.chunks(API_BATCH_SIZE)
            .map(|chunk| chunk.iter().map(|s| (*s).clone()).collect())
            .collect();
        let total_batches = batches.len();

        // Batch info: (batch_idx, genome_files)
        let (tx, rx) = mpsc::sync_channel::<(usize, Vec<PathBuf>)>(queue_capacity);

        // Shared state for progress tracking
        let downloaded = Arc::new(AtomicUsize::new(done_set.len()));
        let aligned = Arc::new(AtomicUsize::new(done_set.len()));
        let download_error = Arc::new(AtomicBool::new(false));

        // Clone necessary data for producer thread
        let genomes_dir = genomes_dir.to_path_buf();
        let temp_dir_producer = temp_dir.to_path_buf();
        let temp_dir_consumer = temp_dir.to_path_buf();
        let email = self.email.clone();
        let downloaded_clone = Arc::clone(&downloaded);
        let download_error_clone = Arc::clone(&download_error);
        let total_accessions = accessions.len();

        // Producer thread: downloads batches and sends to channel
        let producer = thread::spawn(move || -> Result<()> {
            for (batch_idx, batch) in batches.into_iter().enumerate() {
                if download_error_clone.load(Ordering::Relaxed) {
                    break;
                }

                let zip_path = temp_dir_producer.join(format!("batch_{:04}.zip", batch_idx));

                // Download batch via NCBI Datasets API
                let url = format!("{}/genome/download", NCBI_DATASETS_API);
                let acc_list: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();
                let request_body = serde_json::json!({
                    "accessions": acc_list,
                    "include_annotation_type": ["GENOME_FASTA"]
                });

                let response = ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .set("ncbi-client-id", &email)
                    .timeout(Duration::from_secs(3600))
                    .send_json(&request_body);

                match response {
                    Ok(resp) => {
                        // Save ZIP file
                        let mut zip_file = File::create(&zip_path)?;
                        let mut reader = resp.into_reader();
                        std::io::copy(&mut reader, &mut zip_file)?;
                        drop(zip_file);

                        // Extract genomes to individual files
                        let genome_files = Self::extract_batch_to_files_static(&zip_path, &genomes_dir)?;

                        // Clean up ZIP
                        std::fs::remove_file(&zip_path).ok();

                        downloaded_clone.fetch_add(batch.len(), Ordering::Relaxed);
                        eprintln!("    [Download] Batch {}/{}: {} genomes ({}/{})",
                                 batch_idx + 1, total_batches, batch.len(),
                                 downloaded_clone.load(Ordering::Relaxed), total_accessions);

                        // Send to consumer (blocks if channel is full - backpressure)
                        if tx.send((batch_idx, genome_files)).is_err() {
                            break; // Consumer dropped
                        }
                    }
                    Err(e) => {
                        eprintln!("    [Download] Batch {} failed: {}", batch_idx + 1, e);
                        download_error_clone.store(true, Ordering::Relaxed);
                        break;
                    }
                }

                // Rate limit between batches
                std::thread::sleep(Duration::from_millis(200));
            }
            Ok(())
        });

        // Consumer: receives batches, runs minimap2, appends to PAF
        let paf_file = Arc::new(Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(paf_output)?
        ));
        let done_file = Arc::new(Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&done_accessions_path)?
        ));

        let amr_db = self.amr_db_path.clone();
        let max_threads = self.threads;

        while let Ok((batch_idx, genome_files)) = rx.recv() {
            if genome_files.is_empty() {
                continue;
            }

            // Adaptive thread bucketing: group genomes by size category
            let buckets = bucket_genomes_by_size(&genome_files);
            let mut batch_hits: Vec<PafHit> = Vec::new();

            // Process each size category with appropriate thread count
            for (category, bucket_files) in &buckets {
                if bucket_files.is_empty() {
                    continue;
                }

                let bucket_threads = category.thread_count(max_threads);
                let bucket_suffix = format!("batch_{:04}_{:?}", batch_idx, category);
                let bucket_fasta = temp_dir_consumer.join(format!("{}.fas", bucket_suffix));

                // Create bucket FASTA with contig|filename headers
                {
                    let mut fasta_writer = BufWriter::new(File::create(&bucket_fasta)?);
                    for genome_path in bucket_files {
                        let filename = genome_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown.fna");

                        let file = File::open(genome_path)?;
                        let reader = BufReader::new(file);
                        for line in reader.lines() {
                            let line = line?;
                            if let Some(stripped) = line.strip_prefix('>') {
                                let contig_id = stripped.split_whitespace().next().unwrap_or(stripped);
                                writeln!(fasta_writer, ">{}|{}", contig_id, filename)?;
                            } else {
                                writeln!(fasta_writer, "{}", line)?;
                            }
                        }
                    }
                    fasta_writer.flush()?;
                }

                // Run minimap2 with adaptive thread count
                let bucket_paf = temp_dir_consumer.join(format!("{}.paf", bucket_suffix));
                let status = Command::new("minimap2")
                    .args([
                        "-cx", "asm20",
                        "-t", &bucket_threads.to_string(),
                        amr_db.to_str().unwrap(),
                        bucket_fasta.to_str().unwrap(),
                        "-o", bucket_paf.to_str().unwrap(),
                    ])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();

                if let Ok(s) = status {
                    if s.success() && bucket_paf.exists() {
                        // Parse PAF hits from bucket
                        let paf_content = std::fs::read_to_string(&bucket_paf)?;
                        let hits: Vec<PafHit> = paf_content
                            .lines()
                            .filter_map(PafHit::from_paf_line)
                            .collect();
                        batch_hits.extend(hits);
                    }
                }

                // Clean up bucket temp files
                std::fs::remove_file(&bucket_fasta).ok();
                std::fs::remove_file(&bucket_paf).ok();
            }

            // Deduplicate combined hits from all buckets
            let dedup_hits = deduplicate_paf_hits(batch_hits);

            // Append deduplicated hits to main PAF and flush immediately
            {
                let mut paf = paf_file.lock().unwrap();
                for hit in &dedup_hits {
                    writeln!(paf, "{}", hit.raw_line)?;
                }
                paf.flush()?;  // Ensure PAF is written before marking as done
            }

            // Record completed accessions (only after PAF is flushed)
            {
                let mut done = done_file.lock().unwrap();
                for genome_path in &genome_files {
                    if let Some(stem) = genome_path.file_stem().and_then(|s| s.to_str()) {
                        writeln!(done, "{}", stem)?;
                    }
                }
                done.flush()?;  // Ensure done list is persisted
            }

            let count = genome_files.len();
            aligned.fetch_add(count, Ordering::Relaxed);

            // Log with bucket distribution
            let bucket_info: Vec<String> = buckets.iter()
                .map(|(cat, files)| format!("{}={}", cat.name().split_whitespace().next().unwrap_or("?"), files.len()))
                .collect();
            eprintln!("    [Align] Batch {}/{}: {} genomes [{}] ({}/{})",
                     batch_idx + 1, total_batches, count,
                     bucket_info.join(", "),
                     aligned.load(Ordering::Relaxed), total_accessions);
        }

        // Wait for producer
        let _ = producer.join();

        Ok(aligned.load(Ordering::Relaxed))
    }

    /// Extract genomes from ZIP to individual files in genomes_dir
    fn extract_batch_to_files_static(zip_path: &Path, genomes_dir: &Path) -> Result<Vec<PathBuf>> {
        let zip_file = File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;
        let mut genome_files = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with("_genomic.fna") || name.ends_with("_genomic.fasta") {
                let parts: Vec<&str> = name.split('/').collect();
                if let Some(acc_dir) = parts.iter().find(|p| p.starts_with("GCA_") || p.starts_with("GCF_")) {
                    let filename = format!("{}.fna", acc_dir);
                    let output_path = genomes_dir.join(&filename);

                    // Extract to file
                    let mut content = Vec::new();
                    file.read_to_end(&mut content)?;
                    std::fs::write(&output_path, &content)?;

                    genome_files.push(output_path);
                }
            }
        }

        Ok(genome_files)
    }

    /// PLSDB processing: batch align PLSDB sequences
    fn align_plsdb(
        &self,
        plasmids: &[PlasmidInfo],
        plsdb_dir: &Path,
        temp_dir: &Path,
        paf_output: &Path,
    ) -> Result<usize> {
        let fasta_path = plsdb_dir.join("sequences.fasta");

        if !fasta_path.exists() {
            eprintln!("    Warning: sequences.fasta not found in PLSDB directory");
            return Ok(0);
        }

        // Build set of accessions to extract
        let target_accs: FxHashSet<_> = plasmids.iter()
            .map(|p| p.accession.clone())
            .collect();

        if target_accs.is_empty() {
            eprintln!("    No PLSDB plasmids to process");
            return Ok(0);
        }

        // Process in batches
        let acc_list: Vec<_> = target_accs.iter().collect();
        let batches: Vec<_> = acc_list.chunks(API_BATCH_SIZE).collect();
        let total_batches = batches.len();
        let mut total_processed = 0usize;

        eprintln!("    Processing {} PLSDB plasmids in {} batches...",
                 target_accs.len(), total_batches);

        // Open source FASTA and build index for random access
        let file = File::open(&fasta_path)?;
        let reader = BufReader::new(file);

        // Build sequence map (memory intensive but simpler)
        let mut seq_map: FxHashMap<String, String> = FxHashMap::default();
        let mut current_acc: Option<String> = None;
        let mut current_seq = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('>') {
                if let Some(acc) = current_acc.take() {
                    if target_accs.contains(&acc) {
                        seq_map.insert(acc, std::mem::take(&mut current_seq));
                    }
                }
                let header = line.trim_start_matches('>');
                let acc = header.split_whitespace().next().unwrap_or("").to_string();
                current_acc = Some(acc);
                current_seq.clear();
            } else {
                current_seq.push_str(line.trim());
            }
        }
        if let Some(acc) = current_acc {
            if target_accs.contains(&acc) {
                seq_map.insert(acc, current_seq);
            }
        }

        eprintln!("    Loaded {} target sequences from PLSDB", seq_map.len());

        // Process batches with adaptive thread bucketing
        for (batch_idx, batch) in batches.iter().enumerate() {
            // Bucket sequences by size for adaptive thread allocation
            let mut size_buckets: FxHashMap<GenomeSizeCategory, Vec<(&str, &str)>> = FxHashMap::default();

            for acc in *batch {
                if let Some(seq) = seq_map.get(*acc) {
                    // Use sequence length as size estimate (1 byte per base)
                    let category = GenomeSizeCategory::from_size(seq.len() as u64);
                    size_buckets.entry(category)
                        .or_default()
                        .push((acc.as_str(), seq.as_str()));
                }
            }

            let mut batch_hits: Vec<PafHit> = Vec::new();

            // Process each size category with appropriate thread count
            for (category, bucket_seqs) in &size_buckets {
                if bucket_seqs.is_empty() {
                    continue;
                }

                let bucket_threads = category.thread_count(self.threads);
                let bucket_suffix = format!("plsdb_batch_{:04}_{:?}", batch_idx, category);
                let bucket_fasta = temp_dir.join(format!("{}.fas", bucket_suffix));

                // Create bucket FASTA with contig|filename headers
                {
                    let mut fasta_writer = BufWriter::new(File::create(&bucket_fasta)?);
                    for (acc, seq) in bucket_seqs {
                        let filename = format!("{}.fna", acc.replace('.', "_"));
                        writeln!(fasta_writer, ">{}|{}", acc, filename)?;
                        writeln!(fasta_writer, "{}", seq)?;
                    }
                    fasta_writer.flush()?;
                }

                // Run minimap2 with adaptive thread count
                let bucket_paf = temp_dir.join(format!("{}.paf", bucket_suffix));
                let status = Command::new("minimap2")
                    .args([
                        "-cx", "asm20",
                        "-t", &bucket_threads.to_string(),
                        self.amr_db_path.to_str().unwrap(),
                        bucket_fasta.to_str().unwrap(),
                        "-o", bucket_paf.to_str().unwrap(),
                    ])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();

                if let Ok(s) = status {
                    if s.success() && bucket_paf.exists() {
                        // Parse PAF hits from bucket
                        let paf_content = std::fs::read_to_string(&bucket_paf)?;
                        let hits: Vec<PafHit> = paf_content
                            .lines()
                            .filter_map(PafHit::from_paf_line)
                            .collect();
                        batch_hits.extend(hits);
                    }
                }

                // Clean up bucket temp files
                std::fs::remove_file(&bucket_fasta).ok();
                std::fs::remove_file(&bucket_paf).ok();
            }

            // Deduplicate combined hits from all buckets
            let dedup_hits = deduplicate_paf_hits(batch_hits);

            // Append deduplicated hits to main PAF
            {
                let mut paf_file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(paf_output)?;
                for hit in &dedup_hits {
                    writeln!(paf_file, "{}", hit.raw_line)?;
                }
            }

            total_processed += batch.len();

            // Log with bucket distribution
            let bucket_info: Vec<String> = size_buckets.iter()
                .map(|(cat, seqs)| format!("{}={}", cat.name().split_whitespace().next().unwrap_or("?"), seqs.len()))
                .collect();
            eprintln!("    [PLSDB] Batch {}/{}: {} plasmids [{}] ({}/{})",
                     batch_idx + 1, total_batches, batch.len(),
                     bucket_info.join(", "),
                     total_processed, target_accs.len());
        }

        // Also write PLSDB sequences to plsdb_dir as individual files for flanking extraction
        for (acc, seq) in &seq_map {
            let out_path = plsdb_dir.join(format!("{}.fna", acc.replace('.', "_")));
            if !out_path.exists() {
                let mut out_file = File::create(&out_path)?;
                writeln!(out_file, ">{}", acc)?;
                writeln!(out_file, "{}", seq)?;
            }
        }

        eprintln!("    Aligned {} PLSDB sequences", total_processed);
        Ok(total_processed)
    }

    /// Extract flanking sequences from PLSDB files
    fn extract_flanking_from_plsdb(
        &self,
        hits_path: &Path,
        plsdb_dir: &Path,
        output_path: &Path,
        catalog: &GenomeCatalog,
    ) -> Result<()> {
        let hits_file = File::open(hits_path)?;
        let reader = BufReader::new(hits_file);

        // Group hits by genome file (only PLSDB files)
        let mut plsdb_hits: FxHashMap<String, Vec<(String, String, usize, usize)>> = FxHashMap::default();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            // Skip header
            if i == 0 && line.starts_with("gene") {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 4 {
                continue;
            }

            let gene = fields[0];
            let contig_file = fields[1];
            let start: usize = fields[2].parse().unwrap_or(0);
            let end: usize = fields[3].parse().unwrap_or(0);

            // Parse contig_id|filename format
            let (contig_id, genome_file) = if let Some(pipe_pos) = contig_file.rfind('|') {
                (contig_file[..pipe_pos].to_string(), contig_file[pipe_pos + 1..].to_string())
            } else {
                continue;
            };

            // Only process PLSDB files (NZ_*, CP*, etc.)
            if genome_file.starts_with("NZ_") || genome_file.starts_with("CP") ||
               genome_file.starts_with("AP") || genome_file.starts_with("NC_") {
                plsdb_hits.entry(genome_file)
                    .or_default()
                    .push((gene.to_string(), contig_id, start, end));
            }
        }

        if plsdb_hits.is_empty() {
            return Ok(());
        }

        // Append to output file
        let mut writer = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_path)?;

        let mut extracted = 0usize;

        for (genome_file, hits) in &plsdb_hits {
            let genome_path = plsdb_dir.join(genome_file);
            if !genome_path.exists() {
                continue;
            }

            // Load genome sequences
            let sequences = match self.load_genome_sequences(&genome_path) {
                Ok(seqs) => seqs,
                Err(_) => continue,
            };

            let seq_map: FxHashMap<&str, &str> = sequences.iter()
                .map(|(header, seq)| {
                    let key = header.split_whitespace().next().unwrap_or(header.as_str());
                    (key, seq.as_str())
                })
                .collect();

            let genome_acc = genome_file.trim_end_matches(".fna");
            let base_genus = catalog.get_genus(genome_acc).map(|s| s.to_string());

            for (gene, contig_id, start, end) in hits {
                let contig_seq = match seq_map.get(contig_id.as_str()) {
                    Some(seq) => *seq,
                    None => continue,
                };

                let contig_len = contig_seq.len();
                if *start >= contig_len || *end > contig_len || start >= end {
                    continue;
                }

                // Get genus
                let genus = base_genus.clone().unwrap_or_else(|| "Unknown".to_string());

                // Extract flanking sequences
                let upstream_start = start.saturating_sub(self.config.flanking_length);
                let upstream = &contig_seq[upstream_start..*start];

                let downstream_end = std::cmp::min(*end + self.config.flanking_length, contig_len);
                let downstream = &contig_seq[*end..downstream_end];

                writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        gene, contig_id, genus, start, end, upstream, downstream)?;
                extracted += 1;
            }
        }

        if extracted > 0 {
            eprintln!("    Extracted {} PLSDB flanking sequences", extracted);
        }
        Ok(())
    }

    /// Convert PAF to merged hits format (gene, contig|filename, start, end)
    fn convert_paf_to_merged(
        &self,
        paf_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        let mut hits: FxHashSet<(String, String, usize, usize)> = FxHashSet::default();

        // Process PAF: query=contig|file, target=AMR
        // Columns: 0=query, 2=qstart, 3=qend, 5=target
        if paf_path.exists() {
            let file = File::open(paf_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() < 12 {
                    continue;
                }
                let contig_file = fields[0].to_string(); // contig|filename
                let gene = fields[5].to_string();        // AMR gene
                let start: usize = fields[2].parse().unwrap_or(0);
                let end: usize = fields[3].parse().unwrap_or(0);
                hits.insert((gene, contig_file, start, end));
            }
        }

        // Write output
        let mut writer = BufWriter::new(File::create(output_path)?);
        for (gene, contig_file, start, end) in &hits {
            writeln!(writer, "{}\t{}\t{}\t{}", gene, contig_file, start, end)?;
        }

        eprintln!("    Found {} unique hits", hits.len());
        Ok(())
    }

    /// Extract flanking sequences from merged hits TSV
    /// Input format: gene\tcontig_file\tstart\tend (with header)
    /// contig_file format: contig_id|filename.fna
    fn extract_flanking_sequences(
        &self,
        hits_path: &Path,
        genomes_dir: &Path,
        output_path: &Path,
        catalog: &GenomeCatalog,
    ) -> Result<()> {
        // Skip if output already exists and is non-empty
        if output_path.exists() {
            let metadata = std::fs::metadata(output_path)?;
            if metadata.len() > 0 {
                eprintln!("    Flanking sequences file already exists ({} MB), skipping extraction...",
                         metadata.len() / (1024 * 1024));
                return Ok(());
            }
        }

        let hits_file = File::open(hits_path)?;
        let reader = BufReader::new(hits_file);

        // Group hits by genome file
        // Format: genome_file -> Vec<(gene, contig_id, start, end)>
        let mut genome_hits: FxHashMap<String, Vec<(String, String, usize, usize)>> = FxHashMap::default();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            // Skip header
            if i == 0 && line.starts_with("gene") {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 4 {
                continue;
            }

            let gene = fields[0];
            let contig_file = fields[1]; // format: contig_id|filename.fna
            let start: usize = fields[2].parse().unwrap_or(0);
            let end: usize = fields[3].parse().unwrap_or(0);

            // Parse contig_id|filename format
            let (contig_id, genome_file) = if let Some(pipe_pos) = contig_file.rfind('|') {
                (contig_file[..pipe_pos].to_string(), contig_file[pipe_pos + 1..].to_string())
            } else {
                // Fallback: try to extract genome file from contig name
                (contig_file.to_string(), extract_genome_file(contig_file))
            };

            genome_hits.entry(genome_file)
                .or_default()
                .push((gene.to_string(), contig_id, start, end));
        }

        // Process each genome and extract flanking sequences
        let output_file = File::create(output_path)?;
        let mut writer = BufWriter::new(output_file);

        // Write header
        writeln!(writer, "Gene\tContig\tGenus\tStart\tEnd\tUpstream\tDownstream")?;

        let total_genomes = genome_hits.len();
        let mut processed = 0;
        let mut genus_found = 0usize;
        let mut genus_unknown = 0usize;
        let mut sequences_extracted = 0usize;

        for (genome_file, hits) in &genome_hits {
            let genome_path = genomes_dir.join(genome_file);
            if !genome_path.exists() {
                continue;
            }

            // Load genome sequences
            let sequences = match self.load_genome_sequences(&genome_path) {
                Ok(seqs) => seqs,
                Err(_) => continue,
            };

            // Build a map for quick contig lookup
            let seq_map: FxHashMap<&str, &str> = sequences.iter()
                .map(|(header, seq)| {
                    let key = header.split_whitespace().next().unwrap_or(header.as_str());
                    (key, seq.as_str())
                })
                .collect();

            // Try to get genus from genome filename (accession)
            let genome_acc = genome_file.trim_end_matches(".fna");
            let base_genus = catalog.get_genus(genome_acc).map(|s| s.to_string());

            // Extract flanking for each hit
            for (gene, contig_id, start, end) in hits {
                // Look up contig sequence
                let contig_seq = match seq_map.get(contig_id.as_str()) {
                    Some(seq) => *seq,
                    None => continue,
                };

                let contig_len = contig_seq.len();

                // Bounds check
                if *start >= contig_len || *end > contig_len || start >= end {
                    continue;
                }

                let upstream_start = start.saturating_sub(self.config.flanking_length);
                let downstream_end = std::cmp::min(end + self.config.flanking_length, contig_len);

                let upstream = if *start > 0 && upstream_start < *start {
                    &contig_seq[upstream_start..*start]
                } else {
                    ""
                };

                let downstream = if *end < contig_len && *end < downstream_end {
                    &contig_seq[*end..downstream_end]
                } else {
                    ""
                };

                // Try multiple methods to get genus:
                // 1. From genome accession (base_genus from NCBI catalog)
                // 2. From contig accession (for PLSDB plasmids)
                let genus = if let Some(ref g) = base_genus {
                    genus_found += 1;
                    g.clone()
                } else if let Some(g) = catalog.get_genus(contig_id) {
                    genus_found += 1;
                    g.to_string()
                } else {
                    genus_unknown += 1;
                    "Unknown".to_string()
                };

                writeln!(
                    writer,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    gene, contig_id, genus, start, end, upstream, downstream
                )?;
                sequences_extracted += 1;
            }

            processed += 1;
            if processed % 10000 == 0 || processed == total_genomes {
                eprintln!("    Processed {}/{} genomes ({} sequences)",
                    processed, total_genomes, sequences_extracted);
            }
        }

        eprintln!("    Extracted {} flanking sequences from {} genomes",
            sequences_extracted, processed);
        eprintln!("    Genus resolved: {}, Unknown: {}", genus_found, genus_unknown);
        Ok(())
    }

    /// Load genome sequences from FASTA
    fn load_genome_sequences(&self, genome_path: &Path) -> Result<Vec<(String, String)>> {
        let file = File::open(genome_path)?;
        let reader = BufReader::new(file);

        let mut sequences = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_seq = String::new();

        for line in reader.lines() {
            let line = line?;

            if let Some(stripped) = line.strip_prefix('>') {
                if let Some(name) = current_name.take() {
                    sequences.push((name, std::mem::take(&mut current_seq)));
                }
                current_name = Some(stripped.to_string());
                current_seq.clear();
            } else {
                current_seq.push_str(line.trim());
            }
        }

        if let Some(name) = current_name {
            sequences.push((name, current_seq));
        }

        Ok(sequences)
    }
}

/// Parse CSV line handling quoted fields
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}

/// Extract genome filename from contig name
fn extract_genome_file(contig: &str) -> String {
    // Try to extract GCA/GCF accession from contig name
    if let Some(start) = contig.find("GC") {
        let remainder = &contig[start..];
        if let Some(end) = remainder.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
            return format!("{}.fna", &remainder[..end]);
        }
        return format!("{}.fna", remainder);
    }

    // For PLSDB accessions (NZ_*, CP*, etc.)
    let acc = contig.split_whitespace().next().unwrap_or(contig);
    format!("{}.fna", acc.replace('.', "_"))
}

/// Builds the flanking sequence database.
///
/// This is the main entry point for flanking database construction.
/// Downloads genomes, aligns ARGs, and extracts flanking sequences.
///
/// # Arguments
/// * `output_dir` - Directory for output files
/// * `arg_db` - Path to ARG reference database (AMR_NCBI.fas or .mmi)
/// * `threads` - Number of threads for parallel processing
/// * `email` - NCBI API email for higher rate limits
/// * `config` - Flanking build configuration (flanking_length, queue_buffer_gb, plsdb)
///
/// # Output Files
/// - `genome_catalogue.tsv`: Accession → genus mapping
/// - `flanking_db.tsv`: Raw flanking data
/// - `flanking.fdb`: Compressed database
pub fn build(output_dir: &Path, arg_db: &Path, threads: usize, email: &str, config: FlankBuildConfig) -> Result<()> {
    let builder = FlankingDbBuilder::new(arg_db, output_dir, threads, email, config);
    builder.build()
}
