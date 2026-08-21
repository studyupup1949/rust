//! Comparison bench against [`yrs`] — the Rust port of Yjs.
//!
//! `yrs` is treated as the reference for "what a mature, production CRDT
//! should look like." We benchmark identical workloads against
//! [`abyo_crdt::List<char>`] and report the ratio. Numbers are
//! single-machine, single-thread, criterion-driven.
#![allow(missing_docs)]
#![allow(clippy::items_after_statements)]

use abyo_crdt::List;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn as _, Text, Transact};

fn bench_append_chars(c: &mut Criterion) {
    let mut group = c.benchmark_group("append_chars");
    for &size in &[100usize, 1_000, 5_000] {
        group.bench_with_input(BenchmarkId::new("abyo", size), &size, |b, &n| {
            b.iter(|| {
                let mut list = List::<char>::new(1);
                for i in 0..n {
                    list.insert(i, 'x');
                }
                criterion::black_box(list);
            });
        });
        group.bench_with_input(BenchmarkId::new("yrs", size), &size, |b, &n| {
            b.iter(|| {
                let doc = Doc::new();
                let text = doc.get_or_insert_text("t");
                let mut txn = doc.transact_mut();
                for i in 0..n {
                    text.insert(&mut txn, i as u32, "x");
                }
                criterion::black_box(text.get_string(&txn));
            });
        });
    }
    group.finish();
}

fn bench_random_inserts(c: &mut Criterion) {
    use rand::{Rng, SeedableRng};
    let mut group = c.benchmark_group("random_inserts");
    for &size in &[100usize, 1_000] {
        group.bench_with_input(BenchmarkId::new("abyo", size), &size, |b, &n| {
            b.iter(|| {
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
                let mut list = List::<char>::new(1);
                for _ in 0..n {
                    let pos = if list.is_empty() {
                        0
                    } else {
                        rng.gen_range(0..=list.len())
                    };
                    list.insert(pos, 'x');
                }
                criterion::black_box(list);
            });
        });
        group.bench_with_input(BenchmarkId::new("yrs", size), &size, |b, &n| {
            b.iter(|| {
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
                let doc = Doc::new();
                let text = doc.get_or_insert_text("t");
                let mut txn = doc.transact_mut();
                for _ in 0..n {
                    let len = text.len(&txn) as usize;
                    let pos = if len == 0 { 0 } else { rng.gen_range(0..=len) };
                    text.insert(&mut txn, pos as u32, "x");
                }
                criterion::black_box(text.get_string(&txn));
            });
        });
    }
    group.finish();
}

fn bench_two_replica_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("two_replica_merge");
    for &size in &[100usize, 500] {
        group.bench_with_input(BenchmarkId::new("abyo", size), &size, |b, &n| {
            b.iter(|| {
                let mut a = List::<char>::new(1);
                let mut bb = List::<char>::new(2);
                for i in 0..n {
                    a.insert(i, 'a');
                    bb.insert(i, 'b');
                }
                let snap_a = a.clone();
                a.merge(&bb);
                bb.merge(&snap_a);
                criterion::black_box((a, bb));
            });
        });
        group.bench_with_input(BenchmarkId::new("yrs", size), &size, |b, &n| {
            b.iter(|| {
                let doc_a = Doc::new();
                let text_a = doc_a.get_or_insert_text("t");
                let doc_b = Doc::new();
                let text_b = doc_b.get_or_insert_text("t");
                {
                    let mut txn = doc_a.transact_mut();
                    for i in 0..n {
                        text_a.insert(&mut txn, i as u32, "a");
                    }
                }
                {
                    let mut txn = doc_b.transact_mut();
                    for i in 0..n {
                        text_b.insert(&mut txn, i as u32, "b");
                    }
                }
                let update_b = doc_b
                    .transact()
                    .encode_state_as_update_v1(&yrs::StateVector::default());
                let update_a = doc_a
                    .transact()
                    .encode_state_as_update_v1(&yrs::StateVector::default());
                {
                    let mut txn = doc_a.transact_mut();
                    txn.apply_update(yrs::Update::decode_v1(&update_b).unwrap())
                        .unwrap();
                }
                {
                    let mut txn = doc_b.transact_mut();
                    txn.apply_update(yrs::Update::decode_v1(&update_a).unwrap())
                        .unwrap();
                }
                criterion::black_box((
                    text_a.get_string(&doc_a.transact()),
                    text_b.get_string(&doc_b.transact()),
                ));
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_append_chars,
    bench_random_inserts,
    bench_two_replica_merge
);
criterion_main!(benches);
