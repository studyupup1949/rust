# ARGenus

**ARG detection and genus-level classification using flanking sequence analysis**

[![Crates.io](https://img.shields.io/crates/v/argenus.svg)](https://crates.io/crates/argenus)
[![License](https://img.shields.io/crates/l/argenus.svg)](LICENSE)

ARGenus is a bioinformatics tool that simultaneously detects antibiotic resistance genes (ARGs) and identifies their source bacterial genera from metagenomic sequencing data. Unlike existing tools that only detect ARGs, ARGenus provides direct ARG-to-genus linkage through flanking sequence analysis.

## Features

- **Direct ARG-genus linkage**: Identifies the bacterial source of each detected ARG
- **Targeted assembly**: Efficient processing through read filtering and localized assembly
- **SNP verification**: Filters false positives by confirming resistance-conferring mutations
- **Dual database modes**: 1,000 bp (high coverage) and 5,000 bp (high resolution) flanking databases
- **High compression**: Custom FDB format with ~22x compression ratio
- **Memory-efficient**: Streaming FDB builder for large datasets (works with 8-16 GB RAM)
- **Fast processing**: 5-10 minutes per sample with 16 threads

## Installation

### From crates.io

```bash
cargo install argenus
```

### From source

```bash
git clone https://github.com/necoli1822/argenus.git
cd argenus
cargo build --release
```

### Dependencies

ARGenus requires the following tools in your PATH:
- [minimap2](https://github.com/lh3/minimap2) - sequence alignment
- [MEGAHIT](https://github.com/voutcn/megahit) - metagenomic assembly

For building the 5,000 bp flanking database:
- [BLAST+](https://blast.ncbi.nlm.nih.gov/doc/blast-help/downloadblastdata.html) - blastn and blastdbcmd

## Database Setup

ARGenus requires a flanking sequence database for genus classification.

### Pre-built Databases

| Database | Size | Coverage | Genus Resolution | Best For |
|----------|------|----------|------------------|----------|
| flanking_1kbp.fdb | ~50 MB | 97.6% | 83.9% | High-throughput screening |
| flanking_5kbp.fdb | ~8.7 GB | 91.5% | 92.8% | Epidemiological studies |

### Building Flanking Database

#### Short mode (1,000 bp) - from GenBank/PLSDB

```bash
argenus -b fdb -m short -o databases/flanking_1kbp.fdb
```

#### Long mode (5,000 bp) - from NCBI nt_prok

```bash
argenus -b fdb -m long \
    -o databases/flanking_5kbp.fdb \
    --blastn /path/to/blastn \
    --blastdbcmd /path/to/blastdbcmd \
    --nt-prok /path/to/nt_prok \
    --taxdump ./taxonomy
```

#### Streaming mode (for large datasets)

For datasets exceeding available RAM, use the streaming mode with external sorting:

```bash
# Step 1: External sort
sort -t'\t' -k1,1 -S 8G --parallel=8 flanking.tsv > flanking_sorted.tsv

# Step 2: Streaming FDB build
argenus -b fdb --sorted -i flanking_sorted.tsv -o flanking.fdb
```

## Usage

### Basic usage

```bash
argenus run \
    --r1 sample_R1.fastq.gz \
    --r2 sample_R2.fastq.gz \
    --db databases/AMR_PanRes.mmi \
    --fdb databases/flanking_1kbp.fdb \
    --output results/sample_argenus.tsv
```

### Options

```
argenus run [OPTIONS]

Required:
    --r1 <FILE>         Forward reads (FASTQ/FASTQ.gz)
    --r2 <FILE>         Reverse reads (FASTQ/FASTQ.gz)
    --db <FILE>         ARG database index (.mmi)
    --fdb <FILE>        Flanking sequence database (.fdb)
    --output <FILE>     Output TSV file

Optional:
    --threads <N>       Number of threads [default: 16]
    --min-identity <F>  Minimum identity for ARG matching [default: 0.8]
    --min-coverage <F>  Minimum coverage for ARG matching [default: 0.7]
    --flank-identity <F> Minimum identity for genus classification [default: 0.9]
    --include-wildtype  Include wild-type alleles in output
```

### Flanking Database Building Options

```
argenus -b fdb [OPTIONS]

Options:
    -m, --mode <MODE>      Database mode: short (1000bp) or long (5000bp)
    -o, --output <PATH>    Output FDB path
    --sorted               Input TSV is pre-sorted by gene name (streaming mode)
    --taxdump <PATH>       Path to NCBI taxdump directory
    --threads <N>          Number of threads [default: available CPUs]
    --buffer-mb <MB>       Sort buffer size in MB [default: 1024]
```

## Output Format

ARGenus produces a tab-delimited file with the following columns:

| Column | Description |
|--------|-------------|
| sample | Sample identifier |
| gene | ARG gene name |
| drug_class | Antimicrobial drug class |
| genus | Assigned source genus |
| confidence | Classification confidence (mean identity) |
| specificity | Gene-genus association strength |
| identity | ARG sequence identity |
| coverage | ARG sequence coverage |
| contig_length | Assembled contig length |
| upstream_len | Upstream flanking sequence length |
| downstream_len | Downstream flanking sequence length |
| extension_method | Extension method used (strict/flexible) |
| snp_status | SNP verification status |

## Performance

- **Processing speed**: 5-10 minutes per sample (10-20M reads, 16 threads)
- **Memory usage**: ~700 MB for FDB building (streaming mode)
- **Classification rate**: ~73% genus-level assignment
- **False positive rate**: <5% (compared to 15% for KMA)

## Database Statistics

### Flanking Database (v3)

| Metric | 1,000 bp | 5,000 bp |
|--------|----------|----------|
| File size | ~50 MB | ~8.7 GB |
| Total records | 1,069,848 | 23,184,244 |
| Gene count | 11,835 | 11,092 |
| Gene coverage | 97.6% | 91.5% |
| Genus resolution | 83.9% | 92.8% |
| Species resolution | 74.7% | 85.2% |

### Data Sources

| Database | Records |
|----------|---------|
| NCBI nt_prok | 8,722,761 sequences |
| GenBank prokaryotic | 85,269 genomes |
| PLSDB | 14,635 plasmids |
| PanRes | 13,280 ARGs |

## Citation

If you use ARGenus in your research, please cite:

```
[Citation information to be added upon publication]
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contact

For questions and bug reports, please open an issue on GitHub.
