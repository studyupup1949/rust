//! Per-locus reassembly (v3 core/flank read split) — opt-in via `--reassemble`.
//!
//! Rationale (validated in the low-depth ablation, see project memory
//! `perlocus-reassembly-and-emit`): reads that align >=70% of their length to the
//! ARG are the shared, conserved *core* that BRIDGES different genomes' flanking
//! during assembly and collapses them into chimeras. If we split reads per-mate,
//! drop pairs where both mates are core, keep the flanking mate of a half-core pair
//! as a singleton, and assemble the remaining *flanking* reads alone, the flanking
//! separates by genome. We then stitch the ARG gene back onto each flanking contig
//! so ARGenus can re-classify it with the full 4-axis (genus/species/context)
//! machinery — which honestly labels promiscuous plasmid genes rather than forcing
//! a single (usually wrong) genus.
//!
//! This targets STALLED loci only (short flanking on the strict contig): they are
//! `NA` on every axis today, and reassembly is what converts them into something
//! classifiable. Plasmid-context loci are skipped by the caller (reassembly cannot
//! fix a mobile-element genus). All artifacts are written under a per-sample
//! `reassemble/` dir and the step is resumable (existing `spades/contigs.fasta`
//! is reused).

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use rustc_hash::FxHashMap;

use crate::seqio::{FastaReader, FastqFile};

/// Tool paths + tuning for the reassembly step.
pub struct ReasmConfig {
    pub minimap2: String,
    /// Path to `spades.py`.
    pub spades_py: PathBuf,
    /// Python interpreter that can run SPAdes (its pixi env python; the system
    /// python is often too old). Empty => call `spades_py` directly on PATH.
    pub spades_python: PathBuf,
    pub threads: usize,
    /// Fraction of a read covered by the ARG alignment to call it "core" (0.70).
    pub core_cov: f64,
    /// Concurrent SPAdes jobs.
    pub jobs: usize,
}

/// A stalled locus to reassemble: the ARG gene region plus the whole strict contig
/// (used as the orientation seed when stitching).
pub struct StalledLocus {
    /// Unique key, `arg_name:contig` — also the output name prefix.
    pub key: String,
    pub arg_name: String,
    pub arg_class: String,
    pub identity: f64,
    pub coverage: f64,
    /// ARG gene sequence (substring of the contig).
    pub gene_seq: String,
    /// Whole strict contig sequence (orientation seed).
    pub seed_seq: String,
}

/// One stitched flanking contig for a locus, with the ARG gene glued back on and
/// the gene's coordinates on the stitched sequence (for re-classification).
pub struct StitchedContig {
    pub name: String,
    pub seq: String,
    pub arg_start: usize,
    pub arg_end: usize,
}

/// Reassembly output for one original locus (may yield several flanking contigs =
/// genuine multi-genome presence).
pub struct StitchedLocus {
    pub key: String,
    pub arg_name: String,
    pub arg_class: String,
    pub identity: f64,
    pub coverage: f64,
    pub contigs: Vec<StitchedContig>,
}

fn norm_id(h: &str) -> String {
    let mut t = h.split_whitespace().next().unwrap_or("").to_string();
    if let Some(first) = t.chars().next() {
        if first == '@' || first == '>' {
            t = t[1..].to_string();
        }
    }
    // strip trailing /1 or /2 (possibly repeated: minimap2 can append its own)
    loop {
        let b = t.as_bytes();
        if b.len() > 2 && b[b.len() - 2] == b'/' && (b[b.len() - 1] == b'1' || b[b.len() - 1] == b'2') {
            t.truncate(b.len() - 2);
        } else {
            break;
        }
    }
    t
}

fn revcomp(s: &str) -> String {
    s.bytes()
        .rev()
        .map(|b| match b {
            b'A' => 'T', b'a' => 't',
            b'C' => 'G', b'c' => 'g',
            b'G' => 'C', b'g' => 'c',
            b'T' => 'A', b't' => 'a',
            _ => 'N',
        })
        .collect()
}

/// Run minimap2 (query -> target) writing PAF to `out`. Returns Ok even if the PAF
/// is empty (no alignments); errors only on process failure.
fn minimap2_paf(mm2: &str, preset: &str, target: &Path, query: &Path, out: &Path, threads: usize) -> Result<()> {
    let f = File::create(out).with_context(|| format!("create {:?}", out))?;
    let status = Command::new(mm2)
        .args(["-x", preset, "-t", &threads.to_string()])
        .arg(target)
        .arg(query)
        .stdout(std::process::Stdio::from(f))
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("run minimap2 ({} {} -> {})", preset, query.display(), target.display()))?;
    if !status.success() {
        anyhow::bail!("minimap2 exited with {}", status);
    }
    Ok(())
}

/// Parse a reads-vs-genes PAF into best (locus_idx, query_coverage) per read id.
/// `locus_idx` is decoded from the target name `L<idx>`.
fn best_locus_per_read(paf: &Path) -> Result<FxHashMap<String, (usize, f64)>> {
    let mut best: FxHashMap<String, (usize, f64)> = FxHashMap::default();
    let content = fs::read_to_string(paf).with_context(|| format!("read {:?}", paf))?;
    for line in content.lines() {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 11 {
            continue;
        }
        let qlen: f64 = c[1].parse().unwrap_or(0.0);
        let qs: f64 = c[2].parse().unwrap_or(0.0);
        let qe: f64 = c[3].parse().unwrap_or(0.0);
        if qlen <= 0.0 {
            continue;
        }
        let cov = (qe - qs) / qlen;
        let target = c[5];
        let idx: usize = match target.strip_prefix('L').and_then(|s| s.parse().ok()) {
            Some(i) => i,
            None => continue,
        };
        let rid = norm_id(c[0]);
        let e = best.entry(rid).or_insert((idx, 0.0));
        if cov > e.1 {
            *e = (idx, cov);
        }
    }
    Ok(best)
}

/// Split filtered reads into per-locus flanking read sets (core/flank per mate).
/// Returns, per locus index, (has_pairs, has_singles).
fn split_reads(
    loci: &[StalledLocus],
    filtered_r1: &Path,
    filtered_r2: &Path,
    workdir: &Path,
    cfg: &ReasmConfig,
) -> Result<Vec<(bool, bool)>> {
    // 1. gene multi-FASTA (record name L<idx>)
    let genes_fa = workdir.join("genes.fa");
    {
        let mut w = BufWriter::new(File::create(&genes_fa)?);
        for (i, loc) in loci.iter().enumerate() {
            writeln!(w, ">L{}\n{}", i, loc.gene_seq)?;
        }
    }

    // 2. map R1 and R2 to the genes separately
    let r1_paf = workdir.join("r1_to_genes.paf");
    let r2_paf = workdir.join("r2_to_genes.paf");
    minimap2_paf(&cfg.minimap2, "sr", &genes_fa, filtered_r1, &r1_paf, cfg.threads)?;
    minimap2_paf(&cfg.minimap2, "sr", &genes_fa, filtered_r2, &r2_paf, cfg.threads)?;
    let r1_best = best_locus_per_read(&r1_paf)?;
    let r2_best = best_locus_per_read(&r2_paf)?;

    // 3. stream both mates in lockstep, route each recruited pair to its locus
    let dirs: Vec<PathBuf> = (0..loci.len()).map(|i| workdir.join(format!("L{}", i))).collect();
    for d in &dirs {
        fs::create_dir_all(d)?;
    }
    // lazily-opened per-locus writers: (R1, R2, S)
    let mut w1: Vec<Option<BufWriter<File>>> = (0..loci.len()).map(|_| None).collect();
    let mut w2: Vec<Option<BufWriter<File>>> = (0..loci.len()).map(|_| None).collect();
    let mut ws: Vec<Option<BufWriter<File>>> = (0..loci.len()).map(|_| None).collect();
    let mut npair = vec![0u64; loci.len()];
    let mut nsing = vec![0u64; loci.len()];

    let mut f1 = FastqFile::open(filtered_r1)?;
    let mut f2 = FastqFile::open(filtered_r2)?;
    let core = cfg.core_cov;
    loop {
        let a = f1.read_next()?;
        let b = f2.read_next()?;
        let (a, b) = match (a, b) {
            (Some(a), Some(b)) => (a, b),
            _ => break,
        };
        let rid = norm_id(&a.name);
        let o1 = r1_best.get(&rid).copied();
        let o2 = r2_best.get(&rid).copied();
        // choose the locus this pair belongs to
        let locus = match (o1, o2) {
            (Some((l1, c1)), Some((l2, c2))) => {
                if l1 == l2 { Some(l1) } else if c1 >= c2 { Some(l1) } else { Some(l2) }
            }
            (Some((l1, _)), None) => Some(l1),
            (None, Some((l2, _))) => Some(l2),
            (None, None) => None,
        };
        let l = match locus {
            Some(l) => l,
            None => continue,
        };
        let r1core = o1.map_or(false, |(li, cv)| li == l && cv >= core);
        let r2core = o2.map_or(false, |(li, cv)| li == l && cv >= core);
        if r1core && r2core {
            continue; // both core: shared bridge — drop
        } else if r1core {
            // keep flanking mate (R2) as a singleton
            let w = ws[l].get_or_insert_with(|| BufWriter::new(File::create(dirs[l].join("flank_S.fq")).unwrap()));
            write_fq(w, &b)?;
            nsing[l] += 1;
        } else if r2core {
            let w = ws[l].get_or_insert_with(|| BufWriter::new(File::create(dirs[l].join("flank_S.fq")).unwrap()));
            write_fq(w, &a)?;
            nsing[l] += 1;
        } else {
            let x1 = w1[l].get_or_insert_with(|| BufWriter::new(File::create(dirs[l].join("flank_R1.fq")).unwrap()));
            write_fq(x1, &a)?;
            let x2 = w2[l].get_or_insert_with(|| BufWriter::new(File::create(dirs[l].join("flank_R2.fq")).unwrap()));
            write_fq(x2, &b)?;
            npair[l] += 1;
        }
    }
    // flush
    drop(w1); drop(w2); drop(ws);
    Ok((0..loci.len()).map(|i| (npair[i] > 0, nsing[i] > 0)).collect())
}

fn write_fq(w: &mut BufWriter<File>, r: &crate::seqio::FastqRecord) -> Result<()> {
    writeln!(w, "@{}\n{}\n+\n{}", r.name, r.seq, r.qual)?;
    Ok(())
}

/// Run SPAdes (`--only-assembler`) on a locus's flanking reads. Resumable: if
/// `spades/contigs.fasta` already exists and is non-empty, it is reused.
fn run_spades(dir: &Path, has_pairs: bool, has_singles: bool, cfg: &ReasmConfig, per_job_threads: usize) -> Result<Option<PathBuf>> {
    let out = dir.join("spades");
    let contigs = out.join("contigs.fasta");
    if contigs.exists() && fs::metadata(&contigs).map(|m| m.len() > 0).unwrap_or(false) {
        return Ok(Some(contigs));
    }
    if !has_pairs && !has_singles {
        return Ok(None);
    }
    let mut cmd = if cfg.spades_python.as_os_str().is_empty() {
        let mut c = Command::new(&cfg.spades_py);
        c.args(["--only-assembler", "-t", &per_job_threads.to_string(), "-m", "8", "-o"]).arg(&out);
        c
    } else {
        let mut c = Command::new(&cfg.spades_python);
        c.arg(&cfg.spades_py).args(["--only-assembler", "-t", &per_job_threads.to_string(), "-m", "8", "-o"]).arg(&out);
        c
    };
    if has_pairs {
        cmd.arg("-1").arg(dir.join("flank_R1.fq")).arg("-2").arg(dir.join("flank_R2.fq"));
    }
    if has_singles {
        cmd.arg("-s").arg(dir.join("flank_S.fq"));
    }
    let status = cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    match status {
        Ok(s) if s.success() && contigs.exists() => Ok(Some(contigs)),
        _ => Ok(None), // spades_fail — skip this locus
    }
}

/// Orient each flanking contig against the seed and glue the ARG gene onto the end
/// nearest the gene span, recording the gene's coordinates on the stitched contig.
fn stitch(loc: &StalledLocus, contigs: &Path, dir: &Path, cfg: &ReasmConfig) -> Result<Vec<StitchedContig>> {
    // orientation info: contig -> (strand, target_start)
    let mut side: FxHashMap<String, (char, usize)> = FxHashMap::default();
    if !loc.seed_seq.is_empty() {
        let seed_fa = dir.join("seed.fa");
        {
            let mut w = BufWriter::new(File::create(&seed_fa)?);
            writeln!(w, ">seed\n{}", loc.seed_seq)?;
        }
        let cpaf = dir.join("contig2seed.paf");
        minimap2_paf(&cfg.minimap2, "asm10", &seed_fa, contigs, &cpaf, 2)?;
        let content = fs::read_to_string(&cpaf)?;
        for line in content.lines() {
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() < 9 {
                continue;
            }
            let strand = c[4].chars().next().unwrap_or('+');
            let ts: usize = c[7].parse().unwrap_or(0);
            side.entry(c[0].to_string()).or_insert((strand, ts));
        }
    }
    let gene = &loc.gene_seq;
    let seedlen = loc.seed_seq.len();
    let mut out = Vec::new();
    let mut reader = FastaReader::open(contigs)?;
    let mut n = 0;
    while let Some(rec) = reader.read_next()? {
        if rec.seq.len() < 200 {
            continue;
        }
        let cname = rec.name.split_whitespace().next().unwrap_or(&rec.name).to_string();
        let mut oriented = rec.seq.clone();
        // default (no seed hit): gene_down -> gene + oriented
        let mut gene_up = false;
        if let Some((strand, ts)) = side.get(&cname) {
            if *strand == '-' {
                oriented = revcomp(&rec.seq);
            }
            gene_up = *ts < seedlen / 2;
        }
        n += 1;
        let (seq, arg_start, arg_end) = if gene_up {
            let start = oriented.len();
            (format!("{}{}", oriented, gene), start, start + gene.len())
        } else {
            (format!("{}{}", gene, oriented), 0, gene.len())
        };
        out.push(StitchedContig {
            name: format!("{}__c{}", loc.key.replace([':', '|', ' '], "_"), n),
            seq,
            arg_start,
            arg_end,
        });
    }
    Ok(out)
}

/// Reassemble a batch of stalled loci. Returns one `StitchedLocus` per input locus
/// that produced at least one stitched flanking contig.
pub fn reassemble_stalled(
    loci: &[StalledLocus],
    filtered_r1: &Path,
    filtered_r2: &Path,
    workdir: &Path,
    cfg: &ReasmConfig,
) -> Result<Vec<StitchedLocus>> {
    if loci.is_empty() {
        return Ok(Vec::new());
    }
    fs::create_dir_all(workdir)?;
    let flags = split_reads(loci, filtered_r1, filtered_r2, workdir, cfg)?;

    let per_job_threads = (cfg.threads / cfg.jobs.max(1)).max(1);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(cfg.jobs.max(1)).build()?;

    // Assemble + stitch each locus; parallel over loci with a bounded pool.
    let results: Vec<Option<StitchedLocus>> = pool.install(|| {
        use rayon::prelude::*;
        (0..loci.len())
            .into_par_iter()
            .map(|i| {
                let loc = &loci[i];
                let dir = workdir.join(format!("L{}", i));
                let (has_pairs, has_singles) = flags[i];
                let contigs = match run_spades(&dir, has_pairs, has_singles, cfg, per_job_threads) {
                    Ok(Some(c)) => c,
                    _ => return None,
                };
                let stitched = match stitch(loc, &contigs, &dir, cfg) {
                    Ok(s) if !s.is_empty() => s,
                    _ => return None,
                };
                Some(StitchedLocus {
                    key: loc.key.clone(),
                    arg_name: loc.arg_name.clone(),
                    arg_class: loc.arg_class.clone(),
                    identity: loc.identity,
                    coverage: loc.coverage,
                    contigs: stitched,
                })
            })
            .collect()
    });

    Ok(results.into_iter().flatten().collect())
}
