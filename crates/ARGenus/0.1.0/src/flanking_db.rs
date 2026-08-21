
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

const API_BATCH_SIZE: usize = 1000;

#[derive(Clone, Debug, Default)]
pub struct PlsdbOptions {

    pub dir: Option<PathBuf>,

    pub skip: bool,
}

#[derive(Clone, Debug)]
pub struct FlankBuildConfig {

    pub flanking_length: usize,

    pub queue_buffer_gb: u32,

    pub plsdb: PlsdbOptions,
}

impl Default for FlankBuildConfig {
    fn default() -> Self {
        Self {
            flanking_length: 1050,
            queue_buffer_gb: 30,
            plsdb: PlsdbOptions::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssemblyInfo {
    pub accession: String,
    pub taxid: String,
    pub species_taxid: String,
    pub organism_name: String,
}

#[derive(Clone, Debug)]
pub struct PlasmidInfo {
    pub accession: String,
    pub taxonomy_uid: String,
    pub genus: String,
    pub species: String,
}

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

fn compare_paf_hits(a: &PafHit, b: &PafHit) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    b.score.cmp(&a.score)
        .then_with(|| b.gene_length.cmp(&a.gene_length))
        .then_with(|| b.mapq.cmp(&a.mapq))
        .then_with(|| a.divergence.partial_cmp(&b.divergence).unwrap_or(Ordering::Equal))
        .then_with(|| a.gap_count.cmp(&b.gap_count))
        .then_with(|| a.gene_name.cmp(&b.gene_name))
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenomeSizeCategory {

    Small,

    Medium,

    Large,
}

const SMALL_GENOME_THRESHOLD: u64 = 5 * 1024 * 1024;
const LARGE_GENOME_THRESHOLD: u64 = 20 * 1024 * 1024;

impl GenomeSizeCategory {

    pub fn from_size(size_bytes: u64) -> Self {
        if size_bytes < SMALL_GENOME_THRESHOLD {
            GenomeSizeCategory::Small
        } else if size_bytes < LARGE_GENOME_THRESHOLD {
            GenomeSizeCategory::Medium
        } else {
            GenomeSizeCategory::Large
        }
    }

    pub fn thread_count(&self, max_threads: usize) -> usize {
        let recommended = match self {
            GenomeSizeCategory::Small => 4,
            GenomeSizeCategory::Medium => 6,
            GenomeSizeCategory::Large => 8,
        };

        recommended.min(max_threads)
    }

    pub fn name(&self) -> &'static str {
        match self {
            GenomeSizeCategory::Small => "small (<5MB)",
            GenomeSizeCategory::Medium => "medium (5-20MB)",
            GenomeSizeCategory::Large => "large (>20MB)",
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct CatalogEntry {
    pub accession: String,
    pub taxid: String,
    pub species_taxid: String,
    pub genus: String,
    pub species: String,
    pub organism_name: String,
    pub source: String,
}

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

    pub fn add_assembly_with_taxonomy(
        &mut self,
        asm: &AssemblyInfo,
        source: &str,
        taxonomy: Option<&TaxonomyDB>,
    ) {
        let (parsed_genus, species) = parse_organism_name(&asm.organism_name);

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

        self.entries.insert(asm.accession.clone(), entry.clone());
        if let Some(base) = asm.accession.split('.').next() {
            self.entries.insert(base.to_string(), entry);
        }
    }

    pub fn add_plasmid_with_taxonomy(
        &mut self,
        plasmid: &PlasmidInfo,
        taxonomy: Option<&TaxonomyDB>,
    ) {

        let organism_name = format!("{} {}", plasmid.genus, plasmid.species);

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

        self.entries.insert(plasmid.accession.clone(), entry.clone());

        let base = plasmid.accession
            .strip_prefix("NZ_")
            .unwrap_or(&plasmid.accession);
        let no_ver = base.split('.').next().unwrap_or(base);
        self.entries.insert(no_ver.to_string(), entry.clone());

        let underscore_key = plasmid.accession.replace('.', "_");
        self.entries.insert(underscore_key, entry);
    }

    pub fn get_genus(&self, accession: &str) -> Option<&str> {

        if let Some(entry) = self.entries.get(accession) {
            return Some(&entry.genus);
        }

        if let Some(base) = accession.split('.').next() {
            if let Some(entry) = self.entries.get(base) {
                return Some(&entry.genus);
            }
        }

        let stripped = accession.strip_prefix("NZ_").unwrap_or(accession);
        if let Some(entry) = self.entries.get(stripped) {
            return Some(&entry.genus);
        }
        let stripped_base = stripped.split('.').next().unwrap_or(stripped);
        if let Some(entry) = self.entries.get(stripped_base) {
            return Some(&entry.genus);
        }

        if let Some(last_underscore) = accession.rfind('_') {
            let suffix = &accession[last_underscore + 1..];

            if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                let with_dot = format!("{}.{}", &accession[..last_underscore], suffix);
                if let Some(entry) = self.entries.get(&with_dot) {
                    return Some(&entry.genus);
                }

                let stripped_with_dot = with_dot.strip_prefix("NZ_").unwrap_or(&with_dot);
                if let Some(entry) = self.entries.get(stripped_with_dot) {
                    return Some(&entry.genus);
                }
            }
        }

        None
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "accession\ttaxid\tspecies_taxid\tgenus\tspecies\torganism_name\tsource")?;

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

    pub fn len(&self) -> usize {
        let mut seen: FxHashSet<&str> = FxHashSet::default();
        for entry in self.entries.values() {
            seen.insert(&entry.accession);
        }
        seen.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn parse_organism_name(organism: &str) -> (String, String) {
    let parts: Vec<&str> = organism.split_whitespace().collect();

    let genus = parts.first()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let species = if parts.len() >= 2 {

        parts[1..].join(" ")
    } else {
        "Unknown".to_string()
    };

    (clean_genus(&genus), species)
}

fn clean_genus(genus: &str) -> String {

    let cleaned = if let Some(idx) = genus.find('(') {
        genus[..idx].trim()
    } else {
        genus.trim()
    };

    cleaned.to_string()
}

fn resolve_genus(
    organism_name: &str,
    taxid_str: &str,
    _parsed_genus: &str,
    taxonomy: Option<&TaxonomyDB>,
) -> String {

    let organism_lower = organism_name.to_lowercase();
    if organism_lower.contains("uncultured") {
        return "uncultured".to_string();
    }

    if let Some(tax_db) = taxonomy {
        if let Ok(taxid) = taxid_str.parse::<u32>() {

            if let Some(genus) = tax_db.get_genus(taxid) {
                return clean_genus(&genus);
            }

            if let Some((name, _rank)) = tax_db.get_genus_or_higher(taxid) {
                return clean_genus(&name);
            }
        }
    }

    let cleaned = clean_genus(organism_name);
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned
    }
}

pub struct TaxonomyDB {

    names: FxHashMap<u32, String>,

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

    pub fn load(taxdump_dir: &Path) -> Result<Self> {
        let mut db = Self::new();

        let names_path = taxdump_dir.join("names.dmp");
        let nodes_path = taxdump_dir.join("nodes.dmp");

        if !names_path.exists() || !nodes_path.exists() {
            anyhow::bail!("Taxonomy files not found in {}", taxdump_dir.display());
        }

        eprintln!("    Loading names.dmp...");
        let file = File::open(&names_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split("\t|\t").collect();
            if fields.len() < 4 {
                continue;
            }

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

    pub fn get_genus(&self, taxid: u32) -> Option<String> {
        let mut current = taxid;
        let mut visited = 0;

        while visited < 50 {
            if let Some((parent, rank)) = self.nodes.get(&current) {
                if rank == "genus" {
                    return self.names.get(&current).cloned();
                }

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

    pub fn get_genus_or_higher(&self, taxid: u32) -> Option<(String, String)> {
        let mut current = taxid;
        let mut visited = 0;

        let target_ranks = ["genus", "family", "order", "class", "phylum"];
        let mut best_match: Option<(String, String, usize)> = None;

        while visited < 50 {
            if let Some((parent, rank)) = self.nodes.get(&current) {

                if let Some(priority) = target_ranks.iter().position(|r| r == rank) {
                    if let Some(name) = self.names.get(&current) {

                        if rank == "genus" {
                            return Some((name.clone(), rank.clone()));
                        }

                        if best_match.is_none() || priority < best_match.as_ref().unwrap().2 {
                            best_match = Some((name.clone(), rank.clone(), priority));
                        }
                    }
                }

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

pub struct FlankingDbBuilder {
    output_dir: PathBuf,
    threads: usize,
    amr_db_path: PathBuf,

    email: String,

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

        let plsdb_dir = self.config.plsdb.dir.clone().unwrap_or_else(|| self.output_dir.join("plsdb"));
        if !self.config.plsdb.skip {
            std::fs::create_dir_all(&plsdb_dir)?;
        }
        let taxonomy_dir = self.output_dir.join("taxonomy");
        let temp_dir = self.output_dir.join("temp");
        std::fs::create_dir_all(&temp_dir)?;
        let state_path = self.output_dir.join("build_state.json");

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

        let mut catalog = GenomeCatalog::new();
        let catalog_path = self.output_dir.join("genome_catalog.tsv");

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

        eprintln!("[3/9] Downloading NCBI assembly summaries (GenBank)...");
        let assemblies = self.download_assembly_summaries()?;
        eprintln!("    GenBank assemblies: {} (Complete Genome + Chromosome)", assemblies.len());

        for asm in &assemblies {
            catalog.add_assembly_with_taxonomy(asm, "genbank", taxonomy.as_ref());
        }

        let standalone_plasmids = if self.config.plsdb.skip {
            eprintln!("[4/9] PLSDB skipped (--skip-plsdb)");
            eprintln!("[5/9] PLSDB metadata skipped");
            Vec::new()
        } else if self.config.plsdb.dir.is_some() {

            eprintln!("[4/9] Using pre-downloaded PLSDB: {}", plsdb_dir.display());

            let nuccore_csv = plsdb_dir.join("nuccore.csv");
            let fasta_path = plsdb_dir.join("sequences.fasta");
            if !nuccore_csv.exists() {

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

            eprintln!("[5/9] Loading PLSDB metadata...");
            let plasmids = self.load_plsdb_plasmids(&plsdb_dir)?;
            eprintln!("    Standalone circular+complete plasmids: {} (not in any assembly)",
                     plasmids.len());
            plasmids
        } else {

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

            eprintln!("[5/9] Loading PLSDB metadata...");
            let plasmids = self.load_plsdb_plasmids(&plsdb_dir)?;
            eprintln!("    Standalone circular+complete plasmids: {} (not in any assembly)",
                     plasmids.len());
            plasmids
        };

        for plasmid in &standalone_plasmids {
            catalog.add_plasmid_with_taxonomy(plasmid, taxonomy.as_ref());
        }

        eprintln!("[6/9] Saving unified genome catalog...");
        catalog.save(&catalog_path)?;
        eprintln!("    Catalog entries: {}", catalog.len());
        eprintln!("    Saved to: {}", catalog_path.display());

        let paf_output = self.output_dir.join("all_alignments.paf");
        let merged_hits = self.output_dir.join("merged_alignment_hits.tsv");
        let output_tsv = self.output_dir.join("all_flanking_sequences.tsv");

        if !state.is_completed("alignment_flanking") {

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

            state.update_step("alignment_flanking");
            state.save(&state_path)?;

            eprintln!("[9/9] Processing alignment results...");
            self.convert_paf_to_merged(&paf_output, &merged_hits)?;
            state.alignments_done = true;
            state.save(&state_path)?;

            eprintln!("    Extracting flanking sequences (flanking_length: {} bp)...",
                self.config.flanking_length);
            self.extract_flanking_sequences(
                &merged_hits,
                &genomes_dir,
                &output_tsv,
                &catalog,
            )?;

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

            eprintln!("    Cleaning up PAF file...");
            std::fs::remove_file(&paf_output).ok();
        } else {
            eprintln!("[7-9] Batch pipeline already completed, skipping...");
        }

        if temp_dir.exists() {
            eprintln!("\nCleaning up temp files...");
            std::fs::remove_dir_all(&temp_dir).ok();
        }

        let fdb_path = self.output_dir.join("flanking.fdb");
        if !state.is_completed("fdb_build") && output_tsv.exists() {
            state.update_step("fdb_build");
            state.save(&state_path)?;
            eprintln!("\n[10/10] Building FDB from TSV (external sort + zstd)...");

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

    fn download_assembly_summaries(&self) -> Result<Vec<AssemblyInfo>> {
        let mut assemblies = Vec::new();

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

    fn download_plsdb(&self, plsdb_dir: &Path) -> Result<()> {
        let meta_tar = plsdb_dir.join("meta.tar.gz");
        let fasta_path = plsdb_dir.join("sequences.fasta");

        if !plsdb_dir.join("nuccore.csv").exists() {
            eprintln!("    Downloading PLSDB metadata...");
            self.download_file(PLSDB_META_URL, &meta_tar)?;

            eprintln!("    Extracting metadata...");
            let status = Command::new("tar")
                .args(["-xzf", meta_tar.to_str().unwrap(), "-C", plsdb_dir.to_str().unwrap()])
                .status()
                .with_context(|| "Failed to extract PLSDB metadata")?;

            if !status.success() {
                anyhow::bail!("tar extraction failed");
            }

            std::fs::remove_file(&meta_tar).ok();
        } else {
            eprintln!("    PLSDB metadata already exists, skipping download...");
        }

        if !fasta_path.exists() {
            eprintln!("    Downloading PLSDB sequences (~7GB)...");
            self.download_file(PLSDB_FASTA_URL, &fasta_path)?;
        } else {
            eprintln!("    PLSDB sequences already exist, skipping download...");
        }

        Ok(())
    }

    fn download_taxdump(&self, taxonomy_dir: &Path) -> Result<()> {
        let tar_path = taxonomy_dir.join("taxdump.tar.gz");
        let names_path = taxonomy_dir.join("names.dmp");

        if names_path.exists() {
            eprintln!("    Taxdump already exists, skipping download...");
            return Ok(());
        }

        std::fs::create_dir_all(taxonomy_dir)?;

        eprintln!("    Downloading NCBI taxdump (~60MB)...");
        self.download_file(NCBI_TAXDUMP_URL, &tar_path)?;

        eprintln!("    Extracting taxdump...");
        let status = Command::new("tar")
            .args(["-xzf", tar_path.to_str().unwrap(), "-C", taxonomy_dir.to_str().unwrap()])
            .status()
            .with_context(|| "Failed to extract taxdump")?;

        if !status.success() {
            anyhow::bail!("tar extraction failed");
        }

        std::fs::remove_file(&tar_path).ok();

        Ok(())
    }

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

    fn download_file_once(&self, url: &str, output_path: &Path) -> Result<()> {
        let response = ureq::get(url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(7200))
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

            if assembly_level != "Complete Genome" && assembly_level != "Chromosome" {
                continue;
            }

            let ftp_path = fields[19];
            if ftp_path == "na" || ftp_path.is_empty() {
                continue;
            }

            if let Some(exclude) = exclude_set {

                let acc_base = accession.split('.').next().unwrap_or(accession);
                if exclude.contains(accession) || exclude.contains(acc_base) {
                    skipped_duplicates += 1;
                    continue;
                }
            }

            if fields.len() > 18 && accession.starts_with("GCF_") {
                let paired_asm = fields[17];
                let paired_comp = fields[18];
                if paired_asm != "na" && !paired_asm.is_empty() && paired_comp == "identical" {
                    paired_gca.push(paired_asm.to_string());

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

    fn load_plsdb_plasmids(&self, plsdb_dir: &Path) -> Result<Vec<PlasmidInfo>> {
        let nuccore_path = plsdb_dir.join("nuccore.csv");
        let taxonomy_path = plsdb_dir.join("taxonomy.csv");

        let mut taxonomy: FxHashMap<String, (String, String)> = FxHashMap::default();
        if taxonomy_path.exists() {
            let tax_file = File::open(&taxonomy_path)?;
            let tax_reader = BufReader::new(tax_file);

            for (idx, line) in tax_reader.lines().enumerate() {
                let line = line?;
                if idx == 0 {
                    continue;
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

        let mut plasmids = Vec::new();
        let nuc_file = File::open(&nuccore_path)?;
        let nuc_reader = BufReader::new(nuc_file);

        for (idx, line) in nuc_reader.lines().enumerate() {
            let line = line?;
            if idx == 0 {
                continue;
            }

            let fields = parse_csv_line(&line);
            if fields.len() < 15 {
                continue;
            }

            let completeness = &fields[4];
            let topology = &fields[14];
            let assembly_uid = &fields[9];

            if completeness != "complete" {
                continue;
            }
            if topology != "circular" {
                continue;
            }

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

        let mut done_set: FxHashSet<String> = FxHashSet::default();
        if done_accessions_path.exists() {
            let file = File::open(&done_accessions_path)?;
            let reader = BufReader::new(file);
            for acc in reader.lines().map_while(Result::ok) {
                done_set.insert(acc.trim().to_string());
            }

            if !done_set.is_empty() {
                if !paf_output.exists() {
                    eprintln!("    WARNING: PAF file missing but {} genomes marked as done", done_set.len());
                    eprintln!("    Clearing done list and reprocessing all genomes...");
                    done_set.clear();
                    std::fs::remove_file(&done_accessions_path).ok();
                } else {

                    let paf_size = std::fs::metadata(paf_output)?.len();
                    if paf_size == 0 {
                        eprintln!("    WARNING: PAF file is empty but {} genomes marked as done", done_set.len());
                        eprintln!("    Clearing done list and reprocessing all genomes...");
                        done_set.clear();
                        std::fs::remove_file(&done_accessions_path).ok();
                    } else {

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

        let remaining: Vec<&String> = accessions.iter()
            .filter(|a| !done_set.contains(*a))
            .collect();

        if remaining.is_empty() {
            eprintln!("    All {} genomes already processed", accessions.len());
            return Ok(accessions.len());
        }

        eprintln!("    Remaining: {}/{} genomes to process", remaining.len(), accessions.len());

        let bytes_per_batch = API_BATCH_SIZE * 3 * 1024 * 1024;
        let queue_capacity = std::cmp::max(
            1,
            (self.config.queue_buffer_gb as usize * 1024 * 1024 * 1024) / bytes_per_batch
        );
        eprintln!("    Queue capacity: {} batches (based on {} GB buffer)", queue_capacity, self.config.queue_buffer_gb);

        let batches: Vec<Vec<String>> = remaining.chunks(API_BATCH_SIZE)
            .map(|chunk| chunk.iter().map(|s| (*s).clone()).collect())
            .collect();
        let total_batches = batches.len();

        let (tx, rx) = mpsc::sync_channel::<(usize, Vec<PathBuf>)>(queue_capacity);

        let downloaded = Arc::new(AtomicUsize::new(done_set.len()));
        let aligned = Arc::new(AtomicUsize::new(done_set.len()));
        let download_error = Arc::new(AtomicBool::new(false));

        let genomes_dir = genomes_dir.to_path_buf();
        let temp_dir_producer = temp_dir.to_path_buf();
        let temp_dir_consumer = temp_dir.to_path_buf();
        let email = self.email.clone();
        let downloaded_clone = Arc::clone(&downloaded);
        let download_error_clone = Arc::clone(&download_error);
        let total_accessions = accessions.len();

        let producer = thread::spawn(move || -> Result<()> {
            for (batch_idx, batch) in batches.into_iter().enumerate() {
                if download_error_clone.load(Ordering::Relaxed) {
                    break;
                }

                let zip_path = temp_dir_producer.join(format!("batch_{:04}.zip", batch_idx));

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

                        let mut zip_file = File::create(&zip_path)?;
                        let mut reader = resp.into_reader();
                        std::io::copy(&mut reader, &mut zip_file)?;
                        drop(zip_file);

                        let genome_files = Self::extract_batch_to_files_static(&zip_path, &genomes_dir)?;

                        std::fs::remove_file(&zip_path).ok();

                        downloaded_clone.fetch_add(batch.len(), Ordering::Relaxed);
                        eprintln!("    [Download] Batch {}/{}: {} genomes ({}/{})",
                                 batch_idx + 1, total_batches, batch.len(),
                                 downloaded_clone.load(Ordering::Relaxed), total_accessions);

                        if tx.send((batch_idx, genome_files)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("    [Download] Batch {} failed: {}", batch_idx + 1, e);
                        download_error_clone.store(true, Ordering::Relaxed);
                        break;
                    }
                }

                std::thread::sleep(Duration::from_millis(200));
            }
            Ok(())
        });

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

            let buckets = bucket_genomes_by_size(&genome_files);
            let mut batch_hits: Vec<PafHit> = Vec::new();

            for (category, bucket_files) in &buckets {
                if bucket_files.is_empty() {
                    continue;
                }

                let bucket_threads = category.thread_count(max_threads);
                let bucket_suffix = format!("batch_{:04}_{:?}", batch_idx, category);
                let bucket_fasta = temp_dir_consumer.join(format!("{}.fas", bucket_suffix));

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

                        let paf_content = std::fs::read_to_string(&bucket_paf)?;
                        let hits: Vec<PafHit> = paf_content
                            .lines()
                            .filter_map(PafHit::from_paf_line)
                            .collect();
                        batch_hits.extend(hits);
                    }
                }

                std::fs::remove_file(&bucket_fasta).ok();
                std::fs::remove_file(&bucket_paf).ok();
            }

            let dedup_hits = deduplicate_paf_hits(batch_hits);

            {
                let mut paf = paf_file.lock().unwrap();
                for hit in &dedup_hits {
                    writeln!(paf, "{}", hit.raw_line)?;
                }
                paf.flush()?;
            }

            {
                let mut done = done_file.lock().unwrap();
                for genome_path in &genome_files {
                    if let Some(stem) = genome_path.file_stem().and_then(|s| s.to_str()) {
                        writeln!(done, "{}", stem)?;
                    }
                }
                done.flush()?;
            }

            let count = genome_files.len();
            aligned.fetch_add(count, Ordering::Relaxed);

            let bucket_info: Vec<String> = buckets.iter()
                .map(|(cat, files)| format!("{}={}", cat.name().split_whitespace().next().unwrap_or("?"), files.len()))
                .collect();
            eprintln!("    [Align] Batch {}/{}: {} genomes [{}] ({}/{})",
                     batch_idx + 1, total_batches, count,
                     bucket_info.join(", "),
                     aligned.load(Ordering::Relaxed), total_accessions);
        }

        let _ = producer.join();

        Ok(aligned.load(Ordering::Relaxed))
    }

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

                    let mut content = Vec::new();
                    file.read_to_end(&mut content)?;
                    std::fs::write(&output_path, &content)?;

                    genome_files.push(output_path);
                }
            }
        }

        Ok(genome_files)
    }

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

        let target_accs: FxHashSet<_> = plasmids.iter()
            .map(|p| p.accession.clone())
            .collect();

        if target_accs.is_empty() {
            eprintln!("    No PLSDB plasmids to process");
            return Ok(0);
        }

        let acc_list: Vec<_> = target_accs.iter().collect();
        let batches: Vec<_> = acc_list.chunks(API_BATCH_SIZE).collect();
        let total_batches = batches.len();
        let mut total_processed = 0usize;

        eprintln!("    Processing {} PLSDB plasmids in {} batches...",
                 target_accs.len(), total_batches);

        let file = File::open(&fasta_path)?;
        let reader = BufReader::new(file);

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

        for (batch_idx, batch) in batches.iter().enumerate() {

            let mut size_buckets: FxHashMap<GenomeSizeCategory, Vec<(&str, &str)>> = FxHashMap::default();

            for acc in *batch {
                if let Some(seq) = seq_map.get(*acc) {

                    let category = GenomeSizeCategory::from_size(seq.len() as u64);
                    size_buckets.entry(category)
                        .or_default()
                        .push((acc.as_str(), seq.as_str()));
                }
            }

            let mut batch_hits: Vec<PafHit> = Vec::new();

            for (category, bucket_seqs) in &size_buckets {
                if bucket_seqs.is_empty() {
                    continue;
                }

                let bucket_threads = category.thread_count(self.threads);
                let bucket_suffix = format!("plsdb_batch_{:04}_{:?}", batch_idx, category);
                let bucket_fasta = temp_dir.join(format!("{}.fas", bucket_suffix));

                {
                    let mut fasta_writer = BufWriter::new(File::create(&bucket_fasta)?);
                    for (acc, seq) in bucket_seqs {
                        let filename = format!("{}.fna", acc.replace('.', "_"));
                        writeln!(fasta_writer, ">{}|{}", acc, filename)?;
                        writeln!(fasta_writer, "{}", seq)?;
                    }
                    fasta_writer.flush()?;
                }

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

                        let paf_content = std::fs::read_to_string(&bucket_paf)?;
                        let hits: Vec<PafHit> = paf_content
                            .lines()
                            .filter_map(PafHit::from_paf_line)
                            .collect();
                        batch_hits.extend(hits);
                    }
                }

                std::fs::remove_file(&bucket_fasta).ok();
                std::fs::remove_file(&bucket_paf).ok();
            }

            let dedup_hits = deduplicate_paf_hits(batch_hits);

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

            let bucket_info: Vec<String> = size_buckets.iter()
                .map(|(cat, seqs)| format!("{}={}", cat.name().split_whitespace().next().unwrap_or("?"), seqs.len()))
                .collect();
            eprintln!("    [PLSDB] Batch {}/{}: {} plasmids [{}] ({}/{})",
                     batch_idx + 1, total_batches, batch.len(),
                     bucket_info.join(", "),
                     total_processed, target_accs.len());
        }

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

    fn extract_flanking_from_plsdb(
        &self,
        hits_path: &Path,
        plsdb_dir: &Path,
        output_path: &Path,
        catalog: &GenomeCatalog,
    ) -> Result<()> {
        let hits_file = File::open(hits_path)?;
        let reader = BufReader::new(hits_file);

        let mut plsdb_hits: FxHashMap<String, Vec<(String, String, usize, usize)>> = FxHashMap::default();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;

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

            let (contig_id, genome_file) = if let Some(pipe_pos) = contig_file.rfind('|') {
                (contig_file[..pipe_pos].to_string(), contig_file[pipe_pos + 1..].to_string())
            } else {
                continue;
            };

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

                let genus = base_genus.clone().unwrap_or_else(|| "Unknown".to_string());

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

    fn convert_paf_to_merged(
        &self,
        paf_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        let mut hits: FxHashSet<(String, String, usize, usize)> = FxHashSet::default();

        if paf_path.exists() {
            let file = File::open(paf_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() < 12 {
                    continue;
                }
                let contig_file = fields[0].to_string();
                let gene = fields[5].to_string();
                let start: usize = fields[2].parse().unwrap_or(0);
                let end: usize = fields[3].parse().unwrap_or(0);
                hits.insert((gene, contig_file, start, end));
            }
        }

        let mut writer = BufWriter::new(File::create(output_path)?);
        for (gene, contig_file, start, end) in &hits {
            writeln!(writer, "{}\t{}\t{}\t{}", gene, contig_file, start, end)?;
        }

        eprintln!("    Found {} unique hits", hits.len());
        Ok(())
    }

    fn extract_flanking_sequences(
        &self,
        hits_path: &Path,
        genomes_dir: &Path,
        output_path: &Path,
        catalog: &GenomeCatalog,
    ) -> Result<()> {

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

        let mut genome_hits: FxHashMap<String, Vec<(String, String, usize, usize)>> = FxHashMap::default();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;

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

            let (contig_id, genome_file) = if let Some(pipe_pos) = contig_file.rfind('|') {
                (contig_file[..pipe_pos].to_string(), contig_file[pipe_pos + 1..].to_string())
            } else {

                (contig_file.to_string(), extract_genome_file(contig_file))
            };

            genome_hits.entry(genome_file)
                .or_default()
                .push((gene.to_string(), contig_id, start, end));
        }

        let output_file = File::create(output_path)?;
        let mut writer = BufWriter::new(output_file);

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

fn extract_genome_file(contig: &str) -> String {

    if let Some(start) = contig.find("GC") {
        let remainder = &contig[start..];
        if let Some(end) = remainder.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
            return format!("{}.fna", &remainder[..end]);
        }
        return format!("{}.fna", remainder);
    }

    let acc = contig.split_whitespace().next().unwrap_or(contig);
    format!("{}.fna", acc.replace('.', "_"))
}

pub fn build(output_dir: &Path, arg_db: &Path, threads: usize, email: &str, config: FlankBuildConfig) -> Result<()> {
    let builder = FlankingDbBuilder::new(arg_db, output_dir, threads, email, config);
    builder.build()
}
