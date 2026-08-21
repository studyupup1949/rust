//! `ad1-forensic` — anomaly auditor for **AccessData AD1** logical images.
//!
//! Emits graded [`forensicnomicon::report::Finding`]s for AD1-specific anomalies:
//! per-file stored-vs-recomputed MD5/SHA1 mismatches (a tamper signal), `ADCRYPT`
//! (encrypted) images, segment-chain gaps, declared-vs-actual size lies, and
//! images that cannot be parsed. Findings are observations with evidence, never
//! assertions of intent.
//!
//! Anomaly `code` strings are a published, scheme-prefixed contract
//! (`AD1-HASH-MISMATCH`, `AD1-ENCRYPTED`, `AD1-SEGMENT-MISSING`, `AD1-SIZE-LIE`,
//! `AD1-UNREADABLE`).

#![forbid(unsafe_code)]

use std::fmt::Write as _;
use std::fs::File;
use std::io::Read as _;
use std::path::Path;

use ad1::{Ad1Entry, Ad1Error, Ad1Reader};
use forensicnomicon::report::{
    Category, Evidence, Finding, Location, Observation, Severity, Source,
};
use md5::{Digest as _, Md5};
use sha1::Sha1;

const ANALYZER: &str = "ad1-forensic";

/// AD1-specific anomaly kinds. Each implements [`Observation`] so it assembles a
/// canonical [`Finding`] via [`Observation::to_finding`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Ad1Anomaly {
    /// The image is `ADCRYPT` (encrypted); content cannot be verified.
    Encrypted {
        /// Leading signature bytes (hex) for the diagnostic.
        signature: String,
    },
    /// The image could not be opened/parsed (corrupt structure, tree cycle, …).
    Unreadable {
        /// The underlying reader error.
        reason: String,
    },
    /// A segment the header declares is absent on disk.
    SegmentMissing {
        /// 1-based segment index.
        index: u32,
        /// Total segments declared by the header.
        total: u32,
    },
    /// A file's stored hash does not match the recomputed hash (tamper signal).
    HashMismatch {
        /// Logical path of the file.
        path: String,
        /// Hash algorithm (`MD5` or `SHA1`).
        algo: &'static str,
        /// Hash stored in the image.
        stored: String,
        /// Hash recomputed from the decompressed content.
        computed: String,
    },
    /// Fewer bytes decompressed than the declared file size.
    SizeLie {
        /// Logical path of the file.
        path: String,
        /// Size declared in the item header.
        declared: u64,
        /// Bytes actually produced by decompression.
        actual: u64,
    },
}

impl Observation for Ad1Anomaly {
    fn severity(&self) -> Option<Severity> {
        Some(match self {
            Ad1Anomaly::Encrypted { .. } => Severity::Info,
            Ad1Anomaly::Unreadable { .. }
            | Ad1Anomaly::SegmentMissing { .. }
            | Ad1Anomaly::HashMismatch { .. }
            | Ad1Anomaly::SizeLie { .. } => Severity::High,
        })
    }

    fn code(&self) -> &'static str {
        match self {
            Ad1Anomaly::Encrypted { .. } => "AD1-ENCRYPTED",
            Ad1Anomaly::Unreadable { .. } => "AD1-UNREADABLE",
            Ad1Anomaly::SegmentMissing { .. } => "AD1-SEGMENT-MISSING",
            Ad1Anomaly::HashMismatch { .. } => "AD1-HASH-MISMATCH",
            Ad1Anomaly::SizeLie { .. } => "AD1-SIZE-LIE",
        }
    }

    fn category(&self) -> Category {
        match self {
            // It is a property of the image, not a structural contradiction.
            Ad1Anomaly::Encrypted { .. } => Category::Provenance,
            // HASH-MISMATCH resolves to Integrity via the code keyword; the rest
            // are structural.
            _ => Category::from_code(self.code()),
        }
    }

    fn note(&self) -> String {
        match self {
            Ad1Anomaly::Encrypted { .. } => "AD1 image is ADCRYPT-encrypted; file contents and \
                 stored hashes cannot be verified (decryption is out of scope)"
                .to_string(),
            Ad1Anomaly::Unreadable { reason } => {
                format!("AD1 image could not be parsed: {reason}")
            }
            Ad1Anomaly::SegmentMissing { index, total } => format!(
                "segment {index} of {total} is absent on disk; data stored in it cannot be read"
            ),
            Ad1Anomaly::HashMismatch { path, algo, .. } => format!(
                "stored {algo} for '{path}' does not match the hash recomputed from its \
                 decompressed content, consistent with modification after acquisition"
            ),
            Ad1Anomaly::SizeLie {
                path,
                declared,
                actual,
            } => format!(
                "'{path}' declares {declared} bytes but only {actual} decompressed, consistent \
                 with a truncated or mislabeled file record"
            ),
        }
    }

    fn evidence(&self) -> Vec<Evidence> {
        match self {
            Ad1Anomaly::Encrypted { signature } => vec![ev("signature", signature)],
            Ad1Anomaly::Unreadable { reason } => vec![ev("error", reason)],
            Ad1Anomaly::SegmentMissing { index, total } => vec![
                ev("missing_segment", &index.to_string()),
                ev("declared_segments", &total.to_string()),
            ],
            Ad1Anomaly::HashMismatch {
                path,
                algo,
                stored,
                computed,
            } => vec![
                ev_at("path", path, Location::Path(path.clone())),
                ev(&format!("stored_{}", algo.to_ascii_lowercase()), stored),
                ev(&format!("computed_{}", algo.to_ascii_lowercase()), computed),
            ],
            Ad1Anomaly::SizeLie {
                path,
                declared,
                actual,
            } => vec![
                ev_at("path", path, Location::Path(path.clone())),
                ev("declared_size", &declared.to_string()),
                ev("actual_size", &actual.to_string()),
            ],
        }
    }
}

fn ev(field: &str, value: &str) -> Evidence {
    Evidence {
        field: field.to_string(),
        value: value.to_string(),
        location: None,
    }
}

fn ev_at(field: &str, value: &str, location: Location) -> Evidence {
    Evidence {
        field: field.to_string(),
        value: value.to_string(),
        location: Some(location),
    }
}

/// Audit an AD1 image (given its first segment) and return forensic findings.
///
/// Detects `ADCRYPT` encryption from the raw signature (the reader refuses to
/// open such images), then opens the image and verifies every file's stored
/// MD5/SHA1 against the recomputed hash, reports missing segments, and flags any
/// declared-vs-actual size lie.
#[must_use]
pub fn audit(first_segment: &Path) -> Vec<Finding> {
    let source = Source {
        analyzer: ANALYZER.to_string(),
        scope: first_segment.display().to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
    };
    collect(first_segment)
        .iter()
        .map(|a| a.to_finding(source.clone()))
        .collect()
}

fn collect(first_segment: &Path) -> Vec<Ad1Anomaly> {
    let mut out = Vec::new();

    // Raw signature peek: an encrypted image cannot be opened by the reader.
    if let Ok(sig) = peek_signature(first_segment) {
        if sig.starts_with(b"ADCRYPT") {
            out.push(Ad1Anomaly::Encrypted {
                signature: hex(&sig),
            });
            return out;
        }
    }

    let reader = match Ad1Reader::open(first_segment) {
        Ok(r) => r,
        Err(Ad1Error::Unsupported(reason)) => {
            // The only Unsupported case today is ADCRYPT.
            out.push(Ad1Anomaly::Encrypted { signature: reason });
            return out;
        }
        Err(e) => {
            out.push(Ad1Anomaly::Unreadable {
                reason: e.to_string(),
            });
            return out;
        }
    };

    let total = reader.segment_count();
    for index in reader.missing_segments() {
        out.push(Ad1Anomaly::SegmentMissing { index, total });
    }

    for entry in reader.entries() {
        if entry.is_dir || (entry.md5.is_none() && entry.sha1.is_none()) {
            continue;
        }
        let Ok((md5, sha1, actual)) = recompute(&reader, entry) else {
            // Read failed (e.g. data in a missing segment) — already reported via
            // SegmentMissing; do not fabricate a hash verdict.
            continue;
        };
        if actual < entry.size {
            out.push(Ad1Anomaly::SizeLie {
                path: entry.path.clone(),
                declared: entry.size,
                actual,
            });
        }
        if let Some(stored) = &entry.md5 {
            if !stored.eq_ignore_ascii_case(&md5) {
                out.push(Ad1Anomaly::HashMismatch {
                    path: entry.path.clone(),
                    algo: "MD5",
                    stored: stored.clone(),
                    computed: md5,
                });
            }
        }
        if let Some(stored) = &entry.sha1 {
            if !stored.eq_ignore_ascii_case(&sha1) {
                out.push(Ad1Anomaly::HashMismatch {
                    path: entry.path.clone(),
                    algo: "SHA1",
                    stored: stored.clone(),
                    computed: sha1,
                });
            }
        }
    }
    out
}

/// Stream `entry`'s content through MD5 + SHA1, returning `(md5, sha1, bytes)`.
fn recompute(reader: &Ad1Reader, entry: &Ad1Entry) -> Result<(String, String, u64), Ad1Error> {
    let mut md5 = Md5::new();
    let mut sha1 = Sha1::new();
    let mut buf = vec![0u8; reader.chunk_size().max(1) as usize];
    let mut offset = 0u64;
    let mut total = 0u64;
    loop {
        let n = reader.read_at(entry, offset, &mut buf)?;
        if n == 0 {
            break;
        }
        md5.update(&buf[..n]);
        sha1.update(&buf[..n]);
        offset += n as u64;
        total += n as u64;
    }
    Ok((hex(&md5.finalize()), hex(&sha1.finalize()), total))
}

fn peek_signature(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut b = [0u8; 16];
    let n = f.read(&mut b)?;
    Ok(b[..n].to_vec())
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_scheme_prefixed_and_stable() {
        let a = Ad1Anomaly::HashMismatch {
            path: "x".into(),
            algo: "MD5",
            stored: "00".into(),
            computed: "11".into(),
        };
        assert_eq!(a.code(), "AD1-HASH-MISMATCH");
        assert_eq!(a.category(), Category::Integrity);
        assert_eq!(a.severity(), Some(Severity::High));
    }

    #[test]
    fn encrypted_is_provenance_info() {
        let a = Ad1Anomaly::Encrypted {
            signature: "4144".into(),
        };
        assert_eq!(a.category(), Category::Provenance);
        assert_eq!(a.severity(), Some(Severity::Info));
    }
}
