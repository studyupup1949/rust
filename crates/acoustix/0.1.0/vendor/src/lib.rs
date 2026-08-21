//! # Acoustix
//!
//! `acoustix` is a high-performance, open-source Rust library for the automated validation and
//! evaluation of Text-to-Speech (TTS) and voice AI models.
//!
//! It provides tools to calculate objective speech quality and similarity metrics without
//! requiring human raters or heavy machine learning models.
//!
//! ## Core Features
//!
//! - **Objective Quality Metrics** (`quality` module): Segmental Signal-to-Noise Ratio (SegSNR) and Log Spectral Distance (LSD).
//! - **Objective Similarity Metrics** (`similarity` module): Mel Cepstral Distortion (MCD) aligned via Dynamic Time Warping (DTW) over extracted MFCCs.
//! - **Preference and Ranking Engines** (`ranking` module): Automated pairwise model comparison and multi-metric model rank aggregation.
//! - **Transcription & Speaker Evaluation** (`evaluation` module): Word Error Rate (WER), Character Error Rate (CER), speaker similarity (SIM), and speaker attribution accuracy (ACC).
//! - **Advanced TTS Quality Controls** (`advanced` module): Fréchet Distance embedding calculations, F0 pitch tracking, F0 RMSE, Pearson correlation, glitch/transient detection, and duration checks.
//! - **Custom Safe Error Handling** (`error` module): Robust custom error definitions built on `thiserror`.
#![deny(
    warnings,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    unreachable_pub,
    deprecated,
    unknown_lints,
    unreachable_code,
    unused_mut,
    non_camel_case_types
)]

/// Advanced objective speech metrics, pitch tracking, and anomaly detection.
pub mod advanced;
/// Custom error handling types using `thiserror`.
pub mod error;
/// Transcription assessment (WER, CER) and speaker classification accuracy.
pub mod evaluation;
/// Signal quality metrics (SNR, LSD).
pub mod quality;
/// Automated preference and multi-metric ranking aggregation.
pub mod ranking;
/// Spectral similarity calculations (MFCC, MCD) and alignment (DTW).
pub mod similarity;
