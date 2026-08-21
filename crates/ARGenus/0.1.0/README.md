# ARGenus

**Taxonomic inference of antibiotic resistance genes (ARGs) from metagenomic data using flanking sequence analysis**

[![Crates.io](https://img.shields.io/crates/v/argenus.svg)](https://crates.io/crates/argenus)
[![License](https://img.shields.io/crates/l/argenus.svg)](LICENSE)

ARGenus is a bioinformatics tool that simultaneously detects antibiotic resistance genes (ARGs) and identifies their source bacterial genera from metagenomic sequencing data. Unlike existing tools that only detect ARGs, ARGenus provides direct ARG-to-genus linkage through flanking sequence analysis.

## Features

- **Direct ARG-genus linkage**: Identifies the bacterial source of each detected ARG
- **Targeted assembly**: Efficient processing through read filtering and localized assembly
- **SNP verification**: Filters false positives by confirming resistance-conferring mutations
- **High compression**: 450-fold compressed flanking database (27GB → 60MB)
- **Fast processing**: 5-10 minutes per sample with 16 threads

## Installation

### From crates.io

```bash
cargo install argenus
```

### From source

```bash
git clone https://github.com/necoli1822/ARGenus.git
cd argenus
cargo build --release
```

### Dependencies

ARGenus requires the following tools in your PATH:
- [minimap2](https://github.com/lh3/minimap2) - sequence alignment
- [MEGAHIT](https://github.com/voutcn/megahit) - metagenomic assembly

## Database Setup

ARGenus requires a flanking sequence database for genus classification. Download the pre-built database:

```bash
# Download flanking database (approximately 60MB)
argenus download-db --output databases/
```

Or build from source genomes (requires NCBI RefSeq data):

```bash
argenus build-db --genomes /path/to/refseq --output databases/flanking.fdb
```

## Usage

### Basic usage

```bash
argenus run \
    --r1 sample_R1.fastq.gz \
    --r2 sample_R2.fastq.gz \
    --db databases/AMR_PanRes.mmi \
    --fdb databases/flanking.fdb \
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
| contig_length | Length of assembled contig |
| snp_status | SNP verification result |

## Workflow

1. **Read Filtering**: Align reads against ARG database using minimap2
2. **De Novo Assembly**: Assemble filtered reads with MEGAHIT
3. **Contig Extension**: Extend contigs using k-mer overlap analysis
4. **ARG Detection**: Identify ARGs in extended contigs
5. **Genus Classification**: Classify genus using flanking sequence homology
6. **SNP Verification**: Confirm resistance-conferring mutations

## Performance

| Metric | Value |
|--------|-------|
| Processing time | 5-10 min/sample (16 threads) |
| Genus classification rate | ~73% |
| False positive reduction | ~72% (via SNP filtering) |
| Database size | 60 MB (compressed) |

## Comparison with Other Tools

| Feature | ARGenus | RGI | KMA | AMRFinderPlus |
|---------|---------|-----|-----|---------------|
| ARG detection | ✓ | ✓ | ✓ | ✓ |
| Genus classification | ✓ | ✗ | ✗ | ✗ |
| SNP verification | ✓ | ✓ | ✗ | ✓ |
| Targeted assembly | ✓ | ✗ | ✗ | ✗ |

## Citation

If you use ARGenus in your research, please cite:

```
[Citation information to be added upon publication]
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Contact

For questions and feedback, please open an issue on GitHub.
