//! Standalone descent benchmark for `aann`'s k=1 all-nearest-neighbour search.
//!
//! Reproduces the NBLAST all-by-all inner loop (`indices[j].query_prepared(
//! &indices[i])` over ordered neuron pairs) on realistic neuron geometry, and
//! measures wall-time AND heap allocations, single- and multi-threaded. The
//! allocation counter (a `#[global_allocator]` wrapping `System`) lives here in
//! the example binary only, so it never touches the library or its consumers.
//!
//! Data comes from `arena/dump_neurons.py` (points + Delaunay tetrahedra in a
//! flat little-endian `.bin`). Run:
//!
//!     python arena/dump_neurons.py 64 arena/neurons.bin
//!     cargo run --release --example bench_descent -- arena/neurons.bin [threads]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

use aann::ndarray::{Array1, Array2};
use aann::{graph_from_simplices, PreparedF64, Workspace};

// ---- allocation-counting global allocator (benchmark binary only) ----------

static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(l.size(), Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        // Count growth reallocs too, so `Vec` warm-up on the first pair is
        // visible (and then drops to zero for the reuse path).
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(new, Relaxed);
        System.realloc(p, l, new)
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn snapshot() -> (usize, usize) {
    (ALLOCS.load(Relaxed), BYTES.load(Relaxed))
}

// ---- .bin loader -----------------------------------------------------------

struct Neuron {
    points: Array2<f64>,
    indptr: Array1<usize>,
    indices: Array1<usize>,
}

fn read_u32(b: &[u8], o: &mut usize) -> u32 {
    let v = u32::from_le_bytes(b[*o..*o + 4].try_into().unwrap());
    *o += 4;
    v
}
fn read_u64(b: &[u8], o: &mut usize) -> u64 {
    let v = u64::from_le_bytes(b[*o..*o + 8].try_into().unwrap());
    *o += 8;
    v
}

fn load(path: &str) -> Vec<Neuron> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let mut o = 0usize;
    let magic = read_u32(&bytes, &mut o);
    let version = read_u32(&bytes, &mut o);
    assert_eq!(magic, 0x4E4E_4141, "bad magic (not an AANN dump)");
    assert_eq!(version, 1, "unsupported dump version");
    let n = read_u64(&bytes, &mut o) as usize;

    let point_counts: Vec<usize> = (0..n).map(|_| read_u64(&bytes, &mut o) as usize).collect();
    let simplex_counts: Vec<usize> = (0..n).map(|_| read_u64(&bytes, &mut o) as usize).collect();

    // Points section: neuron-contiguous, 3 f64 per point.
    let mut neurons: Vec<Neuron> = Vec::with_capacity(n);
    let mut point_blocks: Vec<Array2<f64>> = Vec::with_capacity(n);
    for &np in &point_counts {
        let mut buf = Vec::with_capacity(np * 3);
        for _ in 0..np * 3 {
            let v = f64::from_le_bytes(bytes[o..o + 8].try_into().unwrap());
            o += 8;
            buf.push(v);
        }
        point_blocks.push(Array2::from_shape_vec((np, 3), buf).unwrap());
    }

    // Simplices section: neuron-contiguous, 4 u64 per tetrahedron. Turn each
    // into a CSR graph via the library's own `graph_from_simplices`.
    for (i, &nt) in simplex_counts.iter().enumerate() {
        let mut buf = Vec::with_capacity(nt * 4);
        for _ in 0..nt * 4 {
            buf.push(read_u64(&bytes, &mut o));
        }
        let simplices = Array2::from_shape_vec((nt, 4), buf).unwrap();
        let np = point_counts[i];
        let (indptr, indices) = graph_from_simplices(simplices.view(), np);
        let points = std::mem::replace(&mut point_blocks[i], Array2::zeros((0, 3)));
        neurons.push(Neuron { points, indptr, indices });
    }
    neurons
}

// ---- benchmark harness -----------------------------------------------------

struct Report {
    name: &'static str,
    threads: usize,
    ms: f64,
    allocs: usize,
    bytes: usize,
    checksum: usize,
}

fn run<F: FnOnce() -> usize>(name: &'static str, threads: usize, f: F) -> Report {
    let (a0, b0) = snapshot();
    let t = Instant::now();
    let checksum = f();
    let ms = t.elapsed().as_secs_f64() * 1e3;
    let (a1, b1) = snapshot();
    Report { name, threads, ms, allocs: a1 - a0, bytes: b1 - b0, checksum }
}

/// Baseline: allocating `query_prepared` per pair, single-threaded.
fn baseline_st(prep: &[PreparedF64], pairs: &[(usize, usize)]) -> usize {
    let mut acc = 0usize;
    for &(i, j) in pairs {
        let (_d, ix) = prep[j].query_prepared(&prep[i], None);
        acc = acc.wrapping_add(ix[0]);
    }
    std::hint::black_box(acc)
}

/// Reuse: `query_prepared_into` with one Workspace + two Vecs, single-threaded.
fn reuse_st(prep: &[PreparedF64], pairs: &[(usize, usize)]) -> usize {
    let mut ws = Workspace::new();
    let (mut d, mut ix) = (Vec::new(), Vec::new());
    let mut acc = 0usize;
    for &(i, j) in pairs {
        prep[j].query_prepared_into(&prep[i], &mut ws, &mut d, &mut ix, None);
        acc = acc.wrapping_add(ix[0]);
    }
    std::hint::black_box(acc)
}

fn chunks(pairs: &[(usize, usize)], threads: usize) -> Vec<&[(usize, usize)]> {
    let chunk = pairs.len().div_ceil(threads);
    pairs.chunks(chunk.max(1)).collect()
}

/// Baseline, multi-threaded: each worker allocates per pair.
fn baseline_mt(prep: &[PreparedF64], pairs: &[(usize, usize)], threads: usize) -> usize {
    let acc = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for slice in chunks(pairs, threads) {
            let acc = &acc;
            s.spawn(move || {
                let mut local = 0usize;
                for &(i, j) in slice {
                    let (_d, ix) = prep[j].query_prepared(&prep[i], None);
                    local = local.wrapping_add(ix[0]);
                }
                acc.fetch_add(local, Relaxed);
            });
        }
    });
    std::hint::black_box(acc.load(Relaxed))
}

/// Reuse, multi-threaded: each worker owns ONE Workspace + two Vecs.
fn reuse_mt(prep: &[PreparedF64], pairs: &[(usize, usize)], threads: usize) -> usize {
    let acc = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for slice in chunks(pairs, threads) {
            let acc = &acc;
            s.spawn(move || {
                let mut ws = Workspace::new();
                let (mut d, mut ix) = (Vec::new(), Vec::new());
                let mut local = 0usize;
                for &(i, j) in slice {
                    prep[j].query_prepared_into(&prep[i], &mut ws, &mut d, &mut ix, None);
                    local = local.wrapping_add(ix[0]);
                }
                acc.fetch_add(local, Relaxed);
            });
        }
    });
    std::hint::black_box(acc.load(Relaxed))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(String::as_str).unwrap_or("arena/neurons.bin");
    let threads = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4));

    let neurons = load(path);
    let n = neurons.len();
    let total_pts: usize = neurons.iter().map(|nb| nb.points.nrows()).sum();
    let prep: Vec<PreparedF64> = neurons
        .iter()
        .map(|nb| PreparedF64::new(nb.points.view(), nb.indptr.view(), nb.indices.view()))
        .collect();

    let pairs: Vec<(usize, usize)> =
        (0..n).flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j))).collect();

    println!(
        "loaded {n} neurons, {total_pts} points (mean {:.0}/neuron), {} ordered pairs, {threads} threads",
        total_pts as f64 / n as f64,
        pairs.len()
    );

    // Warm the OS/allocator and CPU caches once (not measured).
    std::hint::black_box(baseline_st(&prep, &pairs[..pairs.len().min(n)]));

    let reports = vec![
        run("query_prepared       ", 1, || baseline_st(&prep, &pairs)),
        run("query_prepared_into  ", 1, || reuse_st(&prep, &pairs)),
        run("query_prepared       ", threads, || baseline_mt(&prep, &pairs, threads)),
        run("query_prepared_into  ", threads, || reuse_mt(&prep, &pairs, threads)),
    ];

    let st_base = reports[0].ms;
    let mt_base = reports[2].ms;
    println!("\n{:<22} {:>7} {:>10} {:>12} {:>12} {:>9}", "variant", "threads", "wall_ms", "allocs", "bytes", "speedup");
    println!("{}", "-".repeat(76));
    for r in &reports {
        let base = if r.threads == 1 { st_base } else { mt_base };
        println!(
            "{:<22} {:>7} {:>10.1} {:>12} {:>12} {:>8.2}x",
            r.name,
            r.threads,
            r.ms,
            r.allocs,
            r.bytes,
            base / r.ms,
        );
        std::hint::black_box(r.checksum);
    }
    println!(
        "\nallocs/pair: query_prepared st = {:.2}",
        reports[0].allocs as f64 / pairs.len() as f64
    );
}
