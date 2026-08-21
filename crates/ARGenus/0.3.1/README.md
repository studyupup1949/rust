# ARGenus

**ARG detection with honest genus / species / replicon-context reporting from metagenomes**

[![Crates.io](https://img.shields.io/crates/v/argenus.svg)](https://crates.io/crates/argenus)
[![License](https://img.shields.io/crates/l/argenus.svg)](LICENSE)

ARGenus detects antibiotic resistance genes (ARGs) in metagenomic reads and, for each
ARG, reports the bacterial **source** it was found in — using the DNA that *flanks*
the gene on the assembled contig. Unlike tools that only detect ARGs, ARGenus links
each ARG to a genus (and, when the flanking is specific enough, a species) and tells
you how much to trust that link.

## What's new in 0.3.1

- **`--db-dir` picks one flanking DB unambiguously.** If a database folder holds more
  than one `.fdb` (e.g. both the 1 kbp and 5 kbp resolutions), ARGenus now stops and
  asks you to choose one with `-f/--flanking-db`, instead of silently loading the
  alphabetically-first file.

## What's new in 0.3.0

ARGs frequently sit on **plasmids and other mobile elements** that move between
genera, so forcing a single "source genus" is often wrong. 0.3.0 replaces the single
genus call with an **honest 4-axis report** so an ambiguous locus looks ambiguous
instead of confidently wrong:

- **Genus** — a single genus, or `multi-genus(N):A/B/C/…` when several genera share
  the flanking within a small identity margin (a promiscuous gene), or `Unknown`.
- **Species** — a stricter, separately-thresholded call (`--species-identity`);
  `multi-species(N):…` / `Unknown` when the flanking isn't near-identical.
- **Context** — replicon of the matched flanking: `plasmid` / `chromosome` /
  `ambiguous` / `NA`, derived from PLSDB provenance of the flanking references.
  `chromosome` + single-genus is trustworthy; `plasmid` + multi-genus is not.
- **Specificity** — how gene-specific (breadth) the flanking evidence is.

Also new:
- **Per-locus reassembly** (`--reassemble`, opt-in): core/flank read split + SPAdes
  to recover a classifiable flanking for stalled loci.
- **Per-locus exports** (`--emit-*`): write gene / flanking sequences (assembled or
  as reads) for the resolved / no-flank-match / gene-not-in-DB classes.
- **Pluggable read filter** (`--mapper strobealign|minimap2|bwa-mem2`).
- **Contig-only mode** (`--classify-contigs`) to classify a pre-assembled FASTA.
- **Bounded-memory extension** — extension consensus is accumulated as per-position
  base counts, and each contig end is capped (`--max-extension`), so runtime and RAM
  no longer blow up on high-coverage/repetitive loci.

## Features

- **Direct ARG→genus/species linkage** through flanking sequence analysis
- **Honest reporting**: multi-genus / multi-species / replicon context, not a forced call
- **Targeted assembly**: read filtering → MEGAHIT → k-mer extension (optional SPAdes reassembly)
- **SNP verification**: confirms resistance-conferring mutations for point-mutation ARGs
- **Compact database**: custom FDB format (zstd-compressed, on-demand gene blocks)
- **Scales with depth**: bounded extension memory; multi-threaded

## Installation

### From crates.io

```bash
cargo install argenus
```

### From source

```bash
git clone https://github.com/necoli1822/ARGenus.git
cd ARGenus
cargo build --release
```

### External tools (must be on `PATH`)

Required to run ARGenus on reads:
- [minimap2](https://github.com/lh3/minimap2) — alignment for ARG detection **and** flanking classification (always used)
- [MEGAHIT](https://github.com/voutcn/megahit) — assembly of the filtered reads
- [strobealign](https://github.com/ksahlin/strobealign) — the **default** read filter; not needed if you run `--mapper minimap2`

Only for opt-in features — **not** needed for a standard run:
- [SPAdes](https://github.com/ablab/spades) — only for `--reassemble`
- [BLAST+](https://blast.ncbi.nlm.nih.gov/) (`blastn`, `blastdbcmd`) — only to *build* the 5,000 bp (`--mode long`) flanking DB; never used at run time, and unnecessary with the pre-built database below

## Databases

ARGenus needs an **ARG reference** (a FASTA) and a **flanking database**
(`.fdb`). These are **not** shipped with the crate (too large) — download the
pre-built set or build your own.

### Download the pre-built database (recommended)

The **1,000 bp** PanRes database is published as a GitHub Release asset. Download,
extract, and point ARGenus at the folder with `-d`:

```bash
# 297 MB download, ~312 MB extracted into ./db/
curl -L -o argenus-db-1kbp-v0.3.0.tar.gz \
  https://github.com/necoli1822/ARGenus/releases/download/db-1kbp-v0.3.0/argenus-db-1kbp-v0.3.0.tar.gz
tar xzf argenus-db-1kbp-v0.3.0.tar.gz       # -> db/

# -d auto-discovers everything inside the folder
argenus -d db/ -1 R1.fq.gz -2 R2.fq.gz -o results/
```

ARGenus ships two flanking-DB resolutions:

- **1,000 bp** (genus-level, high coverage) — the GitHub Release bundle above.
- **5,000 bp** (species-level, higher resolution) — much larger (~9 GB), archived on
  **Zenodo**: <https://doi.org/10.5281/zenodo.21321983> (CC BY-NC 4.0).

**Picking a resolution.** With `-d`, ARGenus loads the single `.fdb` it finds in the
folder. If the folder holds more than one — e.g. both resolutions — it stops and asks
you to choose one **explicitly with `-f`**:

```bash
# 1,000 bp (genus-level)
argenus -f flanking_1kbp.fdb -a db/PanRes_genes_v1.0.0.fa -1 R1.fq.gz -2 R2.fq.gz -o results/
# 5,000 bp (species-level)
argenus -f flanking_5kbp.fdb -a db/PanRes_genes_v1.0.0.fa -1 R1.fq.gz -2 R2.fq.gz -o results/
```

You can also build either database yourself (see below).

> **Database license:** the pre-built database is derived from
> [PanRes v1.0.0](https://doi.org/10.5281/zenodo.8055116) and is licensed
> **[CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/) (non-commercial)** —
> separate from the MIT-licensed ARGenus code. See [NOTICE](NOTICE) and
> [License & citation](#license--citation).

The bundle contains:

| File | Size | Role |
|------|------|------|
| `PanRes_genes_v1.0.0.fa` | 13 MB | ARG reference (FASTA) |
| `flanking_1kbp.fdb` | 293 MB | Flanking database (**required**) |
| `plasmid_contigs.txt` | 372 KB | Context axis (plasmid/chromosome) |
| `contig_species.tsv` | 6.9 MB | Species axis |

`-d/--db-dir` auto-discovers the ARG reference, the flanking `.fdb`, and the optional
`plasmid_contigs.txt` / `contig_species.tsv` inside the folder. Explicit
`-a/-f/--plasmid-contigs/--species-map` still override what's found there.

> **ARG reference format.** The reference ships as a **FASTA**, which every aligner can
> use: ARGenus indexes it for minimap2 on the fly (the correct `-x asm20` / `-x sr`
> preset per step) and lets strobealign / bwa-mem2 build their own index. You may pass a
> prebuilt minimap2 `.mmi` to `-a` for a small speed-up, but it must be built with
> ARGenus's presets or minimap2 will override them — so the FASTA is the recommended,
> foolproof choice.

### Build your own

```bash
# Build the ARG reference (e.g. from PanRes)
argenus -b arg -x panres -o databases/

# Build a 1,000 bp flanking DB (GenBank/PLSDB)
argenus -b flank --mode short -a databases/PanRes_genes_v1.0.0.fa -o databases/ -e you@email.com

# Compress an existing flanking TSV to FDB (external-sort build)
argenus -b fdb -a flanking.tsv -o databases/flanking_1kbp.fdb
```

The two side files (`plasmid_contigs.txt`, `contig_species.tsv`) are auto-loaded from
beside the `.fdb` if present, and enable the Context and Species axes respectively. You
can also pass them explicitly with `--plasmid-contigs` / `--species-map`.

## Usage

```bash
# Single sample (with a database folder — see Databases below)
argenus -d db/ -1 R1.fq.gz -2 R2.fq.gz -o results/

# ...or point at each database file explicitly
argenus -1 R1.fq.gz -2 R2.fq.gz \
    -a databases/PanRes_genes_v1.0.0.fa \
    -f databases/flanking_1kbp.fdb \
    -o results/

# Batch: a directory (auto-detect *_R[12].fastq.gz) or an ID-list file
argenus -l fastq_dir/ -d db/ -o results/
```

Results are written to `results/results.tsv`.

### Common options

| Option | Default | Description |
|--------|---------|-------------|
| `-1, -2 <FILE>` | — | Paired FASTQ(.gz); comma-separated for multiple samples |
| `-l, --samples <PATH>` | — | Batch: ID-list file or directory of FASTQs |
| `-a, --arg-db <FILE>` | — | ARG reference (`.mmi` or FASTA) |
| `-f, --flanking-db <FILE>` | — | Flanking database (`.fdb`) |
| `-d, --db-dir <DIR>` | — | Auto-discover ARG ref + flanking `.fdb` (+ side files) in one folder |
| `-o, --outdir <DIR>` | `.` | Output directory |
| `-t, --threads <N>` | auto | Total threads |
| `--mapper <TOOL>` | `strobealign` | Read filter: `strobealign` / `minimap2` / `bwa-mem2` |
| `--ref-fasta <FILE>` | derived | FASTA reference for strobealign/bwa-mem2 |
| `-i, --arg-identity <F>` | `0.80` | Min identity for ARG detection |
| `-c, --arg-coverage <F>` | `0.70` | Min coverage for ARG detection |
| `-n, --max-flanking <BP>` | `1000` | Flanking length used for classification |
| `-u, --keep-temp` | off | Keep per-sample intermediates |
| `-v, --verbose` | off | Progress to stderr |

### Honest-reporting options

| Option | Default | Description |
|--------|---------|-------------|
| `--genus-identity <F>` | `0.90` | Min flanking identity to separate genera |
| `--species-identity <F>` | `0.96` | Min flanking identity to call species (0 disables) |
| `--context-plasmid-frac <F>` | `0.5` | Plasmid-fraction ≥ this → Context `plasmid` |
| `--context-chromosome-frac <F>` | `0.1` | Plasmid-fraction ≤ this → Context `chromosome` |
| `--plasmid-contigs <FILE>` | auto | Plasmid accessions for the Context axis |
| `--species-map <FILE>` | auto | `contig<TAB>species` for the Species axis |

### Assembly / reassembly / exports

| Option | Default | Description |
|--------|---------|-------------|
| `--max-extension <BP>` | `0` (=2×max-flanking) | Cap bp added to each contig end (stops runaway extension; no effect on classification) |
| `--reassemble` | off | Per-locus core/flank reassembly (SPAdes) for stalled loci |
| `--spades-path <FILE>` | `spades.py` | SPAdes for `--reassemble` |
| `--reassemble-jobs <N>` | `4` | Concurrent SPAdes jobs |
| `--classify-contigs <FASTA>` | — | Classify a pre-assembled contig FASTA (skip read pipeline) |
| `--emit-class/-part/-state <LIST>` | — | Per-locus FASTA exports (opt-in) |

Run `argenus --help` for the complete list.

## Output format (`results.tsv`)

Tab-delimited, one row per ARG locus:

| Column | Description |
|--------|-------------|
| Sample | Sample identifier |
| Contig_ID | Contig identifier |
| ARG_Name | ARG gene name |
| ARG_Class | Antimicrobial class |
| **Genus** | Source genus, or `multi-genus(N):…`, or `Unknown` |
| **Species** | Source species, or `multi-species(N):…`, or `Unknown` |
| Confidence | Mean flanking identity of the call |
| Specificity | Gene-specificity (breadth) of the flanking evidence |
| **Context** | `plasmid` / `chromosome` / `ambiguous` / `NA` |
| ARG_Identity | ARG sequence identity |
| ARG_Coverage | ARG sequence coverage |
| Contig_Len | Assembled contig length |
| Upstream_Len | Upstream flanking length |
| Downstream_Len | Downstream flanking length |
| Extension_Method | `strict` / `flexible` / `reassemble` |
| SNP_Status | SNP verification status (point-mutation ARGs) |
| Top_Matches | Top genus candidates with scores |

**Reading it:** `chromosome` + single Genus + single Species = trustworthy.
`multi-genus(N)` and/or `plasmid` = a promiscuous / mobile gene — the genus is not a
reliable single source.

## License & citation

ARGenus uses **two different licenses** — one for the code, one for the database:

| Component | License | Notes |
|-----------|---------|-------|
| ARGenus source code | [MIT](LICENSE) | Free for any use, including commercial |
| Pre-built database (`argenus-db-*.tar.gz`) | [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/) | Derived from PanRes — **non-commercial only** |

The pre-built database is derived from the **PanRes v1.0.0** resistance-gene collection
([Zenodo, DOI 10.5281/zenodo.8055116](https://doi.org/10.5281/zenodo.8055116), CC BY-NC 4.0), which aggregates
ResFinder, ResFinderFG, CARD, MEGARes, NCBI AMRFinderPlus, ARG-ANNOT, and BacMet.
See [NOTICE](NOTICE) for full attribution and the modifications ARGenus makes.

If you use ARGenus, please cite this tool (citation to be added upon publication)
**and** the PanRes / ARGprofiler database:

```
Martiny H-M, et al. ARGprofiler—a pipeline for large-scale analysis of antimicrobial
resistance genes and their flanking regions in metagenomic datasets.
Bioinformatics 40(3):btae086 (2024). https://doi.org/10.1093/bioinformatics/btae086
```

## Contact

Created and maintained by **Sunju Kim** ([ORCID 0000-0002-2384-2425](https://orcid.org/0000-0002-2384-2425)).
Questions and bug reports: open an issue at https://github.com/necoli1822/ARGenus
