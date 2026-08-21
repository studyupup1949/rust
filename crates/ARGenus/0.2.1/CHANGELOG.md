# Changelog

All notable changes to ARGenus will be documented in this file.

## [0.2.1] - 2026-02-14

### Added

- **Contig_ID column**: New `Contig_ID` column in output TSV after `Sample` column
  - Links each ARG detection to its source contig (e.g., contig_1, contig_2)
  - Enables tracing ARG variants back to assembled contigs

### Changed

- **Package naming**: Standardized to `ARGenus` (capital ARG) across all configurations
- **Documentation**: Updated README with Contig_ID in Output Format table

## [0.2.0] - 2026-02-13

### Added

- **Dual database mode**: New `--mode short|long` option for flanking database building
  - `short` mode (1,000 bp): High coverage (97.6%) from GenBank + PLSDB
  - `long` mode (5,000 bp): High resolution (92.8%) from NCBI nt_prok
- **New source file**: `flanking_db_ntprok.rs` for 5,000 bp database construction using BLASTN
- **Streaming FDB builder**: Memory-efficient processing for large datasets
  - `--sorted` flag for pre-sorted input (streaming mode)
  - External merge sort support
  - Works with 8-16 GB RAM for 190+ GB datasets
- **Auto-download taxdump**: Automatic download of NCBI taxonomy files if not present

### Changed

- **FDB format v2**: Enhanced binary format with improved compression
  - ~22x compression ratio (194 GB → 8.7 GB for 5,000 bp database)
  - O(1) random access via gene name index
- **Improved classifier**: Enhanced genus classification with confidence metrics
- **Updated dependencies**: All dependencies updated to latest stable versions

### Database Statistics

| Database | Records | Genes | Coverage | Genus Resolution |
|----------|---------|-------|----------|------------------|
| 1,000 bp | 1,069,848 | 11,835 | 97.6% | 83.9% |
| 5,000 bp | 23,184,244 | 11,092 | 91.5% | 92.8% |

### Performance

- Database query: sub-millisecond per ARG match
- FDB building: ~700 MB peak memory (streaming mode)
- Processing: 5-10 minutes per sample (16 threads)

## [0.1.5] - 2026-02-06

### Initial Release

- ARG detection using minimap2 alignment
- Genus classification via flanking sequence analysis
- SNP verification for point mutation ARGs
- Targeted assembly workflow with MEGAHIT
- Compressed FDB format for flanking database
