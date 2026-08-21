use adler32_checksum_rs::adler32::{Adler32, Adler32Builder};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use std::time::Duration;

fn generate_random_hash() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.gen::<u8>()).collect()
}

fn adler32_checksum_parallel(init: [u8; 8], data: Vec<Vec<u8>>) {
    Adler32Builder::new(Adler32::new(init))
        .push_vec(data)
        .finalize();
}

fn adler32_checksum_blocking(init: [u8; 8], data: Vec<Vec<u8>>) {
    let adler = Adler32::new(init);
    data.iter().for_each(|hash| {
        adler.adler32_checksum(hash);
    })
}

fn adler32_checksum_parallel_benchmark(c: &mut Criterion) {
    let count: usize = 100_000;

    let init: [u8; 8] = [0u8; 8];
    let data_random: Vec<Vec<u8>> = (0..=count).map(|_| generate_random_hash()).collect();
    let data: Vec<Vec<u8>> = (0..=count).map(|x| [x as u8; 32].to_vec()).collect();

    let mut group = c.benchmark_group("Adler32");

    group.bench_function("Checksum parallel randomized", |b| {
        b.iter(|| adler32_checksum_parallel(init, data_random.clone()))
    });
    group.bench_function("Checksum parallel", |b| {
        b.iter(|| adler32_checksum_parallel(init, data.clone()))
    });
    group.bench_function("Checksum blocking randomized", |b| {
        b.iter(|| adler32_checksum_blocking(init, data_random.clone()))
    });
    group.bench_function("Checksum blocking", |b| {
        b.iter(|| adler32_checksum_blocking(init, data.clone()))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(1_000).measurement_time(Duration::from_secs(60));
    targets = adler32_checksum_parallel_benchmark
}
criterion_main!(benches);
