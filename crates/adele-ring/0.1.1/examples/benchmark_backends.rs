//! CPU (rayon) vs GPU (wgpu) timing for batch RNS arithmetic across batch sizes.
//!
//! Run with: `cargo run --release --example benchmark_backends`

use std::time::Instant;

use adele_ring::backend::ArithmeticBackend;
use adele_ring::{executor, Basis, RnsBatch, RnsInt};

fn time_backend<F: Fn() -> RnsBatch>(f: F, iters: u32) -> f64 {
    // Warm up.
    for _ in 0..10 {
        let _ = f();
    }
    let start = Instant::now();
    for _ in 0..iters {
        let _ = f();
    }
    start.elapsed().as_secs_f64() * 1e6 / iters as f64 // microseconds per op
}

fn main() {
    let exec = executor();
    let ch = Basis::standard();
    let has_gpu = exec.gpu().is_some();

    println!("== adele-ring :: backend benchmark ({} channels) ==", ch.len());
    println!(
        "GPU available: {}\n",
        if has_gpu {
            exec.gpu().map(|g| g.adapter_name().to_string()).unwrap_or_default()
        } else {
            "no (CPU-only)".to_string()
        }
    );

    println!("{:>10} | {:>14} | {:>12} | winner", "batch_size", "cpu_rayon_us", "gpu_us");
    println!("{}", "-".repeat(56));

    for &size in &[1usize, 16, 128, 1024, 16_384, 65_536] {
        let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, ch.clone()); size]);
        let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, ch.clone()); size]);

        let iters = if size <= 128 { 2000 } else { 100 };
        let cpu_us = time_backend(|| exec.cpu().batch_add(&a, &b), iters);

        let (gpu_str, winner) = if let Some(gpu) = exec.gpu() {
            let gpu_us = time_backend(|| gpu.batch_add(&a, &b), iters);
            let w = if cpu_us <= gpu_us { "CPU" } else { "GPU" };
            (format!("{gpu_us:>12.2}"), w)
        } else {
            ("         n/a".to_string(), "CPU")
        };

        println!("{size:>10} | {cpu_us:>14.2} | {gpu_str} | {winner}");
    }

    println!(
        "\nNote: CPU wins for small batches (GPU upload/dispatch overhead ~100us);\n\
         GPU pulls ahead once the batch is large enough to amortize that fixed cost."
    );
}
