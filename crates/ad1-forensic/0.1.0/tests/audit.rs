//! Integration tests for the AD1 forensic auditor.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::format_collect)]

use ad1::testfix;
use ad1_forensic::audit;
use forensicnomicon::report::{Category, Finding, Severity};
use std::path::PathBuf;

fn write_one(bytes: &[u8], name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    (dir, path)
}

fn by_code<'a>(findings: &'a [Finding], code: &str) -> Vec<&'a Finding> {
    findings.iter().filter(|f| f.code == code).collect()
}

#[test]
fn clean_image_reports_no_anomalies() {
    let built = testfix::build(testfix::sample_tree());
    let (_d, p) = write_one(&built.bytes, "clean.ad1");
    let findings = audit(&p);
    assert!(
        findings.is_empty(),
        "clean image should be quiet, got: {:?}",
        findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}

#[test]
fn tampered_stored_hash_is_flagged_as_integrity_anomaly() {
    let built = testfix::build(testfix::sample_tree());
    // Corrupt hello.txt's stored MD5 in the image so it no longer matches content.
    let stored_md5 = built
        .expected
        .iter()
        .find(|e| e.path == "root/hello.txt")
        .unwrap()
        .md5
        .clone()
        .unwrap();
    let mut bytes = built.bytes.clone();
    let pos = find_subslice(&bytes, stored_md5.as_bytes()).expect("stored md5 present");
    // Flip one hex digit (keep it valid hex, same length).
    bytes[pos] = if bytes[pos] == b'a' { b'b' } else { b'a' };

    let (_d, p) = write_one(&bytes, "tampered.ad1");
    let findings = audit(&p);
    let mismatches = by_code(&findings, "AD1-HASH-MISMATCH");
    assert_eq!(mismatches.len(), 1, "exactly one hash mismatch expected");
    let f = mismatches[0];
    assert_eq!(f.severity, Some(Severity::High));
    assert_eq!(f.category, Category::Integrity);
    assert_eq!(f.source.analyzer, "ad1-forensic");
    // Show-the-evidence: both stored and recomputed hashes are present.
    let ev: String = f
        .evidence
        .iter()
        .map(|e| format!("{}={}", e.field, e.value))
        .collect();
    assert!(
        ev.to_lowercase().contains("md5"),
        "evidence names the algo: {ev}"
    );
    assert!(
        f.note.contains("hello.txt"),
        "note names the file: {}",
        f.note
    );
}

#[test]
fn encrypted_image_is_reported_not_opened() {
    let mut bytes = vec![0u8; 512];
    bytes[0..8].copy_from_slice(b"ADCRYPT\0");
    let (_d, p) = write_one(&bytes, "enc.ad1");
    let findings = audit(&p);
    assert_eq!(by_code(&findings, "AD1-ENCRYPTED").len(), 1);
}

#[test]
fn missing_segment_is_reported() {
    let built = testfix::build(testfix::sample_tree());
    let logical_len = built.bytes.len() - testfix::MARGIN;
    let fragments_size = 2u32;
    let stride = (fragments_size as usize) * 0x1_0000 - testfix::MARGIN;
    let n = logical_len.div_ceil(stride) as u32;
    let segs = testfix::split(&built.bytes, n, fragments_size);

    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("gap.ad1");
    std::fs::write(&first, &segs[0]).unwrap(); // only segment 1

    let findings = audit(&first);
    let missing = by_code(&findings, "AD1-SEGMENT-MISSING");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].severity, Some(Severity::High));
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

const MARGIN: usize = 512;
fn phys(logical: u64) -> usize {
    MARGIN + logical as usize
}
fn rd_u64(b: &[u8], p: usize) -> u64 {
    u64::from_le_bytes(b[p..p + 8].try_into().unwrap())
}
fn put_u64(b: &mut [u8], p: usize, v: u64) {
    b[p..p + 8].copy_from_slice(&v.to_le_bytes());
}
fn put_u32(b: &mut [u8], p: usize, v: u32) {
    b[p..p + 4].copy_from_slice(&v.to_le_bytes());
}
fn item_offset(bytes: &[u8], name: &str) -> u64 {
    let first = rd_u64(bytes, phys(0) + 0x24);
    let mut stack = vec![first];
    while let Some(addr) = stack.pop() {
        if addr == 0 {
            continue;
        }
        let base = phys(addr);
        let nl = u32::from_le_bytes(bytes[base + 0x2c..base + 0x30].try_into().unwrap()) as usize;
        if String::from_utf8_lossy(&bytes[base + 0x30..base + 0x30 + nl]) == name {
            return addr;
        }
        stack.push(rd_u64(bytes, base));
        stack.push(rd_u64(bytes, base + 0x08));
    }
    panic!("item {name} not found");
}

#[test]
fn unparseable_image_is_reported_unreadable() {
    let built = testfix::build(testfix::sample_tree());
    let mut bytes = built.bytes;
    put_u32(&mut bytes, 0x22, 0); // fragments_size = 0 -> open fails Malformed
    let (_d, p) = write_one(&bytes, "bad.ad1");
    let findings = audit(&p);
    assert_eq!(by_code(&findings, "AD1-UNREADABLE").len(), 1);
    assert_eq!(findings[0].severity, Some(Severity::High));
}

#[test]
fn declared_size_larger_than_content_is_a_size_lie() {
    let built = testfix::build(testfix::sample_tree());
    let off = item_offset(&built.bytes, "a.bin");
    let mut bytes = built.bytes;
    let real = rd_u64(&bytes, phys(off) + 0x20);
    // Inflate the declared size; the chunks still only yield `real` bytes.
    put_u64(&mut bytes, phys(off) + 0x20, real + 50_000);
    let (_d, p) = write_one(&bytes, "lie.ad1");
    let findings = audit(&p);
    let lies = by_code(&findings, "AD1-SIZE-LIE");
    assert_eq!(lies.len(), 1);
    assert_eq!(lies[0].category, Category::Structure);
}
