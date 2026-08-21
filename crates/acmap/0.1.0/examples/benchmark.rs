use acmap::AcMap;
use dashmap::DashMap;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

const NUM_ITERATIONS: u64 = 1_000_000;
const NUM_WORKERS: u64 = 8;
const BATCH_SIZE: usize = 512;

#[derive(Debug, Clone)]
struct BenchResult {
    name: &'static str,
    elapsed: Duration,
    ops: u64,
}

#[tokio::main]
async fn main() {
    println!(
        "Benchmark config: ops={}, workers={}",
        NUM_ITERATIONS, NUM_WORKERS
    );

    let single_thread = vec![
        run_sync("HashMap", NUM_ITERATIONS, test_hashmap),
        run_sync("DashMap", NUM_ITERATIONS, test_dashmap),
        run_async("AcMap", NUM_ITERATIONS, test_amap).await,
        run_async("AcMap(Fast)", NUM_ITERATIONS, test_amap_fast).await,
        run_async("AcMap(FastBatch)", NUM_ITERATIONS, test_amap_fast_batch).await,
    ];

    let multi_thread = vec![
        run_async(
            "HashMap+Mutex(MT)",
            NUM_ITERATIONS,
            test_hashmap_multi_thread,
        )
        .await,
        run_async("DashMap(MT)", NUM_ITERATIONS, test_dashmap_multi_thread).await,
        run_async("AcMap(MT)", NUM_ITERATIONS, test_amap_multi_thread).await,
        run_async(
            "AcMap(Fast,MT)",
            NUM_ITERATIONS,
            test_amap_fast_multi_thread,
        )
        .await,
        run_async(
            "AcMap(FastBatch,MT)",
            NUM_ITERATIONS,
            test_amap_fast_batch_multi_thread,
        )
        .await,
    ];

    print_section("Single-thread", &single_thread);
    print_section("Multi-thread", &multi_thread);
}

fn run_sync(name: &'static str, expected_len: u64, f: fn() -> usize) -> BenchResult {
    let start = Instant::now();
    let len = f();
    let elapsed = start.elapsed();

    assert_eq!(len as u64, expected_len, "{name} length mismatch");

    BenchResult {
        name,
        elapsed,
        ops: expected_len,
    }
}

async fn run_async<Fut>(name: &'static str, expected_len: u64, f: fn() -> Fut) -> BenchResult
where
    Fut: std::future::Future<Output = usize>,
{
    let start = Instant::now();
    let len = f().await;
    let elapsed = start.elapsed();

    assert_eq!(len as u64, expected_len, "{name} length mismatch");

    BenchResult {
        name,
        elapsed,
        ops: expected_len,
    }
}

fn print_section(title: &str, results: &[BenchResult]) {
    println!("\n=== {title} ===");
    println!(
        "{:<18} {:>12} {:>14} {:>12}",
        "Case", "Elapsed", "Throughput", "Speedup"
    );

    let baseline = results[0].elapsed.as_secs_f64();
    for r in results {
        let secs = r.elapsed.as_secs_f64();
        let mops = (r.ops as f64 / secs) / 1_000_000.0;
        let speedup = baseline / secs;

        println!(
            "{:<18} {:>10.3}s {:>11.3} Mops/s {:>9.2}x",
            r.name, secs, mops, speedup
        );
    }
}

fn test_hashmap() -> usize {
    let mut map = HashMap::new();

    for i in 0..NUM_ITERATIONS {
        map.insert(i, i);
    }

    map.len()
}

fn test_dashmap() -> usize {
    let map = DashMap::new();

    for i in 0..NUM_ITERATIONS {
        map.insert(i, i);
    }

    map.len()
}

async fn test_amap() -> usize {
    let map = AcMap::<u64, u64>::new();

    for i in 0..NUM_ITERATIONS {
        map.insert(i, i).await;
    }

    map.len().await
}

async fn test_amap_fast() -> usize {
    let map = AcMap::<u64, u64>::new();

    for i in 0..NUM_ITERATIONS {
        map.insert_fast(i, i);
    }

    map.len().await
}

async fn test_amap_fast_batch() -> usize {
    let map = AcMap::<u64, u64>::new();
    for start in (0..NUM_ITERATIONS).step_by(BATCH_SIZE) {
        let end = (start + BATCH_SIZE as u64).min(NUM_ITERATIONS);
        map.insert_fast_batch((start..end).map(|i| (i, i)));
    }
    map.len().await
}

fn worker_range(worker: u64) -> (u64, u64) {
    let chunk = NUM_ITERATIONS / NUM_WORKERS;
    let start = worker * chunk;
    let end = if worker == NUM_WORKERS - 1 {
        NUM_ITERATIONS
    } else {
        start + chunk
    };
    (start, end)
}

async fn test_hashmap_multi_thread() -> usize {
    let map = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = Vec::new();

    for worker in 0..NUM_WORKERS {
        let map = Arc::clone(&map);
        let (start, end) = worker_range(worker);
        handles.push(tokio::spawn(async move {
            for i in start..end {
                map.lock().await.insert(i, i);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("hashmap worker panicked");
    }

    map.lock().await.len()
}

async fn test_dashmap_multi_thread() -> usize {
    let map = Arc::new(DashMap::new());
    let mut handles = Vec::new();

    for worker in 0..NUM_WORKERS {
        let map = Arc::clone(&map);
        let (start, end) = worker_range(worker);
        handles.push(tokio::spawn(async move {
            for i in start..end {
                map.insert(i, i);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("dashmap worker panicked");
    }

    map.len()
}

async fn test_amap_multi_thread() -> usize {
    let map = AcMap::<u64, u64>::new();
    let mut handles = Vec::new();

    for worker in 0..NUM_WORKERS {
        let map = map.clone();
        let (start, end) = worker_range(worker);
        handles.push(tokio::spawn(async move {
            for i in start..end {
                map.insert(i, i).await;
            }
        }));
    }

    for handle in handles {
        handle.await.expect("amap worker panicked");
    }

    map.len().await
}

async fn test_amap_fast_multi_thread() -> usize {
    let map = AcMap::<u64, u64>::new();
    let mut handles = Vec::new();

    for worker in 0..NUM_WORKERS {
        let map = map.clone();
        let (start, end) = worker_range(worker);
        handles.push(tokio::spawn(async move {
            for i in start..end {
                map.insert_fast(i, i);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("amap fast worker panicked");
    }

    map.len().await
}

async fn test_amap_fast_batch_multi_thread() -> usize {
    let map = AcMap::<u64, u64>::new();
    let mut handles = Vec::new();

    for worker in 0..NUM_WORKERS {
        let map = map.clone();
        let (start, end) = worker_range(worker);
        handles.push(tokio::spawn(async move {
            for chunk_start in (start..end).step_by(BATCH_SIZE) {
                let chunk_end = (chunk_start + BATCH_SIZE as u64).min(end);
                map.insert_fast_batch((chunk_start..chunk_end).map(|i| (i, i)));
            }
        }));
    }

    for handle in handles {
        handle.await.expect("amap fast batch worker panicked");
    }

    map.len().await
}
