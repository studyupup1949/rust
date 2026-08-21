use std::time::{Duration, Instant};

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use adaptive_timeout::{
    AdaptiveTimeout, BackoffInterval, LatencyTracker, TimeoutConfig, TrackerConfig,
};

/// Pre-populate a tracker with `n` latency samples spread across `num_dests`
/// destinations.
fn populated_tracker(now: Instant, n: u64, num_dests: u32, latency_ms: u64) -> LatencyTracker<u32> {
    let config = TrackerConfig {
        min_samples: 30,
        ..TrackerConfig::default()
    };
    let mut tracker = LatencyTracker::<u32>::new(config, now);
    for i in 0..n {
        let dest = (i as u32) % num_dests;
        tracker.record_latency_ms(&dest, latency_ms, now);
    }
    tracker
}

// ---------------------------------------------------------------------------
// Benchmark: record_latency_ms (the hot write path)
// ---------------------------------------------------------------------------
fn bench_record_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("record_latency_ms");
    let now = Instant::now();

    for &prefill in &[0, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("prefill", prefill),
            &prefill,
            |b, &prefill| {
                let mut tracker = populated_tracker(now, prefill, 10, 50);
                let mut i = prefill;
                b.iter(|| {
                    let dest = (i as u32) % 10;
                    tracker.record_latency_ms(black_box(&dest), black_box(50), now);
                    i += 1;
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: quantile query (the hot read path)
// ---------------------------------------------------------------------------
fn bench_quantile_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantile_query");
    let now = Instant::now();

    for &samples in &[100, 1000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("samples", samples),
            &samples,
            |b, &samples| {
                let mut tracker = populated_tracker(now, samples, 10, 50);
                b.iter(|| tracker.quantile_ms(black_box(&0u32), black_box(0.9999), now));
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: select_timeout (the full adaptive timeout path)
// ---------------------------------------------------------------------------
fn bench_select_timeout(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_timeout");
    let now = Instant::now();

    let timeout_config = TimeoutConfig {
        backoff: "10ms..60s".parse::<BackoffInterval>().unwrap(),
        quantile: 0.9999,
        safety_factor: 2.0,
    };
    let at = AdaptiveTimeout::new(timeout_config);

    // With data: adaptive path
    for &num_dests in &[1, 3, 10] {
        group.bench_with_input(
            BenchmarkId::new("with_data/dests", num_dests),
            &num_dests,
            |b, &num_dests| {
                let mut tracker = populated_tracker(now, 1000, num_dests, 50);
                let dests: Vec<u32> = (0..num_dests).collect();
                b.iter(|| {
                    at.select_timeout_ms(
                        black_box(&mut tracker),
                        black_box(&dests),
                        black_box(1),
                        now,
                    )
                });
            },
        );
    }

    // Without data: fallback path
    group.bench_function("fallback_no_data", |b| {
        let mut tracker = LatencyTracker::<u32>::with_defaults(now);
        b.iter(|| {
            at.select_timeout_ms(
                black_box(&mut tracker),
                black_box(&[1u32]),
                black_box(1),
                now,
            )
        });
    });

    // Exponential backoff only (no tracker interaction)
    group.bench_function("exponential_backoff_only", |b| {
        b.iter(|| at.exponential_backoff_ms(black_box(3)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: sliding window rotation
// ---------------------------------------------------------------------------
fn bench_rotation(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_rotation");
    let now = Instant::now();

    group.bench_function("record_after_1s_advance", |b| {
        let config = TrackerConfig::default();
        let mut tracker = LatencyTracker::<u32>::new(config, now);

        // Pre-fill
        for _ in 0..100 {
            tracker.record_latency_ms(&0u32, 50, now);
        }

        // Each iteration records at 1.1s after the previous, forcing one
        // sub-window rotation.
        let mut t = now + Duration::from_millis(1100);
        b.iter(|| {
            tracker.record_latency_ms(black_box(&0u32), black_box(50), t);
            t += Duration::from_millis(1100);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Sync benchmarks (feature = "sync")
//
// Compare three approaches for thread-safe access:
//   1. LatencyTracker (bare, no locking — single-threaded baseline)
//   2. Mutex<LatencyTracker> (coarse-grained locking)
//   3. SyncLatencyTracker (DashMap, per-destination fine-grained locking)
//
// Scenarios:
//   - Uncontended: single thread, 1–4 destinations
//   - Contended:   24 threads, 1–4 destinations (realistic service mesh)
// ---------------------------------------------------------------------------

#[cfg(feature = "sync")]
const BG_THREADS: usize = 23; // + 1 measured thread = 24 total

#[cfg(feature = "sync")]
fn populated_sync_tracker(
    now: Instant,
    n: u64,
    num_dests: u32,
    latency_ms: u64,
) -> adaptive_timeout::SyncLatencyTracker<u32> {
    let config = TrackerConfig {
        min_samples: 30,
        ..TrackerConfig::default()
    };
    let tracker = adaptive_timeout::SyncLatencyTracker::<u32>::new(config, now);
    for i in 0..n {
        let dest = (i as u32) % num_dests;
        tracker.record_latency_ms(&dest, latency_ms, now);
    }
    tracker
}

/// Helper: spin up `n` background threads that call `work` in a tight loop
/// until `stop` is set. Returns join handles.
#[cfg(feature = "sync")]
fn spawn_bg_workers<F>(
    n: usize,
    stop: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    work: F,
) -> Vec<std::thread::JoinHandle<()>>
where
    F: Fn(usize) + Send + Sync + 'static,
{
    let work = std::sync::Arc::new(work);
    (0..n)
        .map(|tid| {
            let stop = std::sync::Arc::clone(stop);
            let work = std::sync::Arc::clone(&work);
            std::thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    work(tid);
                }
            })
        })
        .collect()
}

/// Helper: stop background threads and join them.
#[cfg(feature = "sync")]
fn stop_and_join(stop: &std::sync::atomic::AtomicBool, handles: Vec<std::thread::JoinHandle<()>>) {
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    for h in handles {
        h.join().unwrap();
    }
}

// ---------------------------------------------------------------------------
// Uncontended: single thread, sync vs non-sync, 1–4 destinations
// ---------------------------------------------------------------------------
#[cfg(feature = "sync")]
fn bench_uncontended(c: &mut Criterion) {
    use std::sync::Mutex;

    let mut group = c.benchmark_group("uncontended");
    let now = Instant::now();

    let timeout_config = TimeoutConfig {
        backoff: "10ms..60s".parse::<BackoffInterval>().unwrap(),
        quantile: 0.9999,
        safety_factor: 2.0,
    };
    let at = AdaptiveTimeout::new(timeout_config);

    for &num_dests in &[1u32, 2, 4] {
        let dests: Vec<u32> = (0..num_dests).collect();

        // --- record ---

        group.bench_with_input(
            BenchmarkId::new("record/LatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let mut tracker = populated_tracker(now, 1000, nd, 50);
                let mut i = 1000u64;
                b.iter(|| {
                    let dest = (i as u32) % nd;
                    tracker.record_latency_ms(black_box(&dest), black_box(50), now);
                    i += 1;
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("record/Mutex<LatencyTracker>", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Mutex::new(populated_tracker(now, 1000, nd, 50));
                let mut i = 1000u64;
                b.iter(|| {
                    let dest = (i as u32) % nd;
                    tracker
                        .lock()
                        .unwrap()
                        .record_latency_ms(black_box(&dest), black_box(50), now);
                    i += 1;
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("record/SyncLatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = populated_sync_tracker(now, 1000, nd, 50);
                let mut i = 1000u64;
                b.iter(|| {
                    let dest = (i as u32) % nd;
                    tracker.record_latency_ms(black_box(&dest), black_box(50), now);
                    i += 1;
                });
            },
        );

        // --- quantile ---

        group.bench_with_input(
            BenchmarkId::new("quantile/LatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let mut tracker = populated_tracker(now, 1000, nd, 50);
                b.iter(|| tracker.quantile_ms(black_box(&0u32), black_box(0.9999), now));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("quantile/Mutex<LatencyTracker>", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Mutex::new(populated_tracker(now, 1000, nd, 50));
                b.iter(|| {
                    tracker
                        .lock()
                        .unwrap()
                        .quantile_ms(black_box(&0u32), black_box(0.9999), now)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("quantile/SyncLatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = populated_sync_tracker(now, 1000, nd, 50);
                b.iter(|| tracker.quantile_ms(black_box(&0u32), black_box(0.9999), now));
            },
        );

        // --- select_timeout ---

        group.bench_with_input(
            BenchmarkId::new("select_timeout/LatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let mut tracker = populated_tracker(now, 1000, nd, 50);
                let d: Vec<u32> = (0..nd).collect();
                b.iter(|| {
                    at.select_timeout_ms(black_box(&mut tracker), black_box(&d), black_box(1), now)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("select_timeout/Mutex<LatencyTracker>", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Mutex::new(populated_tracker(now, 1000, nd, 50));
                let d: Vec<u32> = (0..nd).collect();
                b.iter(|| {
                    at.select_timeout_ms(
                        black_box(&mut tracker.lock().unwrap()),
                        black_box(&d),
                        black_box(1),
                        now,
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("select_timeout/SyncLatencyTracker", num_dests),
            &num_dests,
            |b, &_nd| {
                let tracker = populated_sync_tracker(now, 1000, num_dests, 50);
                b.iter(|| {
                    at.select_timeout_sync_ms(
                        black_box(&tracker),
                        black_box(&dests),
                        black_box(1),
                        now,
                    )
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Contended: 24 threads, Mutex<LatencyTracker> vs SyncLatencyTracker
//
// The measured thread performs the benchmarked operation while 23 background
// threads simulate realistic RPC traffic: each iteration does one read
// (select_timeout / quantile) then one write (record_latency), mirroring the
// real call pattern where every RPC requires a timeout lookup before the call
// and a latency recording after it completes.
// ---------------------------------------------------------------------------
#[cfg(feature = "sync")]
fn bench_contended_24t(c: &mut Criterion) {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    let mut group = c.benchmark_group("contended_24t");
    // Longer measurement time for contended benchmarks to reduce variance.
    group.measurement_time(Duration::from_secs(10));
    let now = Instant::now();

    let timeout_config = TimeoutConfig {
        backoff: "10ms..60s".parse::<BackoffInterval>().unwrap(),
        quantile: 0.9999,
        safety_factor: 2.0,
    };
    let at_for_mutex = Arc::new(AdaptiveTimeout::new(timeout_config));
    let at_for_sync = Arc::new(AdaptiveTimeout::new(timeout_config));

    for &num_dests in &[1u32, 2, 4] {
        // ---------------------------------------------------------------
        // record: Mutex<LatencyTracker>
        // ---------------------------------------------------------------
        group.bench_with_input(
            BenchmarkId::new("record/Mutex<LatencyTracker>", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Arc::new(Mutex::new(populated_tracker(now, 1000, nd, 50)));
                let stop = Arc::new(AtomicBool::new(false));
                let ctr = Arc::new(AtomicU64::new(0));

                let handles = {
                    let tracker = Arc::clone(&tracker);
                    let ctr = Arc::clone(&ctr);
                    // Each bg iteration = one RPC: read then write.
                    spawn_bg_workers(BG_THREADS, &stop, move |_tid| {
                        let i = ctr.fetch_add(1, Ordering::Relaxed);
                        let dest = (i as u32) % nd;
                        let _ = tracker.lock().unwrap().quantile_ms(&dest, 0.9999, now);
                        tracker.lock().unwrap().record_latency_ms(&dest, 50, now);
                    })
                };

                let mut i = 0u64;
                b.iter(|| {
                    let dest = (i as u32) % nd;
                    tracker
                        .lock()
                        .unwrap()
                        .record_latency_ms(black_box(&dest), black_box(50), now);
                    i += 1;
                });

                stop_and_join(&stop, handles);
            },
        );

        // ---------------------------------------------------------------
        // record: SyncLatencyTracker
        // ---------------------------------------------------------------
        group.bench_with_input(
            BenchmarkId::new("record/SyncLatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Arc::new(populated_sync_tracker(now, 1000, nd, 50));
                let stop = Arc::new(AtomicBool::new(false));
                let ctr = Arc::new(AtomicU64::new(0));

                let handles = {
                    let tracker = Arc::clone(&tracker);
                    let ctr = Arc::clone(&ctr);
                    spawn_bg_workers(BG_THREADS, &stop, move |_tid| {
                        let i = ctr.fetch_add(1, Ordering::Relaxed);
                        let dest = (i as u32) % nd;
                        let _ = tracker.quantile_ms(&dest, 0.9999, now);
                        tracker.record_latency_ms(&dest, 50, now);
                    })
                };

                let mut i = 0u64;
                b.iter(|| {
                    let dest = (i as u32) % nd;
                    tracker.record_latency_ms(black_box(&dest), black_box(50), now);
                    i += 1;
                });

                stop_and_join(&stop, handles);
            },
        );

        // ---------------------------------------------------------------
        // quantile: Mutex<LatencyTracker>
        // ---------------------------------------------------------------
        group.bench_with_input(
            BenchmarkId::new("quantile/Mutex<LatencyTracker>", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Arc::new(Mutex::new(populated_tracker(now, 1000, nd, 50)));
                let stop = Arc::new(AtomicBool::new(false));
                let ctr = Arc::new(AtomicU64::new(0));

                let handles = {
                    let tracker = Arc::clone(&tracker);
                    let ctr = Arc::clone(&ctr);
                    spawn_bg_workers(BG_THREADS, &stop, move |_tid| {
                        let i = ctr.fetch_add(1, Ordering::Relaxed);
                        let dest = (i as u32) % nd;
                        let _ = tracker.lock().unwrap().quantile_ms(&dest, 0.9999, now);
                        tracker.lock().unwrap().record_latency_ms(&dest, 50, now);
                    })
                };

                b.iter(|| {
                    tracker
                        .lock()
                        .unwrap()
                        .quantile_ms(black_box(&0u32), black_box(0.9999), now)
                });

                stop_and_join(&stop, handles);
            },
        );

        // ---------------------------------------------------------------
        // quantile: SyncLatencyTracker
        // ---------------------------------------------------------------
        group.bench_with_input(
            BenchmarkId::new("quantile/SyncLatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Arc::new(populated_sync_tracker(now, 1000, nd, 50));
                let stop = Arc::new(AtomicBool::new(false));
                let ctr = Arc::new(AtomicU64::new(0));

                let handles = {
                    let tracker = Arc::clone(&tracker);
                    let ctr = Arc::clone(&ctr);
                    spawn_bg_workers(BG_THREADS, &stop, move |_tid| {
                        let i = ctr.fetch_add(1, Ordering::Relaxed);
                        let dest = (i as u32) % nd;
                        let _ = tracker.quantile_ms(&dest, 0.9999, now);
                        tracker.record_latency_ms(&dest, 50, now);
                    })
                };

                b.iter(|| tracker.quantile_ms(black_box(&0u32), black_box(0.9999), now));

                stop_and_join(&stop, handles);
            },
        );

        // ---------------------------------------------------------------
        // select_timeout: Mutex<LatencyTracker>
        // ---------------------------------------------------------------
        let dests: Vec<u32> = (0..num_dests).collect();

        group.bench_with_input(
            BenchmarkId::new("select_timeout/Mutex<LatencyTracker>", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Arc::new(Mutex::new(populated_tracker(now, 1000, nd, 50)));
                let stop = Arc::new(AtomicBool::new(false));
                let ctr = Arc::new(AtomicU64::new(0));

                let handles = {
                    let tracker = Arc::clone(&tracker);
                    let ctr = Arc::clone(&ctr);
                    spawn_bg_workers(BG_THREADS, &stop, move |_tid| {
                        let i = ctr.fetch_add(1, Ordering::Relaxed);
                        let dest = (i as u32) % nd;
                        let _ = tracker.lock().unwrap().quantile_ms(&dest, 0.9999, now);
                        tracker.lock().unwrap().record_latency_ms(&dest, 50, now);
                    })
                };

                let d = dests.clone();
                b.iter(|| {
                    at_for_mutex.select_timeout_ms(
                        black_box(&mut tracker.lock().unwrap()),
                        black_box(&d),
                        black_box(1),
                        now,
                    )
                });

                stop_and_join(&stop, handles);
            },
        );

        // ---------------------------------------------------------------
        // select_timeout: SyncLatencyTracker
        // ---------------------------------------------------------------
        group.bench_with_input(
            BenchmarkId::new("select_timeout/SyncLatencyTracker", num_dests),
            &num_dests,
            |b, &nd| {
                let tracker = Arc::new(populated_sync_tracker(now, 1000, nd, 50));
                let stop = Arc::new(AtomicBool::new(false));
                let ctr = Arc::new(AtomicU64::new(0));

                let handles = {
                    let tracker = Arc::clone(&tracker);
                    let ctr = Arc::clone(&ctr);
                    spawn_bg_workers(BG_THREADS, &stop, move |_tid| {
                        let i = ctr.fetch_add(1, Ordering::Relaxed);
                        let dest = (i as u32) % nd;
                        let _ = tracker.quantile_ms(&dest, 0.9999, now);
                        tracker.record_latency_ms(&dest, 50, now);
                    })
                };

                let d = dests.clone();
                b.iter(|| {
                    at_for_sync.select_timeout_sync_ms(
                        black_box(&tracker),
                        black_box(&d),
                        black_box(1),
                        now,
                    )
                });

                stop_and_join(&stop, handles);
            },
        );
    }

    group.finish();
}

#[cfg(not(feature = "sync"))]
criterion_group!(
    benches,
    bench_record_latency,
    bench_quantile_query,
    bench_select_timeout,
    bench_rotation,
);

#[cfg(feature = "sync")]
criterion_group!(
    benches,
    bench_record_latency,
    bench_quantile_query,
    bench_select_timeout,
    bench_rotation,
    bench_uncontended,
    bench_contended_24t,
);

criterion_main!(benches);
