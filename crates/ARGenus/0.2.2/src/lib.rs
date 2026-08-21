//! ARGenus - Antibiotic Resistance Gene Detection with Genus Classification
//!
//! A targeted assembly pipeline for detecting ARGs from metagenomic reads
//! and classifying their source genus using flanking sequence analysis.
//!
//! # Modules
//! - `seqio`: FASTA/FASTQ file I/O with gzip support
//! - `paf`: PAF (Pairwise Alignment Format) parsing
//! - `extender`: K-mer based contig extension
//! - `classifier`: Genus classification using flanking sequences
//! - `snp`: SNP verification for point mutation ARGs
//! - `arg_db`: ARG reference database building (NCBI/CARD)
//! - `flanking_db`: Flanking sequence database building
//! - `fdb`: FDB format compression and indexing
//! - `flanking_db_ntprok`: 5000bp flanking database building (nt_prok-based)

pub mod seqio;
pub mod paf;
pub mod extender;
pub mod classifier;
pub mod snp;
pub mod arg_db;
pub mod flanking_db;
pub mod fdb;
pub mod flanking_db_ntprok;
