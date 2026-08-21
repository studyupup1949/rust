//! Criterion benchmarks for the diff engine.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use aaai_core::DiffEngine;

fn setup_tree(n_files: usize) -> (tempfile::TempDir, tempfile::TempDir) {
    let before = tempfile::tempdir().unwrap();
    let after  = tempfile::tempdir().unwrap();
    for i in 0..n_files {
        fs::write(before.path().join(format!("file_{i:04}.txt")), format!("before content {i}\n")).unwrap();
        let content = if i % 5 == 0 {
            format!("after content {i}\n")   // modified
        } else {
            format!("before content {i}\n")  // unchanged
        };
        fs::write(after.path().join(format!("file_{i:04}.txt")), content).unwrap();
    }
    // Some additions
    for i in n_files..n_files + 10 {
        fs::write(after.path().join(format!("new_{i:04}.txt")), "new file\n").unwrap();
    }
    (before, after)
}

fn bench_diff_100(c: &mut Criterion) {
    let (before, after) = setup_tree(100);
    c.bench_function("diff_100_files", |b| {
        b.iter(|| {
            DiffEngine::compare(black_box(before.path()), black_box(after.path())).unwrap()
        })
    });
}

fn bench_diff_1000(c: &mut Criterion) {
    let (before, after) = setup_tree(1000);
    c.bench_function("diff_1000_files", |b| {
        b.iter(|| {
            DiffEngine::compare(black_box(before.path()), black_box(after.path())).unwrap()
        })
    });
}

fn bench_masking(c: &mut Criterion) {
    use aaai_core::MaskingEngine;
    let engine = MaskingEngine::builtin();
    let texts = vec![
        "api_key = \"sk-abcdefghijklmnop12345678\"",
        "password = super_secret_password_123",
        "ordinary log line with no secrets here",
        "AKIAIOSFODNN7EXAMPLE aws secret key",
        "postgres://user:password@host:5432/db",
    ];
    c.bench_function("masking_mixed_texts", |b| {
        b.iter(|| {
            for text in &texts {
                black_box(engine.mask(black_box(text)));
            }
        })
    });
}

criterion_group!(benches, bench_diff_100, bench_diff_1000, bench_masking);
criterion_main!(benches);
