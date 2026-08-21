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
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::process::Command;

use crate::snp::{self, SnpStatus};

const FDB_MAGIC: &[u8; 8] = b"FLANKDB\0";

// ============================================================================
// Embedded phylogenetic-context tables (compiled into the binary)
// ============================================================================
// The GTDB genus distance/lineage tables and the conformal calibration are embedded
// (zstd-compressed) so a deployed binary is self-contained — only the flanking DB (FDB)
// ships separately. The kernel constants (lambda, coherence radius, absent distance) are
// calibrated to THIS table's patristic scale, so binding them into one binary prevents an
// old-table/new-constant version mismatch. An external file of the same name next to the
// FDB still overrides (for in-place updates); the loader logs which source it used.
const EMB_GENUS_DIST: &[u8] = include_bytes!("embedded/genus_dist.tsv.zst");
const EMB_GENUS_LINEAGE: &[u8] = include_bytes!("embedded/genus_lineage.tsv.zst");
const EMB_CONFORMAL: &[u8] = include_bytes!("embedded/conformal.tsv.zst");

/// Text of a phylogenetic-context table plus a label for its source. Prefers an external file
/// of `filename` in `db_dir` (lets a newer table override without recompiling); otherwise
/// decompresses the copy embedded in the binary at build time. The embedded copy always
/// exists, so these tables are never missing.
fn context_table_text(
    db_dir: Option<&Path>,
    filename: &str,
    embedded_zst: &[u8],
) -> Result<(String, String)> {
    if let Some(dir) = db_dir {
        let p = dir.join(filename);
        if p.exists() {
            let s = std::fs::read_to_string(&p).with_context(|| format!("read {:?}", p))?;
            return Ok((s, format!("{:?}", p)));
        }
    }
    let bytes =
        zstd::decode_all(embedded_zst).with_context(|| format!("decode embedded {}", filename))?;
    let s = String::from_utf8(bytes).with_context(|| format!("embedded {} utf8", filename))?;
    Ok((s, format!("embedded:{}", filename)))
}

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
    /// Redundant PanRes reference IDs tied at this locus (includes `arg_name`). Their
    /// flanking reference sets are unioned for classification. Never empty.
    pub members: Vec<String>,
}

/// Genus classification result for a single ARG.
/// Two genera are "tied" (indistinguishable) when their scores are within this many
/// identity points. Used to decide multi-genus reporting.
pub const GENUS_TIE_PCT: f64 = 1.0;

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
    /// Extracted upstream flanking sequence (retained for export/inspection of
    /// unresolved loci — e.g. flanking present but no flanking-DB match).
    pub upstream_seq: String,
    /// Extracted downstream flanking sequence (retained for export/inspection).
    pub downstream_seq: String,
    /// Replicon context of the matched reference flanking: "plasmid" (mobile, genus
    /// unreliable), "chromosome", "ambiguous", or "NA" (no plasmid list loaded).
    pub context: String,
    /// Number of genera within GENUS_TIE_PCT of the top score (from the full score
    /// list, not just top-5). ≥2 means the call cannot be pinned to a single genus.
    pub n_genera_tied: usize,
    /// Best species (binomial) when a species map is loaded and matches clear the
    /// (higher) species identity threshold; None if unavailable/undeterminable.
    pub species: Option<String>,
    /// Top species matches with scores, for multi-species reporting.
    pub species_top_matches: Vec<(String, f64)>,
    /// Number of species within GENUS_TIE_PCT of the top species score.
    pub n_species_tied: usize,
    /// Credible set: genera accumulating posterior mass to the conformal threshold, each
    /// with its posterior — the structured form of the answer (a set of taxa, not a single
    /// call). The TSV emits its grouped display (`credible_set_grouped`); this Vec is kept
    /// for programmatic consumers.
    #[allow(dead_code)]
    pub credible_set: Vec<(String, f64)>,
    /// Posterior mass covered by `credible_set` (≈0.9+). UNCALIBRATED until conformal
    /// calibration is fitted on a ground-truth set; treat as a raw confidence for now.
    pub support: f64,
    /// Mash radius of the credible set: max distance from its medoid to any member. 0 for
    /// a single-genus call; larger = the shared context spans a broader clade.
    pub resolution_distance: f64,
    /// Why resolution stopped: "query" (flank truncated by the contig edge), "biology"
    /// (full flank but context genuinely shared across genera), or "none" (single genus).
    pub limited_by: String,
    /// Ragged rank of the answer: the LCA rank of the credible set (species/genus/family/
    /// …/root). This is the taxonomic level at which the ARG's context is actually shared.
    pub resolution_rank: String,
    /// LCA taxon name at `resolution_rank` (e.g. "Enterobacteriaceae").
    pub resolution_taxon: String,
    /// The credible set rendered with its close-genera sub-clusters grouped (see
    /// [`group_credible_set`]): `g1(p1),g2(p2) | g3(p3)`. This is the display form of
    /// `credible_set`; empty when there is none.
    pub credible_set_grouped: String,
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

    /// Retrieves all flanking records for a specific gene.
    ///
    /// Decompresses the gene block on demand.
    /// Supports both direct key lookup and gene name mapping (e.g., "mexQ" -> "mexQ|DRUG|CLASS|CODE").
    pub fn get_gene_records(&self, gene: &str) -> Result<Vec<FlankingRecord>> {
        // Try direct lookup, then gene name mapping
        let lookup_key = if self.index.contains_key(gene) {
            gene.to_string()
        } else if let Some(full_key) = self.gene_name_to_key.get(gene) {
            full_key.clone()
        } else {
            anyhow::bail!("Gene not found: {}", gene);
        };

        let entry = self.index.get(&lookup_key)
            .ok_or_else(|| anyhow::anyhow!("Gene not found in index: {}", lookup_key))?;

        // Positioned read at the block offset — does not move a shared file cursor, so
        // this is &self and safe to call concurrently from many threads.
        let mut compressed = vec![0u8; entry.compressed_len as usize];
        self.file.read_exact_at(&mut compressed, entry.offset)?;

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

}

// ============================================================================
// Genus Classifier
// ============================================================================

/// Mash distance between two genus representatives from a neighbor map; 0 if identical,
/// 1.0 (≈ w→0) if the pair is absent (pruned as too distant, or a novel label).
fn ou_lookup(neighbors: &FxHashMap<String, FxHashMap<String, f64>>, a: &str, b: &str) -> f64 {
    if a == b {
        return 0.0;
    }
    if let Some(m) = neighbors.get(a) {
        if let Some(&d) = m.get(b) {
            return d;
        }
    }
    if let Some(m) = neighbors.get(b) {
        if let Some(&d) = m.get(a) {
            return d;
        }
    }
    ABSENT_PATRISTIC
}

/// Distance assigned when a genus pair is absent from the GTDB patristic table. Set beyond the
/// inter-domain median (~2.76 subs/site) so an absent relative contributes ≈0 kernel weight
/// (exp(-3/λ) with λ=0.3 ≈ 5e-5): "unknown neighbor" is treated as "maximally far", never as a
/// close borrow. (Was 1.0 on the old mash scale where distances saturated near 0.34.)
const ABSENT_PATRISTIC: f64 = 3.0;

/// Phylogenetic OU-kernel posterior over genera from per-genus likelihoods.
///
/// post[T] = Σ_{T'} w(T,T')·L[T'] / Σ_{T'} w(T,T'),  w(d) = exp(-d/λ), then normalized to
/// sum 1. Sparse genera borrow evidence from close relatives (Escherichia↔Shigella
/// d≈0.03), so a novel query's mass climbs to the correct clade instead of collapsing
/// onto one reference — this is what kills the argmax/DB-depth bias. Returns (genus,
/// posterior) sorted descending. With an empty neighbor map every off-diagonal weight is
/// exp(-ABSENT_PATRISTIC/λ)≈0, so the result degenerates to the normalized likelihoods (no
/// smoothing).
fn ou_posterior(
    likelihoods: &FxHashMap<String, f64>,
    neighbors: &FxHashMap<String, FxHashMap<String, f64>>,
    lambda: f64,
) -> Vec<(String, f64)> {
    let genera: Vec<&String> = likelihoods.keys().collect();
    let mut post: Vec<(String, f64)> = Vec::with_capacity(genera.len());
    for &ta in &genera {
        let (mut num, mut den) = (0.0f64, 0.0f64);
        for &tb in &genera {
            let w = (-ou_lookup(neighbors, ta, tb) / lambda).exp();
            num += w * likelihoods[tb];
            den += w;
        }
        post.push((ta.clone(), if den > 0.0 { num / den } else { 0.0 }));
    }
    let z: f64 = post.iter().map(|(_, p)| *p).sum();
    if z > 0.0 {
        for p in post.iter_mut() {
            p.1 /= z;
        }
    }
    post.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    post
}

/// A credible set whose patristic radius is within this is "phylogenetically coherent" — a
/// tight cluster of close genera — and is reported as the genus list rather than rolled up.
/// The criterion is phylogenetic COHERENCE (the radius), NOT the number of genera: two far
/// genera and ten close ones are different answers even at the same family. Scaled to the GTDB
/// patristic table (genus-pair median ≈0.19, family-pair median ≈0.60), so 0.5 sits just below
/// the family diameter: within-family sisters list individually; a set spilling past one family
/// rolls up to its LCA. (Was 0.12 on the old mash scale, which saturated near 0.30 at family.)
const GENUS_COHERENCE_RADIUS: f64 = 0.5;

/// Renders the credible set's resolution notation. The credible set itself is the answer
/// (in `Credible_Set`); this chooses the human label for `Resolution_Rank`/`_Taxon` from the
/// set's phylogenetic COHERENCE (its mash `radius`), not its size:
///   - one genus                    → ("genus", that genus)
///   - several genera, radius tight  → ("genus", "Escherichia|Shigella") — a coherent
///     cluster of close genera, named individually (strictly more specific than the family)
///   - several genera, radius spread → (LCA rank, LCA taxon) — the genera fan out across a
///     family/order/…, so the clade name is the honest summary
/// {Escherichia, Shigella} (radius ≈0.03) lists both genera; a set fanning across
/// Enterobacteriaceae (radius ≈0.17) reports "Enterobacteriaceae" even if it's only 2 genera.
fn render_resolution(members: &[&str], lca_rank: &str, lca_taxon: &str, radius: f64)
    -> (String, String) {
    if members.len() == 1 {
        ("genus".to_string(), members[0].to_string())
    } else if radius <= GENUS_COHERENCE_RADIUS {
        ("genus".to_string(), members.join("|"))
    } else {
        (lca_rank.to_string(), lca_taxon.to_string())
    }
}

/// Renders the credible set with its internal phylogenetic structure exposed: genera are
/// single-linkage clustered by mash distance (two genera join if within `threshold`), so a
/// tight sub-cluster (Escherichia+Pseudescherichia) is shown as one group and the genera it
/// fans out to are shown separately. Clusters are sorted by total posterior mass; within a
/// cluster, members by posterior. Format: `g1(p1),g2(p2) | g3(p3) | g4(p4)` — commas inside
/// a tight cluster, ` | ` between clusters. A flat list hides that the mass is really on one
/// close group plus a few outliers; this shows it.
fn group_credible_set(
    members: &[(String, f64)],
    neighbors: &FxHashMap<String, FxHashMap<String, f64>>,
    threshold: f64,
) -> String {
    let n = members.len();
    if n == 0 {
        return "NA".to_string();
    }
    if n == 1 {
        return format!("{}({:.2})", members[0].0, members[0].1);
    }
    // Single-linkage union-find over pairs within `threshold`.
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if ou_lookup(neighbors, &members[i].0, &members[j].0) <= threshold {
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }
    let mut clusters: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for i in 0..n {
        let r = find(&mut parent, i);
        clusters.entry(r).or_default().push(i);
    }
    let mut rendered: Vec<(f64, String)> = clusters
        .values()
        .map(|idxs| {
            let mut mem = idxs.clone();
            mem.sort_by(|&a, &b| {
                members[b].1.partial_cmp(&members[a].1).unwrap_or(std::cmp::Ordering::Equal)
            });
            let mass: f64 = mem.iter().map(|&k| members[k].1).sum();
            let s = mem.iter()
                .map(|&k| format!("{}({:.2})", members[k].0, members[k].1))
                .collect::<Vec<_>>()
                .join(",");
            (mass, s)
        })
        .collect();
    rendered.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    rendered.into_iter().map(|(_, s)| s).collect::<Vec<_>>().join(" | ")
}

/// Lowest common ancestor of a set of genera over their lineages: the finest standard
/// rank at which every member shares one non-empty taxon. Returns (rank, taxon). A single
/// genus resolves to ("genus", that genus); genera that agree only at the root give
/// ("root", "unclassified"). This renders the credible set as one ragged-rank label —
/// multi-genus {Escherichia, Salmonella} → ("family", "Enterobacteriaceae").
fn lca_of(genera: &[&str], lineage: &FxHashMap<String, [String; 7]>) -> (String, String) {
    let lins: Vec<&[String; 7]> = genera.iter().filter_map(|g| lineage.get(*g)).collect();
    if lins.is_empty() {
        // No lineage info: fall back to the genus label itself if singleton.
        return if genera.len() == 1 {
            ("genus".to_string(), genera[0].to_string())
        } else {
            ("root".to_string(), "unclassified".to_string())
        };
    }
    // Finest → coarsest, capped at genus (idx 5): the credible set is a set of GENERA, so
    // the species column (a representative genome's species) is not a valid resolution —
    // species-level calls come from the separate species tally, not this LCA.
    for r in (0..6).rev() {
        let first = &lins[0][r];
        if !first.is_empty() && lins.iter().all(|l| &l[r] == first) {
            return (LINEAGE_RANKS[r].to_string(), first.clone());
        }
    }
    ("root".to_string(), "unclassified".to_string())
}

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
    /// Source contig accessions known to be plasmids (from PLSDB-derived flanking).
    /// Empty if no plasmid list was provided → Context reported as "NA".
    plasmid_contigs: rustc_hash::FxHashSet<String>,
    /// Higher identity threshold (0-1) required to call species. 0 disables species.
    species_identity: f64,
    /// Source contig accession → species (binomial). Empty disables species calls.
    species_map: FxHashMap<String, String>,
    /// Plasmid-fraction thresholds for the Context call: frac >= `plasmid_hi`
    /// → "plasmid", frac <= `plasmid_lo` → "chromosome", else "ambiguous".
    plasmid_hi: f64,
    plasmid_lo: f64,
    /// Optional cap on reference flanks aligned per locus (0 = unlimited). A few genes
    /// carry tens of thousands of flanks; capping trades some accuracy for speed.
    max_ref_flanks: usize,
    /// Phylogenetic OU kernel: genus -> [(neighbor_genus, mash_distance)]. Loaded from a
    /// sibling `genus_dist.tsv` next to the FDB (genus-representative mash distances,
    /// k=21 s=100000). Empty => no smoothing (posterior = normalized per-genus likelihood).
    genus_neighbors: FxHashMap<String, FxHashMap<String, f64>>,
    /// OU relaxation length (= 1/alpha). ln2*lambda = distance at which borrowed
    /// evidence halves. NOT Pagel's lambda.
    kernel_lambda: f64,
    /// Likelihood sharpness: L = exp(-(1 - ani)/tau) per alignment, ani = identity/100.
    kernel_tau: f64,
    /// genus -> its standard 7-rank lineage [superkingdom, phylum, class, order, family,
    /// genus, species]. Loaded from sibling `genus_lineage.tsv`. Enables reporting the
    /// credible set's LCA as a ragged rank (e.g. multi-genus Enterobacteriaceae → family).
    genus_lineage: FxHashMap<String, [String; 7]>,
    /// Conformal credible-set mass threshold θ for the requested target coverage, read
    /// from sibling `conformal.tsv`. The set grows until its posterior mass reaches θ,
    /// which guarantees the true host's clade is covered at ≥ the target rate. Falls back
    /// to `DEFAULT_CREDIBLE_MASS` (uncalibrated) when the table is absent.
    conformal_theta: f64,
}

/// Uncalibrated fallback credible-set mass when no `conformal.tsv` is available.
const DEFAULT_CREDIBLE_MASS: f64 = 0.9;

/// Standard ranks, coarsest → finest, matching the columns of `genus_lineage.tsv`.
const LINEAGE_RANKS: [&str; 7] =
    ["superkingdom", "phylum", "class", "order", "family", "genus", "species"];

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
        plasmid_contigs_path: Option<&Path>,
        species_identity: f64,
        species_map_path: Option<&Path>,
        plasmid_hi: f64,
        plasmid_lo: f64,
        max_ref_flanks: usize,
        // Requested coverage for the conformal credible set (e.g. 0.9 = the true host's
        // clade is inside the reported set >=90% of the time). Selects θ from conformal.tsv.
        target_coverage: f64,
    ) -> Result<Self> {
        let db_dir = db_path.as_ref().parent().map(|p| p.to_path_buf());
        let db = FlankingDatabase::open(db_path)?;
        let mut plasmid_contigs = rustc_hash::FxHashSet::default();
        if let Some(p) = plasmid_contigs_path {
            let f = File::open(p).with_context(|| format!("open plasmid list {:?}", p))?;
            for line in BufReader::new(f).lines() {
                let acc = line?.trim().to_string();
                if !acc.is_empty() { plasmid_contigs.insert(acc); }
            }
        }
        let mut species_map = FxHashMap::default();
        if let Some(p) = species_map_path {
            let f = File::open(p).with_context(|| format!("open species map {:?}", p))?;
            for line in BufReader::new(f).lines() {
                let line = line?;
                if let Some((c, sp)) = line.split_once('\t') {
                    let (c, sp) = (c.trim(), sp.trim());
                    if !c.is_empty() && !sp.is_empty() { species_map.insert(c.to_string(), sp.to_string()); }
                }
            }
        }
        // Phylogenetic OU kernel distances: sibling `genus_dist.tsv` next to the FDB.
        // Format: genus_a \t genus_b \t mash_distance (both directions present). Absent
        // => empty map => posterior degenerates to normalized per-genus likelihood.
        let mut genus_neighbors: FxHashMap<String, FxHashMap<String, f64>> = FxHashMap::default();
        {
            let (gd_text, gd_src) =
                context_table_text(db_dir.as_deref(), "genus_dist.tsv", EMB_GENUS_DIST)?;
            let mut npair = 0usize;
            for line in gd_text.lines() {
                let mut it = line.split('\t');
                if let (Some(a), Some(b), Some(d)) = (it.next(), it.next(), it.next()) {
                    if let Ok(d) = d.trim().parse::<f64>() {
                        genus_neighbors.entry(a.to_string()).or_default()
                            .insert(b.to_string(), d);
                        npair += 1;
                    }
                }
            }
            eprintln!("[kernel] loaded {} genus-distance pairs from {}", npair, gd_src);
        }
        // Ragged-rank lineage: sibling `genus_lineage.tsv`, header
        // `genus<TAB>superkingdom..species`. Absent => LCA reporting disabled (genus only).
        let mut genus_lineage: FxHashMap<String, [String; 7]> = FxHashMap::default();
        {
            let (gl_text, gl_src) =
                context_table_text(db_dir.as_deref(), "genus_lineage.tsv", EMB_GENUS_LINEAGE)?;
            for (i, line) in gl_text.lines().enumerate() {
                if i == 0 {
                    continue; // header
                }
                let cols: Vec<&str> = line.split('\t').collect();
                if cols.len() >= 8 {
                    let mut lin: [String; 7] = Default::default();
                    for r in 0..7 {
                        lin[r] = cols[r + 1].to_string();
                    }
                    genus_lineage.insert(cols[0].to_string(), lin);
                }
            }
            eprintln!("[kernel] loaded {} genus lineages from {}", genus_lineage.len(), gl_src);
        }
        // Conformal θ: sibling `conformal.tsv`, rows `target_coverage<TAB>theta<TAB>...`.
        // Pick the row whose target is closest to the requested coverage.
        let mut conformal_theta = DEFAULT_CREDIBLE_MASS;
        let mut conformal_target = 0.0;
        {
            let (cf_text, cf_src) =
                context_table_text(db_dir.as_deref(), "conformal.tsv", EMB_CONFORMAL)?;
            let mut best = f64::MAX;
            for line in cf_text.lines() {
                if line.starts_with('#') || line.trim().is_empty() {
                    continue;
                }
                let c: Vec<&str> = line.split('\t').collect();
                if c.len() >= 2 {
                    if let (Ok(t), Ok(th)) = (c[0].parse::<f64>(), c[1].parse::<f64>()) {
                        let d = (t - target_coverage).abs();
                        if d < best {
                            best = d;
                            conformal_theta = th;
                            conformal_target = t;
                        }
                    }
                }
            }
            if conformal_target > 0.0 {
                eprintln!("[kernel] conformal θ={:.3} for target coverage {:.2} (from {})",
                          conformal_theta, conformal_target, cf_src);
            }
        }
        Ok(Self {
            db,
            minimap2_path: minimap2_path.to_string(),
            min_identity,
            min_align_len,
            max_flanking,
            plasmid_contigs,
            species_identity,
            species_map,
            plasmid_hi,
            plasmid_lo,
            max_ref_flanks,
            genus_neighbors,
            kernel_lambda: 0.3,
            kernel_tau: 0.01,
            genus_lineage,
            conformal_theta,
        })
    }

    /// Phylogenetic OU-kernel posterior over genera from per-genus likelihoods, using
    /// this classifier's loaded genus distances and lambda. Thin wrapper over the free
    /// [`ou_posterior`] (extracted so the kernel math is unit-testable without an FDB).
    fn kernel_posterior(&self, likelihoods: &FxHashMap<String, f64>) -> Vec<(String, f64)> {
        ou_posterior(likelihoods, &self.genus_neighbors, self.kernel_lambda)
    }

    /// Classifies genus for multiple ARG positions.
    ///
    /// Two phases: (1) a serial pass reads every gene's flanking records and genus
    /// distribution from the .fdb — the only work needing `&mut self` (the reader
    /// seeks the file) — deduplicated by gene; (2) the flanking alignment + scoring
    /// per locus, which is independent and CPU/subprocess-bound, runs in parallel.
    /// Each locus writes its own indexed temp files, so there are no conflicts.
    pub fn classify_batch(&mut self, positions: &[ArgPosition], threads: usize) -> Result<Vec<GenusResult>> {
        use rayon::prelude::*;
        use std::collections::HashSet;

        let this: &GenusClassifier = self;
        let pool = rayon::ThreadPoolBuilder::new().num_threads(threads.max(1)).build()?;

        // Unique genes present in the DB — dedup so a gene shared by many loci is read
        // and decompressed only once.
        let mut seen: HashSet<&str> = HashSet::new();
        let unique_genes: Vec<&str> = positions.iter()
            .flat_map(|p| p.members.iter().map(|s| s.as_str()))
            .filter(|&g| this.db.has_gene(g) && seen.insert(g))
            .collect();

        pool.install(|| -> Result<Vec<GenusResult>> {
            // Phase 1 (parallel): positioned reads (read_at) decompress each gene block
            // concurrently; the genus distribution is derived from the same records so
            // the block is decompressed only once.
            let loaded: Vec<(String, Vec<FlankingRecord>, FxHashMap<String, usize>)> = unique_genes
                .par_iter()
                .map(|&g| {
                    let recs = this.db.get_gene_records(g)?;
                    let mut dist: FxHashMap<String, usize> = FxHashMap::default();
                    for rec in &recs { *dist.entry(rec.genus.clone()).or_default() += 1; }
                    Ok((g.to_string(), recs, dist))
                })
                .collect::<Result<Vec<_>>>()?;

            let mut recs_by_gene: FxHashMap<String, Vec<FlankingRecord>> = FxHashMap::default();
            let mut dist_by_gene: FxHashMap<String, FxHashMap<String, usize>> = FxHashMap::default();
            for (g, recs, dist) in loaded {
                recs_by_gene.insert(g.clone(), recs);
                dist_by_gene.insert(g, dist);
            }

            // Phase 2 (parallel): flanking alignment + scoring, one locus per task.
            positions
                .par_iter()
                .enumerate()
                .map(|(idx, pos)| this.classify_prepared(pos, idx, &recs_by_gene, &dist_by_gene))
                .collect()
        })
    }

    /// Classifies genus for a single ARG position using preloaded per-gene data
    /// (so it needs only `&self` and is safe to run in parallel across loci).
    ///
    /// # Algorithm
    /// 1. Extract flanking sequences from contig
    /// 2. Write query and reference FASTA files (indexed by `idx` to avoid conflicts)
    /// 3. Run minimap2 alignment (single-threaded; parallelism is across loci)
    /// 4. Parse PAF and score genus candidates
    /// 5. Return top genus with confidence metrics
    fn classify_prepared(
        &self,
        pos: &ArgPosition,
        idx: usize,
        recs_by_gene: &FxHashMap<String, Vec<FlankingRecord>>,
        dist_by_gene: &FxHashMap<String, FxHashMap<String, usize>>,
    ) -> Result<GenusResult> {
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
                upstream_seq: upstream.clone(),
                downstream_seq: downstream.clone(),
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
            });
        }

        // Check if gene exists in database (any tied member present == has_gene)
        if !pos.members.iter().any(|m| recs_by_gene.contains_key(m)) {
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
                upstream_seq: upstream.clone(),
                downstream_seq: downstream.clone(),
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
            });
        }

        // Union the flanking references of all tied redundant refs at this locus, so the
        // same physical gene's evidence from every source DB is used together.
        let mut ref_records: Vec<&FlankingRecord> = pos.members.iter()
            .filter_map(|m| recs_by_gene.get(m))
            .flatten()
            .collect();
        // Optional speed cap (self.max_ref_flanks, 0 = unlimited/default): some genes
        // carry tens of thousands of flanks (up to ~128k), which dominate wall-clock
        // (writing + indexing + aligning). An evenly-strided sample keeps most of the
        // genus signal but DOES shift the call on those heavy loci, so it is off by
        // default; the specificity denominator always uses the full counts.
        if self.max_ref_flanks > 0 && ref_records.len() > self.max_ref_flanks {
            let step = ref_records.len() / self.max_ref_flanks;
            ref_records = ref_records.iter().step_by(step).copied().take(self.max_ref_flanks).collect();
        }
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
                upstream_seq: upstream.clone(),
                downstream_seq: downstream.clone(),
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
            });
        }

        // Create temporary files for alignment. Prefer tmpfs (/dev/shm) to avoid disk
        // I/O, and key names by pid+idx so parallel loci never collide.
        let temp_dir = if Path::new("/dev/shm").is_dir() {
            std::path::PathBuf::from("/dev/shm")
        } else {
            std::env::temp_dir()
        };
        let pid = std::process::id();
        let query_path = temp_dir.join(format!("argenus_query_{}_{}.fas", pid, idx));
        let ref_path = temp_dir.join(format!("argenus_ref_{}_{}.fas", pid, idx));
        let paf_path = temp_dir.join(format!("argenus_align_{}_{}.paf", pid, idx));

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
            .args(["-x", "sr", "-t", "1", "-c", "--secondary=yes", "-N", "100", "-k", "15", "-w", "5"])
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
                upstream_seq: upstream.clone(),
                downstream_seq: downstream.clone(),
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
            });
        }

        // Parse PAF and calculate genus scores + plasmid provenance + species scores.
        let (genus_likelihood, genus_confidence, plasmid_frac, species_scores) =
            self.calculate_genus_scores(&paf_path)?;

        // Cleanup temporary files
        let _ = std::fs::remove_file(&query_path);
        let _ = std::fs::remove_file(&ref_path);
        let _ = std::fs::remove_file(&paf_path);

        // Context from provenance of the matched reference flanking (mobility proxy).
        // "plasmid" = the flanking mostly matched PLSDB-derived plasmid references
        // (genus is then unreliable — the ARG is on a mobile element). NA when no
        // plasmid list is loaded OR the flanking matched nothing (no basis to judge).
        let context = if self.plasmid_contigs.is_empty() || genus_likelihood.is_empty() {
            "NA".to_string()
        } else if plasmid_frac >= self.plasmid_hi {
            "plasmid".to_string()
        } else if plasmid_frac <= self.plasmid_lo {
            "chromosome".to_string()
        } else {
            "ambiguous".to_string()
        };

        // Calculate genus specificity from the DB, combined over all tied members.
        let mut genus_dist: FxHashMap<String, usize> = FxHashMap::default();
        for m in &pos.members {
            if let Some(d) = dist_by_gene.get(m) {
                for (k, v) in d { *genus_dist.entry(k.clone()).or_default() += *v; }
            }
        }
        let total_in_db: usize = genus_dist.values().sum();

        // Determine best genus via the phylogenetic OU-kernel posterior (replaces the
        // old count-weighted identity score). `sorted_scores` now holds (genus, posterior)
        // in descending posterior order; DB depth no longer biases the ranking.
        let sorted_scores: Vec<(String, f64)> = self.kernel_posterior(&genus_likelihood);

        let (genus, confidence, specificity) = if let Some((best_genus, _post)) = sorted_scores.first() {
            let genus_count = genus_dist.get(best_genus).copied().unwrap_or(0);
            let specificity = if total_in_db > 0 {
                (genus_count as f64 / total_in_db as f64) * 100.0
            } else {
                0.0
            };
            // Confidence stays on the 0-100 identity scale (mean alignment identity of the
            // called genus), independent of the posterior used for ranking.
            let conf = genus_confidence.get(best_genus).copied().unwrap_or(0.0);
            (Some(best_genus.clone()), conf, specificity)
        } else {
            (None, 0.0, 0.0)
        };

        // Credible set: the smallest prefix of the posterior-sorted genera whose mass
        // reaches the conformal threshold θ. θ is calibrated so that the true host's clade
        // falls inside the set's LCA at ≥ the target coverage — this is what makes Support
        // mean something. Uncalibrated (θ = DEFAULT_CREDIBLE_MASS) when conformal.tsv is
        // absent. Single genus = resolved; several = shared/mobile context.
        let mut credible_set: Vec<(String, f64)> = Vec::new();
        let mut support = 0.0f64;
        for (g, p) in &sorted_scores {
            credible_set.push((g.clone(), *p));
            support += *p;
            if support >= self.conformal_theta {
                break;
            }
        }
        let n_genera_tied = credible_set.len();

        // Resolution distance = mash radius of the credible set around its posterior-
        // weighted medoid (the member minimizing Σ p·d to the others). 0 for one genus.
        let resolution_distance = if credible_set.len() < 2 {
            0.0
        } else {
            let members: Vec<&str> = credible_set.iter().map(|(g, _)| g.as_str()).collect();
            let medoid = members.iter().min_by(|&&a, &&b| {
                let sa: f64 = credible_set.iter()
                    .map(|(g, p)| p * ou_lookup(&self.genus_neighbors, a, g)).sum();
                let sb: f64 = credible_set.iter()
                    .map(|(g, p)| p * ou_lookup(&self.genus_neighbors, b, g)).sum();
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            }).copied().unwrap_or(members[0]);
            members.iter()
                .map(|&g| ou_lookup(&self.genus_neighbors, medoid, g))
                .fold(0.0f64, f64::max)
        };

        // Limited-by: was resolution capped by the query (contig edge cut the flank) or by
        // biology (full flank, but the context is genuinely shared across genera)?
        let reach_up = pos.arg_start.min(self.max_flanking);
        let reach_dn = pos.contig_len.saturating_sub(pos.arg_end).min(self.max_flanking);
        let contig_limited = reach_up < self.max_flanking || reach_dn < self.max_flanking;
        let limited_by = if credible_set.len() < 2 {
            "none".to_string()
        } else if contig_limited {
            "query".to_string()
        } else {
            "biology".to_string()
        };

        // Ragged rank: the LCA of the credible set is the taxonomic level at which this
        // ARG's context is actually shared (single genus → genus; several → their family
        // or higher). This is the honest answer, not a forced single genus.
        let (resolution_rank, resolution_taxon) = if credible_set.is_empty() {
            ("NA".to_string(), "NA".to_string())
        } else {
            let members: Vec<&str> = credible_set.iter().map(|(g, _)| g.as_str()).collect();
            let (lca_rank, lca_taxon) = lca_of(&members, &self.genus_lineage);
            render_resolution(&members, &lca_rank, &lca_taxon, resolution_distance)
        };
        let credible_set_grouped = if credible_set.is_empty() {
            String::new()
        } else {
            group_credible_set(&credible_set, &self.genus_neighbors, GENUS_COHERENCE_RADIUS)
        };

        let top_matches: Vec<(String, f64)> = sorted_scores.into_iter().take(5).collect();

        // Species resolution (parallel to genus, from the higher-threshold tally).
        let mut sorted_species: Vec<(String, f64)> = species_scores.into_iter().collect();
        sorted_species.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let species = sorted_species.first().map(|(s, _)| s.clone());
        let n_species_tied = match sorted_species.first() {
            Some((_, best)) => sorted_species.iter()
                .filter(|(s, sc)| !s.is_empty() && *sc >= best - GENUS_TIE_PCT)
                .count(),
            None => 0,
        };
        let species_top_matches: Vec<(String, f64)> = sorted_species.into_iter().take(5).collect();

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
            upstream_seq: upstream.clone(),
            downstream_seq: downstream.clone(),
            context,
            n_genera_tied,
            species,
            species_top_matches,
            n_species_tied,
            credible_set,
            support,
            resolution_distance,
            limited_by,
            resolution_rank,
            resolution_taxon,
            credible_set_grouped,
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
    /// Parses the PAF and returns per-genus (mean likelihood, mean identity), the plasmid
    /// fraction, and per-species mean identity. The kernel posterior consumes the
    /// likelihoods; the identities feed confidence/context (kept on the 0-100 scale).
    fn calculate_genus_scores(&self, paf_path: &Path)
        -> Result<(FxHashMap<String, f64>, FxHashMap<String, f64>, f64, FxHashMap<String, f64>)> {
        let file = File::open(paf_path)?;
        let reader = BufReader::new(file);

        let mut genus_matches: FxHashMap<String, Vec<f64>> = FxHashMap::default();
        let min_identity_pct = self.min_identity * 100.0;
        // Track plasmid provenance of the matched reference flanking records.
        let (mut plasmid_hits, mut total_hits) = (0usize, 0usize);
        // Species tally (only alignments clearing the higher species threshold whose
        // source contig has a species label).
        let mut species_matches: FxHashMap<String, Vec<f64>> = FxHashMap::default();
        let species_pct = self.species_identity * 100.0;
        let do_species = self.species_identity > 0.0 && !self.species_map.is_empty();

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

            // Target name is "genus|contig|direction_idx".
            let target_name = fields[5];
            let mut toks = target_name.split('|');
            let genus = toks.next().unwrap_or("");
            let contig = toks.next().unwrap_or("");
            // Skip records with an empty genus label (DB label gaps) — they must not
            // win the classification (fix for the empty-"" winner bug).
            if genus.is_empty() {
                continue;
            }
            total_hits += 1;
            if !contig.is_empty() && self.plasmid_contigs.contains(contig) {
                plasmid_hits += 1;
            }
            genus_matches.entry(genus.to_string()).or_default().push(identity);

            // Species tally: stricter identity + a species label for this contig.
            if do_species && identity >= species_pct {
                if let Some(sp) = self.species_map.get(contig) {
                    species_matches.entry(sp.clone()).or_default().push(identity);
                }
            }
        }

        // Per-genus mean likelihood (kernel input) and mean identity (confidence). The
        // likelihood is averaged WITHIN genus so DB depth (many flanks of one genus) can
        // no longer inflate the score — the old count bonus did exactly that.
        let tau = self.kernel_tau;
        let mut genus_likelihood: FxHashMap<String, f64> = FxHashMap::default();
        let mut genus_confidence: FxHashMap<String, f64> = FxHashMap::default();
        for (genus, scores) in genus_matches {
            if scores.is_empty() {
                continue;
            }
            let n = scores.len() as f64;
            let mean_id = scores.iter().sum::<f64>() / n;
            let mean_lik = scores.iter()
                .map(|id| (-(1.0 - id / 100.0) / tau).exp())
                .sum::<f64>() / n;
            genus_likelihood.insert(genus.clone(), mean_lik);
            genus_confidence.insert(genus, mean_id);
        }

        let mut species_scores: FxHashMap<String, f64> = FxHashMap::default();
        for (sp, scores) in species_matches {
            if scores.is_empty() { continue; }
            let avg = scores.iter().sum::<f64>() / scores.len() as f64;
            species_scores.insert(sp, avg);
        }

        let plasmid_frac = if total_hits > 0 { plasmid_hits as f64 / total_hits as f64 } else { 0.0 };
        Ok((genus_likelihood, genus_confidence, plasmid_frac, species_scores))
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

    fn lik(pairs: &[(&str, f64)]) -> FxHashMap<String, f64> {
        pairs.iter().map(|(g, l)| (g.to_string(), *l)).collect()
    }

    #[test]
    fn test_ou_posterior_no_neighbors_is_normalized_likelihood() {
        // Empty distance map => every off-diagonal weight ≈ 0 => posterior is just the
        // likelihoods renormalized (no phylogenetic smoothing).
        let neighbors = FxHashMap::default();
        let l = lik(&[("Escherichia", 3.0), ("Salmonella", 1.0)]);
        let post = ou_posterior(&l, &neighbors, 0.1);
        assert_eq!(post[0].0, "Escherichia");
        // Not exactly 0.75/0.25: the absent pair still carries w=exp(-ABSENT_PATRISTIC/λ)=
        // exp(-30)≈9e-14 of residual smoothing, which is the intended "≈0" degenerate limit.
        assert!((post[0].1 - 0.75).abs() < 1e-3, "got {}", post[0].1);
        assert!((post[1].1 - 0.25).abs() < 1e-3, "got {}", post[1].1);
    }

    #[test]
    fn test_ou_posterior_borrows_from_close_relative() {
        // Escherichia has all the likelihood; Salmonella has none but sits d=0.0406 away
        // (same family, GTDB patristic), while Bacillus is far (absent => d=ABSENT_PATRISTIC).
        // The kernel must lift Salmonella's posterior well above Bacillus's — borrowed by
        // proximity.
        let mut neighbors: FxHashMap<String, FxHashMap<String, f64>> = FxHashMap::default();
        neighbors.entry("Escherichia".into()).or_default().insert("Salmonella".into(), 0.0406);
        neighbors.entry("Salmonella".into()).or_default().insert("Escherichia".into(), 0.0406);
        let l = lik(&[("Escherichia", 1.0), ("Salmonella", 0.0), ("Bacillus", 0.0)]);
        let post = ou_posterior(&l, &neighbors, 0.3);
        let p: FxHashMap<_, _> = post.into_iter().collect();
        assert!(p["Salmonella"] > p["Bacillus"], "close relative must outrank far one");
        assert!(p["Salmonella"] > 0.05, "Salmonella should borrow real mass, got {}", p["Salmonella"]);
        assert!(p["Escherichia"] > p["Salmonella"], "the observed genus still leads");
    }

    fn ent(g: &str, fam: &str, gen: &str) -> (String, [String; 7]) {
        // [superkingdom, phylum, class, order, family, genus, species]
        (g.to_string(), [
            "Bacteria".into(), "Pseudomonadota".into(), "Gammaproteobacteria".into(),
            "Enterobacterales".into(), fam.into(), gen.into(), format!("{} sp.", gen),
        ])
    }

    #[test]
    fn test_lca_ragged_rank() {
        let mut lin: FxHashMap<String, [String; 7]> = FxHashMap::default();
        for (k, v) in [ent("Escherichia", "Enterobacteriaceae", "Escherichia"),
                       ent("Salmonella", "Enterobacteriaceae", "Salmonella")] {
            lin.insert(k, v);
        }
        lin.insert("Bacillus".into(), [
            "Bacteria".into(), "Bacillota".into(), "Bacilli".into(), "Bacillales".into(),
            "Bacillaceae".into(), "Bacillus".into(), "Bacillus subtilis".into(),
        ]);
        // Single genus → resolves to genus.
        assert_eq!(lca_of(&["Escherichia"], &lin), ("genus".into(), "Escherichia".into()));
        // Two Enterobacteriaceae genera → family LCA (the ragged-rank answer).
        assert_eq!(lca_of(&["Escherichia", "Salmonella"], &lin),
                   ("family".into(), "Enterobacteriaceae".into()));
        // Across phyla → superkingdom.
        assert_eq!(lca_of(&["Escherichia", "Bacillus"], &lin),
                   ("superkingdom".into(), "Bacteria".into()));
    }

    #[test]
    fn test_render_resolution_notation() {
        // Single genus → that genus (radius irrelevant).
        assert_eq!(render_resolution(&["Escherichia"], "genus", "Escherichia", 0.0),
                   ("genus".into(), "Escherichia".into()));
        // Tight cluster (small radius) → list the genera, NOT the family — coherence, not count.
        assert_eq!(render_resolution(&["Escherichia", "Shigella"], "family", "Enterobacteriaceae", 0.03),
                   ("genus".into(), "Escherichia|Shigella".into()));
        // Many genera but still tight → still listed (count does NOT trigger rollup).
        let tight_many = ["A", "B", "C", "D", "E", "F"];
        assert_eq!(render_resolution(&tight_many, "family", "Enterobacteriaceae", 0.10),
                   ("genus".into(), "A|B|C|D|E|F".into()));
        // Two genera spread past the family diameter (radius > 0.5) → the family name.
        assert_eq!(render_resolution(&["Escherichia", "Klebsiella"], "family", "Enterobacteriaceae", 0.6),
                   ("family".into(), "Enterobacteriaceae".into()));
        // Genera spanning families (large patristic radius, coarse LCA) → the LCA clade.
        assert_eq!(render_resolution(&["Escherichia", "Bacillus"], "superkingdom", "Bacteria", 1.5),
                   ("superkingdom".into(), "Bacteria".into()));
    }

    #[test]
    fn test_group_credible_set_clusters_by_distance() {
        // Escherichia+Pseudescherichia are tight (0.08 patristic); Cronobacter and Klebsiella
        // sit >0.5 from them and from each other → three clusters. The tight pair (mass 0.55)
        // leads; within it the higher-posterior genus is first.
        let mut nb: FxHashMap<String, FxHashMap<String, f64>> = FxHashMap::default();
        let mut set = |a: &str, b: &str, d: f64| {
            nb.entry(a.into()).or_default().insert(b.into(), d);
            nb.entry(b.into()).or_default().insert(a.into(), d);
        };
        set("Escherichia", "Pseudescherichia", 0.08);
        set("Escherichia", "Cronobacter", 0.90);
        set("Escherichia", "Klebsiella", 0.70);
        set("Pseudescherichia", "Cronobacter", 0.90);
        set("Pseudescherichia", "Klebsiella", 0.70);
        set("Cronobacter", "Klebsiella", 0.80);
        let members = vec![
            ("Escherichia".to_string(), 0.30),
            ("Cronobacter".to_string(), 0.25),
            ("Pseudescherichia".to_string(), 0.25),
            ("Klebsiella".to_string(), 0.20),
        ];
        let out = group_credible_set(&members, &nb, GENUS_COHERENCE_RADIUS);
        assert_eq!(out, "Escherichia(0.30),Pseudescherichia(0.25) | Cronobacter(0.25) | Klebsiella(0.20)");
    }
}
