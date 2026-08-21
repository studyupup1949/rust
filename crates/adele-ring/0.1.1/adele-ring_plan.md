# adele-ring — Exact Arithmetic Engine
**Cursor Build Plan**

---

## What This Is

A Rust crate implementing multi-base exact arithmetic via the Residue Number System (RNS).
Instead of a single binary representation, numbers live simultaneously across multiple prime
channels. By the Chinese Remainder Theorem, any integer up to M = ∏(all channels) is
uniquely identified by its residue tuple.

**The RNS channel structure IS the parallelism structure.** Each channel is completely
independent of every other — no carry propagation, no inter-channel communication during
add/mul. This maps perfectly to both CPU threads (rayon) and GPU threads (wgpu):

```
CPU:  channel[0]  channel[1]  channel[2]  ...  channel[31]
      thread 0    thread 1    thread 2    ...   thread 7 (rayon work-stealing)

GPU:  channel[0]  channel[1]  channel[2]  ...  channel[31]   × batch_size
      thread(0,0) thread(0,1) thread(0,2) ...  thread(0,31)  ← item 0
      thread(1,0) thread(1,1) thread(1,2) ...  thread(1,31)  ← item 1
      ...
```

Both backends are first-class. The engine probes for GPU at startup and falls back
to CPU automatically. For small batches (<128 items) CPU wins due to GPU upload
overhead; for large batches GPU wins decisively. The `Executor` makes this choice
transparently.

---

## Crate Name & Workspace

```
adele-ring/
├── Cargo.toml
├── shaders/
│   ├── rns_add.wgsl          ← GPU: one thread per (batch_item × channel)
│   └── rns_mul.wgsl
├── src/
│   ├── lib.rs
│   ├── primes.rs             ← Prime utilities (pure functions)
│   ├── batch.rs              ← RnsBatch: shared flat buffer format for both backends
│   ├── backend.rs            ← ArithmeticBackend trait + Executor (auto CPU/GPU select)
│   ├── rns.rs            ← Level 0: ℤ  (RNS integers — rayon parallel over channels)
│   ├── rational.rs       ← Level 1: ℚ  (exact rationals over RNS)
│   ├── algebraic.rs      ← Level 2: ℚ̄  (algebraic numbers, min poly + interval)
│   ├── computable.rs     ← Level 3: ℝ_c (computable reals, lazy precision-on-demand)
│   ├── symbolic.rs       ← Level 4: 𝒮  (expression DAGs + identity graph)
│   ├── tower.rs          ← Tower-level dispatch: routes ops to cheapest level
│   ├── dispatch.rs       ← RNS channel dispatch (base-aware ALU routing)
│   ├── cpu.rs            ← CpuBackend: rayon parallel channel operations
│   └── gpu.rs            ← GpuBackend: wgpu compute shaders
└── examples/
    ├── engineering.rs
    ├── float_comparison.rs
    └── benchmark_backends.rs ← CPU vs GPU timing at various batch sizes
```

**Cargo.toml dependencies:**
```toml
num-bigint  = "0.4"
num-traits  = "0.2"
num-integer = "0.1"
thiserror   = "1.0"
rayon       = "1.10"       # CPU multi-core channel parallelism
parking_lot = "0.12"       # fast Mutex for ComputableReal precision cache
bytemuck    = { version = "1.14", features = ["derive"] }  # GPU buffer casting
wgpu        = "22"         # GPU compute — NOT optional; graceful CPU fallback at runtime
pollster    = "0.3"        # block_on for wgpu async init

[features]
default = []
# No gpu feature flag — wgpu probes hardware at runtime and falls back cleanly.
# If wgpu finds no compatible GPU adapter it returns Err; Executor uses CpuBackend instead.
```

---

## Module 0a — `batch.rs`

**Purpose:** `RnsBatch` — the shared flat buffer format used by both backends.
Having one layout means no reformatting when switching between CPU rayon and GPU.
The GPU buffer IS this struct's data; the CPU just iterates it with rayon.

```rust
/// Flat row-major buffer: [batch_size × n_channels]
/// Element [b * n_channels + c] = residue of item b in channel c.
/// This layout matches the GPU buffer exactly — no reformatting on upload.
pub struct RnsBatch {
    pub data:       Vec<u64>,      // flat: [B × K]
    pub batch_size: usize,         // B
    pub channels:   Channels,      // K channels
}

impl RnsBatch {
    pub fn new(batch_size: usize, channels: Channels) -> Self
    pub fn zeros(batch_size: usize, channels: Channels) -> Self

    /// Index: residue of item `b` in channel `c`
    #[inline] pub fn get(&self, b: usize, c: usize) -> u64
    #[inline] pub fn set(&mut self, b: usize, c: usize, val: u64)

    /// Pack a slice of RnsInt values into a batch.
    pub fn from_rns_ints(items: &[RnsInt]) -> Self

    /// Unpack back to individual RnsInt values.
    pub fn to_rns_ints(&self) -> Vec<RnsInt>

    /// Pack for GPU: convert u64 → u32 (wgpu uses u32 in shaders).
    /// Moduli chosen ≤ 2^31 so residues always fit in u32.
    pub fn as_u32_bytes(&self) -> Vec<u8>   // bytemuck cast, zero-copy
}
```

---

## Module 0b — `backend.rs`

**Purpose:** The `ArithmeticBackend` trait plus the `Executor` which selects CPU or
GPU at runtime. All higher-level math goes through the Executor — it never hard-codes
a backend.

### Trait: `ArithmeticBackend`
```rust
pub trait ArithmeticBackend: Send + Sync {
    /// Elementwise RNS addition: result[b][c] = (a[b][c] + b_batch[b][c]) % m[c]
    fn batch_rns_add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch;

    /// Elementwise RNS multiplication: result[b][c] = (a[b][c] * b_batch[b][c]) % m[c]
    fn batch_rns_mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch;

    /// CRT reconstruction for all items in the batch — returns Vec<BigUint>
    fn batch_crt(&self, batch: &RnsBatch) -> Vec<num_bigint::BigUint>;

    /// Backend name for logging/diagnostics
    fn name(&self) -> &'static str;
}
```

### Type: `Executor`
```rust
pub struct Executor {
    cpu: Arc<CpuBackend>,
    gpu: Option<Arc<GpuBackend>>,
    /// Batch sizes below this threshold use CPU even if GPU is available.
    /// Default: 128. Tune based on GPU upload overhead vs compute time.
    gpu_threshold: usize,
}

impl Executor {
    /// Probe for GPU at startup. If wgpu finds no adapter, gpu = None.
    /// This is the ONLY constructor — always call this, never construct directly.
    pub fn init() -> Self {
        let cpu = Arc::new(CpuBackend::new());
        let gpu = GpuBackend::try_init().ok().map(Arc::new);
        if gpu.is_some() {
            log::info!("adele-ring: GPU backend active ({})", gpu.as_ref().unwrap().adapter_name());
        } else {
            log::info!("adele-ring: no GPU found, using CPU backend");
        }
        Self { cpu, gpu, gpu_threshold: 128 }
    }

    /// Select the best backend for this batch size.
    fn select(&self, batch_size: usize) -> &dyn ArithmeticBackend {
        match &self.gpu {
            Some(g) if batch_size >= self.gpu_threshold => g.as_ref(),
            _ => self.cpu.as_ref(),
        }
    }

    /// Public API — callers never touch backends directly.
    pub fn add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.select(a.batch_size).batch_rns_add(a, b)
    }
    pub fn mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.select(a.batch_size).batch_rns_mul(a, b)
    }
    pub fn crt(&self, batch: &RnsBatch) -> Vec<num_bigint::BigUint> {
        // CRT reconstruction is always CPU — the Garner algorithm is sequential.
        // GPU batch_crt would need a parallel prefix reduction; not worth it for k≤32.
        self.cpu.batch_crt(batch)
    }
}
```

**Global executor (lazily initialized, available crate-wide):**
```rust
use std::sync::OnceLock;
static EXECUTOR: OnceLock<Executor> = OnceLock::new();

pub fn executor() -> &'static Executor {
    EXECUTOR.get_or_init(Executor::init)
}
```

---

## Module 0c — `cpu.rs`

**Purpose:** `CpuBackend` — implements `ArithmeticBackend` using rayon for parallel
channel processing. This is the fallback and is always available.

```rust
pub struct CpuBackend {
    pool: rayon::ThreadPool,
}

impl CpuBackend {
    pub fn new() -> Self {
        Self {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(0)     // 0 = use all logical cores
                .thread_name(|i| format!("adele-ring-cpu-{i}"))
                .build()
                .expect("rayon pool init failed"),
        }
    }
}
```

**Implementation of `batch_rns_add`:**
```rust
fn batch_rns_add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
    use rayon::prelude::*;
    let k = a.channels.len();
    let mut result = RnsBatch::zeros(a.batch_size, a.channels.clone());

    // Outer parallel: batch items. Inner: channels (small k, sequential is fine).
    // For k=32 and batch=1024, this spawns 1024 tasks across all cores.
    result.data
        .par_chunks_mut(k)             // one chunk per batch item
        .enumerate()
        .for_each(|(b_idx, out_row)| {
            for c in 0..k {
                let m   = a.channels.modulus(c);
                let av  = a.get(b_idx, c);
                let bv  = b.get(b_idx, c);
                out_row[c] = (av + bv) % m;
            }
        });
    result
}
```

**Implementation of `batch_rns_mul`:**
Same pattern but uses `u128` for the product to avoid overflow before mod:
```rust
out_row[c] = ((av as u128 * bv as u128) % m as u128) as u64;
```

**Implementation of `batch_crt`:**
```rust
fn batch_crt(&self, batch: &RnsBatch) -> Vec<BigUint> {
    use rayon::prelude::*;
    let k = batch.channels.len();
    (0..batch.batch_size)
        .into_par_iter()
        .map(|b| {
            let residues: Vec<u64> = (0..k).map(|c| batch.get(b, c)).collect();
            garner_crt(&residues, &batch.channels.0)
        })
        .collect()
}
```

**Single-item convenience (used by `RnsInt` directly):**
```rust
impl CpuBackend {
    /// For single-number operations: parallel over channels, not batch.
    /// Only worthwhile when k is large (≥16). For k<8, sequential is faster.
    pub fn rns_add_single(&self, a: &RnsInt, b: &RnsInt) -> RnsInt {
        use rayon::prelude::*;
        let k = a.channels.len();
        let moduli = &a.channels.0;
        let residues: Vec<u64> = if k >= 16 {
            // parallel over channels
            a.residues.par_iter()
                .zip(b.residues.par_iter())
                .zip(moduli.par_iter())
                .map(|((&av, &bv), &m)| (av + bv) % m)
                .collect()
        } else {
            // sequential for small k (rayon overhead not worth it)
            a.residues.iter()
                .zip(b.residues.iter())
                .zip(moduli.iter())
                .map(|((&av, &bv), &m)| (av + bv) % m)
                .collect()
        };
        RnsInt { residues, channels: a.channels.clone(), negative: false }
    }
}
```

The threshold of 16 channels is empirical. Rayon's task overhead (~50ns) vs
one channel op (~1ns): break-even around 8-16 channels. Document this and make
it a const so it can be tuned.

---

## Module 0d — `gpu.rs`

**Purpose:** `GpuBackend` — implements `ArithmeticBackend` using wgpu compute shaders.
Not feature-gated — wgpu is always compiled in. If the hardware probe fails,
`try_init()` returns `Err` and `Executor` uses CPU.

```rust
pub struct GpuBackend {
    device:      wgpu::Device,
    queue:       wgpu::Queue,
    add_pipeline: wgpu::ComputePipeline,
    mul_pipeline: wgpu::ComputePipeline,
    moduli_buf:  wgpu::Buffer,   // pre-uploaded channel moduli; never changes
    adapter_info: wgpu::AdapterInfo,
}

impl GpuBackend {
    pub fn try_init() -> Result<Self, wgpu::RequestDeviceError> {
        pollster::block_on(Self::try_init_async())
    }

    async fn try_init_async() -> Result<Self, wgpu::RequestDeviceError> {
        let instance = wgpu::Instance::default();
        let adapter  = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .ok_or(/* map None to error */)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await?;

        let add_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/rns_add.wgsl"));
        let mul_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/rns_mul.wgsl"));
        // create pipelines, bind group layouts, moduli buffer...

        Ok(Self { device, queue, add_pipeline, mul_pipeline, moduli_buf, adapter_info })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_info.name
    }

    /// Upload a batch to GPU, run shader, download result.
    /// Internally: create_buffer → write_buffer → dispatch → map_async → read.
    fn run_pipeline(
        &self,
        pipeline:   &wgpu::ComputePipeline,
        a:          &RnsBatch,
        b:          &RnsBatch,
    ) -> RnsBatch {
        let k = a.channels.len();
        let b_size = a.batch_size;

        // Upload
        let a_buf = self.upload(a.as_u32_bytes());
        let b_buf = self.upload(b.as_u32_bytes());
        let out_buf = self.alloc_output(b_size * k * 4);

        // Dispatch: workgroup (16, 16) → grid ((B+15)/16, (K+15)/16)
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            // set bind groups with a_buf, b_buf, moduli_buf, out_buf, params...
            pass.dispatch_workgroups(
                (b_size as u32 + 15) / 16,
                (k as u32 + 15) / 16,
                1,
            );
        }
        self.queue.submit([encoder.finish()]);

        // Download and unpack
        self.download_to_rns_batch(out_buf, b_size, a.channels.clone())
    }
}
```

### Shader: `shaders/rns_add.wgsl`
```wgsl
struct Params {
    batch_size: u32,
    n_channels: u32,
}

@group(0) @binding(0) var<uniform>            params:  Params;
@group(0) @binding(1) var<storage, read>      moduli:  array<u32>;
@group(0) @binding(2) var<storage, read>      a_data:  array<u32>;
@group(0) @binding(3) var<storage, read>      b_data:  array<u32>;
@group(0) @binding(4) var<storage, read_write> result: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b_idx = gid.x;   // batch item
    let c     = gid.y;   // channel

    if (b_idx >= params.batch_size || c >= params.n_channels) { return; }

    let i = b_idx * params.n_channels + c;
    let m = moduli[c];

    result[i] = (a_data[i] + b_data[i]) % m;
}
```

### Shader: `shaders/rns_mul.wgsl`
```wgsl
// Same structure as rns_add.wgsl.
// Multiplication needs u64 intermediate to avoid overflow.
// WGSL has no u64 — use two u32s (hi/lo) with manual carry:

fn mul_mod(a: u32, b: u32, m: u32) -> u32 {
    // Safe only when m < 2^16 (product < 2^32).
    // For moduli up to 131 (our 32-prime set), all fit in u16 — safe.
    return (a * b) % m;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b_idx = gid.x;
    let c     = gid.y;
    if (b_idx >= params.batch_size || c >= params.n_channels) { return; }
    let i = b_idx * params.n_channels + c;
    result[i] = mul_mod(a_data[i], b_data[i], moduli[c]);
}
```

**WGSL overflow note:** The first 32 primes go up to 131. Since 131 < 2¹⁶,
products of two residues fit in u32. If channels are extended beyond prime 131
(e.g. 137, 139, 149...), switch to the hi/lo u32 trick or emit u64 via two
storage atomics. Document this limit in a const: `MAX_SAFE_MODULUS: u64 = 65535`.

---

## Module 1 — `primes.rs`

**Purpose:** Everything about primes, factorization, and channel selection.

**Functions to implement:**

| Function | Signature | Notes |
|---|---|---|
| `primes_up_to` | `(n: usize) -> Vec<u64>` | Sieve of Eratosthenes |
| `first_n_primes` | `(n: usize) -> Vec<u64>` | Use upper bound: n*(ln n + ln ln n)*1.3 |
| `factorize` | `(n: u64) -> Vec<(u64, u32)>` | Returns (prime, exponent) pairs |
| `radical` | `(n: u64) -> u64` | Product of distinct prime factors. radical(12)=6 |
| `distinct_primes` | `(n: u64) -> Vec<u64>` | Just the primes, no exponents |
| `gcd` | `(a: u64, b: u64) -> u64` | Euclidean |
| `lcm` | `(a: u64, b: u64) -> u64` | a/gcd(a,b)*b |
| `extended_gcd` | `(a: i64, b: i64) -> (i64, i64, i64)` | Returns (gcd, x, y) s.t. ax+by=gcd |
| `mod_inverse` | `(a: u64, m: u64) -> Option<u64>` | None if gcd(a,m)≠1 |
| `termination_period` | `(n: u64, base: u64) -> Option<u64>` | 0=terminates, k=period length |
| `natural_base` | `(denominators: &[u64]) -> u64` | LCM of radicals — minimal exact base |

**Key concept to encode in docs:**
> The "natural base" of a computation is the LCM of the radicals of all denominators
> involved. This is the minimal base in which every fraction in the computation
> terminates exactly. `natural_base(&[6, 10, 15]) = 30`.

---

## Module 2 — `rns.rs`

**Purpose:** The RNS integer type and the two core algorithms.

### Type: `Channels`
```rust
pub struct Channels(pub Arc<Vec<u64>>);
```
- `Channels::new(moduli: Vec<u64>)` — validates pairwise coprime in debug
- `Channels::standard(n: usize)` — first n primes
- `fn capacity(&self) -> BigUint` — product of all moduli = dynamic range
- `fn signed_capacity(&self) -> BigInt` — ⌊capacity/2⌋ for signed range

### Type: `RnsInt`
```rust
pub struct RnsInt {
    pub residues: Vec<u64>,   // residues[i] = |value| mod channels[i]
    pub channels: Channels,
    pub negative: bool,
}
```

**Methods:**
- `from_bigint(n: &BigInt, channels: Channels) -> Self`
- `from_i64(n: i64, channels: Channels) -> Self`
- `to_bigint(&self) -> BigInt` — calls Garner CRT
- `add(&self, other: &Self) -> Self`
- `sub(&self, other: &Self) -> Self`
- `mul(&self, other: &Self) -> Self` — GPU path: `(a[i] as u128 * b[i] as u128) % m[i]`
- `is_zero(&self) -> bool`

**CPU vs GPU note (put in module doc):**
Single `RnsInt` operations use `CpuBackend::rns_add_single` which applies rayon
when k ≥ 16 channels. For batch operations (a `Vec<RnsInt>`) always go through
`executor().add(batch_a, batch_b)` which auto-selects CPU rayon or GPU based on
batch size. Never call GPU code directly from this module — always go through the
Executor.

### Algorithm: `garner_crt(residues: &[u64], moduli: &[u64]) -> BigUint`

Garner's algorithm — preferred over naive CRT because it never computes the
large basis elements M/m_i directly. Instead it builds a mixed-radix
representation where all intermediate values are small (each fits in its modulus).

```
Step 1 — Mixed-radix coefficients (forward substitution):
  c = residues.clone()
  for i in 0..k:
    for j in 0..i:
      c[i] = (c[i] - c[j]) * modinv(m[j], m[i])  mod m[i]

Step 2 — Horner reconstruction (one BigUint accumulation):
  result = c[k-1]
  for i in (0..k-1).rev():
    result = result * m[i] + c[i]
```

This is the algorithm that should appear directly as `garner_crt`. No fancy
abstractions needed — it's short and its correctness is easy to test.

**Also implement these free functions (GPU reference implementations):**
```rust
pub fn gpu_add_channel(a: u64, b: u64, m: u64) -> u64 { (a + b) % m }
pub fn gpu_mul_channel(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}
```
These document exactly what each GPU thread does. They're also used in tests.

---

## Module 3 — `rational.rs`

**Purpose:** Exact rational arithmetic — the main user-facing type.

### Type: `RnsRational`
```rust
pub struct RnsRational {
    pub numer: RnsInt,
    pub denom: RnsInt,      // always positive after normalization
    pub channels: Channels,
}
```

**Constructor:** `new(p: BigInt, q: BigInt, channels: Channels) -> Self`
- Normalize sign: denom always positive
- Reduce by GCD before storing
- Store reduced p, q as RnsInt

**Convenience:** `from_fraction(p: i64, q: i64, channels: Channels) -> Self`

**Arithmetic:** Implement by reconstructing to BigInt, computing, re-normalizing:
- `add(&self, other: &Self) -> Self` — p1*q2 + p2*q1 / q1*q2
- `sub`, `mul`, `div`

**Key methods (the "base awareness"):**
```rust
// Which primes appear in the denominator?
pub fn denom_prime_signature(&self) -> Vec<u64>

// Is this fraction exact in the given base?
// True iff every prime in denom_prime_signature divides base.
pub fn exact_in_base(&self, base: u64) -> bool

// Minimal base for exact representation = radical(denom)
pub fn natural_base(&self) -> u64

// How many digits in base b? 0 = terminates, k = repeating period
pub fn termination_period_in_base(&self, base: u64) -> u64

// f64 approximation + the exact error as an RnsRational
pub fn to_f64_with_error(&self) -> (f64, RnsRational)

// Human-readable: "3/7" or "5" (if denom=1)
pub fn display(&self) -> String
```

**Test cases that must pass:**
```
1/6 + 1/10 + 1/15  == 1/3       (not 0.3333...)
1/3 * 3            == 1          (not 0.9999...)
1/7 * 7            == 1
0.1 (= 1/10) + 0.2 (= 1/5) == 3/10   (not 0.30000000000000004)
(√2 represented as 1414.../1000 ) -- rational approx only, exact √2 is symbolic
```

---

## Module 4 — `algebraic.rs` (Level 2: ℚ̄)

**Purpose:** Exact representation of algebraic numbers — roots of polynomials with
rational coefficients. Examples: √2, ∛5, the golden ratio φ, roots of x⁵ - x - 1.
These are NOT representable as rationals, but they ARE exactly representable as
(minimal polynomial, isolating interval) pairs.

**Core insight:** You never store the decimal expansion of √2. You store the fact
that it is *the unique root of x² - 2 in the interval (1, 2)*. All arithmetic
operates on this representation symbolically; digits are only produced when Level 3
demands them.

### Type: `Polynomial`
```rust
pub struct Polynomial {
    pub coeffs: Vec<RnsRational>,   // coeffs[i] = coefficient of x^i
}
```

**Methods:**
- `degree(&self) -> usize`
- `eval(&self, x: &RnsRational) -> RnsRational`
- `derivative(&self) -> Polynomial`
- `sturm_sequence(&self) -> Vec<Polynomial>` — for root counting and isolation
- `sign_changes(seq: &[Polynomial], x: &RnsRational) -> usize`
- `resultant(p: &Polynomial, q: &Polynomial) -> Polynomial` — eliminates a variable;
  core of algebraic arithmetic (see below)

### Type: `AlgebraicNumber`
```rust
pub struct AlgebraicNumber {
    pub min_poly: Polynomial,              // minimal polynomial over ℚ — irreducible
    pub interval: (RnsRational, RnsRational), // isolating interval (lo, hi)
                                           // exactly one root of min_poly lies in (lo, hi)
}
```

**Constructors:**
- `AlgebraicNumber::sqrt(n: u64) -> Self` — min poly is x² - n, interval bisects to isolate root
- `AlgebraicNumber::cbrt(n: u64) -> Self` — min poly is x³ - n
- `AlgebraicNumber::from_poly_root(p: Polynomial, root_index: usize) -> Self`
  — enumerate all real roots of p via Sturm, return the `root_index`-th one
- `AlgebraicNumber::from_rational(r: RnsRational) -> Self` — min poly is x - r (degree 1)

**Arithmetic (the hard part — use resultants):**

To add two algebraic numbers α (root of p) and β (root of q):
```
α + β  is a root of:  Res_y( p(y),  q(x - y) )   [resultant eliminating y]
α × β  is a root of:  Res_y( p(y),  y^deg(q) * q(x/y) )
```
The resultant gives a polynomial whose roots include α+β (among others). Then:
1. Factor the resultant (or use Sturm) to find the irreducible factor containing α+β
2. Refine the isolating interval by bisection until α+β is isolated

This is expensive — O(d²) polynomial operations where d = degree. But it is **exact**.

**Methods:**
- `add(&self, other: &Self) -> Self`
- `mul(&self, other: &Self) -> Self`
- `neg(&self) -> Self`
- `recip(&self) -> Self`
- `to_rational(&self) -> Option<RnsRational>` — Some if min_poly has degree 1
- `refine_interval(&mut self, target_width: &RnsRational)` — bisect until interval < target
- `to_computable(&self) -> ComputableReal` — drop down to Level 3 for digit production
- `sign(&self) -> i32` — determine sign exactly using interval + Sturm

**Sturm sequence (implement this carefully — it's used everywhere):**
```
sturm_sequence(p):
  s[0] = p
  s[1] = p'   (derivative)
  s[i+1] = -remainder(s[i-1], s[i])   until remainder = 0

root_count(seq, a, b):
  sign_changes(seq, a) - sign_changes(seq, b)
  where sign_changes counts sign changes ignoring zeros
```

**Test cases:**
```
sqrt(2).min_poly                     == x² - 2
sqrt(2) + sqrt(2)                    == 2*sqrt(2)  (min poly: x² - 8)
sqrt(2) * sqrt(2)                    == 2          (min poly: x - 2, i.e. rational)
sqrt(2) * sqrt(3)                    == sqrt(6)    (min poly: x² - 6)
sqrt(2).to_rational()                == None
AlgebraicNumber::from_rational(r).to_rational() == Some(r)
```

---

## Module 5 — `computable.rs` (Level 3: ℝ_c)

**Purpose:** Exact computation of transcendental and other computable real numbers.
A `ComputableReal` is a *function* from precision demand to rational approximation.
You don't store digits — you store the *algorithm* that can produce any number of
digits on demand.

**Core insight:** π is not 3.14159... — it is an oracle. When you ask for 50 digits,
the oracle runs and returns a rational approximation accurate to 10⁻⁵⁰. The oracle
itself is exact; only the *output format* introduces approximation, and only when
you ask for it.

### Trait: `Computable`
```rust
pub trait Computable: Send + Sync {
    /// Return a rational r such that |self - r| < 10^(-precision).
    fn evaluate(&self, precision: u64) -> RnsRational;
}
```

### Type: `ComputableReal`
```rust
pub struct ComputableReal {
    inner: Arc<dyn Computable>,
    // Cached evaluations at various precisions — avoid recomputing
    cache: Mutex<BTreeMap<u64, RnsRational>>,
}

impl ComputableReal {
    pub fn evaluate(&self, precision: u64) -> RnsRational
    pub fn evaluate_f64(&self) -> f64   // convenience: evaluate at f64 precision (~15)
}
```

**Constructors — implement each as a struct implementing `Computable`:**

| Constructor | Algorithm | Notes |
|---|---|---|
| `ComputableReal::from_rational(r)` | Returns r directly | Trivial |
| `ComputableReal::from_algebraic(a)` | Bisect isolating interval | Uses `refine_interval` |
| `ComputableReal::pi()` | Chudnovsky series | Fastest known: ~14 digits per term |
| `ComputableReal::e()` | Sum 1/k! using RNS exact rationals | Converges rapidly |
| `ComputableReal::ln(r)` | AGM-based algorithm or Taylor | For r near 1 use series |
| `ComputableReal::sqrt(r)` | Newton's method on RnsRational | Converges quadratically |
| `ComputableReal::exp(r)` | Taylor series with rigorous error | |

**Arithmetic — operations produce new lazy `ComputableReal` values:**
```rust
impl ComputableReal {
    pub fn add(&self, other: &Self) -> Self
    // Inner::evaluate(prec+1) for both, add, result is accurate to 10^(-prec)

    pub fn mul(&self, other: &Self) -> Self
    // Need extra precision: if |self|, |other| < B, evaluate both at prec + log10(B) + 1

    pub fn neg(&self) -> Self
    pub fn recip(&self) -> Self   // Newton's method or continued fraction
}
```

**Chudnovsky algorithm for π (implement this as `PiComputable`):**
```
π = 1 / (12 * Σ_{k=0}^{∞}  (-1)^k (6k)! (13591409 + 545140134k)
                              ────────────────────────────────────── )
                                    (3k)! (k!)³ 640320^(3k+3/2)
```
Each term adds ~14.18 decimal digits. For n-digit π, sum ⌈n/14.18⌉ terms.
Use RnsRational for partial sums to avoid floating-point contamination of the series.

**Continued fraction representation (alternative for many constants):**
Store a `ComputableReal` as its continued fraction coefficient sequence:
```rust
pub struct ContinuedFraction {
    coeffs: Arc<dyn Fn(u64) -> BigInt>,   // coeffs(n) = nth CF coefficient
}
```
- π = [3; 7, 15, 1, 292, 1, 1, 1, 2, ...]
- e = [2; 1, 2, 1, 1, 4, 1, 1, 6, 1, 1, 8, ...] — has closed form pattern
- √2 = [1; 2, 2, 2, 2, ...] — purely periodic, stored as finite repeating block

The CF representation connects cleanly to Level 2: periodic CFs are exactly
the quadratic irrationals (Lagrange's theorem), so √n always has a finite CF
representation that's better stored at Level 2 than Level 3.

**Test cases:**
```
pi().evaluate(10)  within 1e-10 of true π
e().evaluate(15)   within 1e-15 of true e
sqrt(r2).evaluate(20)  within 1e-20 of true √2
from_rational(r).evaluate(50) == r  (exact, regardless of precision demand)
// Precision scaling:
pi_10  = pi().evaluate(10)
pi_50  = pi().evaluate(50)
|pi_10 - pi_50| < 1e-10   (low-precision result is within its stated bound)
```

---

## Module 6 — `tower.rs` (Level Dispatch)

**Purpose:** The number tower router. Every value in the system has a `TowerLevel`.
Operations try to *stay at the lowest (cheapest) level* they can. When an operation
would produce a result at a lower level than its inputs (e.g. √2 × √2 = 2), it
*drops down*. When it can't simplify, it *stays put* or *moves up* to the symbolic level.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TowerLevel {
    Integer     = 0,    // ℤ   — RnsInt
    Rational    = 1,    // ℚ   — RnsRational
    Algebraic   = 2,    // ℚ̄   — AlgebraicNumber
    Computable  = 3,    // ℝ_c — ComputableReal
    Symbolic    = 4,    // 𝒮   — SymbolicExpr DAG
}
```

### Type: `TowerValue`
```rust
pub enum TowerValue {
    Integer(RnsInt),
    Rational(RnsRational),
    Algebraic(AlgebraicNumber),
    Computable(ComputableReal),
    Symbolic(SymbolicExpr),
}

impl TowerValue {
    pub fn level(&self) -> TowerLevel
    
    /// Try to reduce to the lowest valid level.
    /// e.g. AlgebraicNumber with degree-1 min poly → Rational
    ///      Rational with denom=1 → Integer
    pub fn reduce(&self) -> TowerValue
    
    /// Elevate to at least the given level (for mixed-level operations).
    pub fn elevate_to(&self, level: TowerLevel) -> TowerValue
    
    /// Produce a decimal approximation. Only valid for levels 0-3.
    /// Level 4 (Symbolic) must be reduced or evaluated first.
    pub fn to_f64(&self) -> Option<f64>
    
    /// Produce exact digits to `precision` decimal places.
    /// Levels 0-1: exact, precision ignored.
    /// Level 2: refine interval until width < 10^(-precision), then read midpoint.
    /// Level 3: delegate to ComputableReal::evaluate(precision).
    /// Level 4: attempt symbolic simplification first; if still symbolic, error.
    pub fn digits(&self, precision: u64) -> RnsRational
}
```

### Tower Arithmetic
```rust
impl TowerValue {
    pub fn add(&self, other: &TowerValue) -> TowerValue {
        // 1. Elevate both to the same level (max of the two)
        // 2. Attempt identity-graph simplification (Level 4 check first)
        // 3. Compute at that level
        // 4. Call .reduce() on result — drop to lowest valid level
    }
    pub fn mul(&self, other: &TowerValue) -> TowerValue  // same pattern
    pub fn sin(&self) -> TowerValue   // check identity graph; else elevate to Symbolic or Computable
    pub fn sqrt(&self) -> TowerValue  // check if perfect square (→ Integer); else AlgebraicNumber
}
```

**The reduction rules (encode these as a table in tower.rs):**

```
Algebraic(deg 1 poly)       → Rational
Rational(denom = 1)         → Integer
Algebraic + Algebraic       → may reduce to Rational (if roots cancel)
sqrt(n²)                    → Integer(n)
sqrt(a²·b)                  → Algebraic(√b) scaled by a  [simplify radicand]
Computable(from_rational(r)) → Rational(r)
Symbolic(sin(0))            → Integer(0)
Symbolic(Integer(n))        → Integer(n)
```

**Precision flow (document this clearly):**

When a user asks for digits, the request flows *down* the tower:
```
User: "give me π+1 to 50 digits"
  → TowerValue::Symbolic(Add(Pi, Integer(1)))
  → try simplify → stays Symbolic (no identity for π+1)
  → elevate Pi to Computable, elevate 1 to Computable
  → Computable.add(pi, one).evaluate(50)   [Level 3 handles it]
  → RnsRational accurate to 10⁻⁵⁰
```

---

## Module 7 — `symbolic.rs` (Level 4: 𝒮)

**Purpose:** Exact representation of algebraic and transcendental numbers as
expression trees. The key insight: most computations involving π, e, √2 etc.
never need decimal digits — they need algebraic relationships.

### Type: `SymbolicExpr`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolicExpr {
    Integer(BigInt),
    Rational(BigInt, BigInt),                          // p/q
    Sqrt { radicand: BigInt },                         // √n (simplified)
    ScaledSqrt { coeff: (BigInt, BigInt), rad: BigInt }, // (p/q)·√n
    Pi,
    E,
    Add(Vec<SymbolicExpr>),                            // flattened n-ary
    Mul(Vec<SymbolicExpr>),                            // flattened n-ary
    Pow { base: Box<SymbolicExpr>, exp: Box<SymbolicExpr> },
    Sin(Box<SymbolicExpr>),
    Cos(Box<SymbolicExpr>),
    Exp(Box<SymbolicExpr>),
    Ln(Box<SymbolicExpr>),
}
```

### Type: `IdentityGraph`
```rust
pub struct IdentityGraph {
    rules: Vec<(SymbolicExpr, SymbolicExpr)>,  // (pattern, reduction)
}
```

**Rules to implement (hardcoded in `IdentityGraph::standard()`):**

Algebraic reductions (always exact):
- `√n · √n  →  n`
- `√(a²·b)  →  a·√b`     (simplify square roots)
- `x · 0    →  0`
- `x · 1    →  x`
- `x + 0    →  x`
- `0 + x    →  x`

Trig at special values:
- `sin(0)   →  0`
- `sin(π)   →  0`
- `sin(π/6) →  1/2`
- `sin(π/4) →  √2/2`
- `sin(π/3) →  √3/2`
- `sin(π/2) →  1`
- `cos(π)   →  -1`
- `cos(π/2) →  0`
- `cos(0)   →  1`

Exponential/log:
- `exp(0)   →  1`
- `ln(1)    →  0`
- `exp(iπ)+1 →  0`   (Euler)

**Method:** `pub fn simplify(&self, expr: SymbolicExpr) -> SymbolicExpr`
- Walk the expression tree recursively
- At each node, try every rule
- Repeat until no rule fires (fixed-point)

**Number tower level:**
```rust
pub enum TowerLevel { Integer, Rational, Algebraic, Symbolic, Transcendental }
pub fn tower_level(expr: &SymbolicExpr) -> TowerLevel
```

**Critical behavior:** `simplify` should reduce `sin(Pi)` to `Integer(0)` in O(1),
not compute `3.14159...` and then `sin(3.14159...)`. The identity is a table lookup.
This is the core value proposition: operations that classical float gets wrong
(sin(π) ≈ 1.2e-16) become exact here.

---

## Module 5 — `dispatch.rs`

**Purpose:** Base-aware ALU dispatcher. Before any operation, inspect the
denominator signatures of both operands and determine: which channels are
needed, what the result's natural base will be, and which channels can sleep.

```rust
pub struct Dispatcher {
    channels: Channels,
}

pub struct DispatchPlan {
    pub active_channels: Vec<usize>,   // indices into channels
    pub natural_base: u64,             // LCM of operand natural bases
    pub channel_mask: u64,             // bitmask for GPU dispatch
}
```

**Methods:**
```rust
impl Dispatcher {
    pub fn new(channels: Channels) -> Self

    // Inspect operands → return which channels are needed + natural base
    pub fn plan_add(&self, a: &RnsRational, b: &RnsRational) -> DispatchPlan
    pub fn plan_mul(&self, a: &RnsRational, b: &RnsRational) -> DispatchPlan

    // Execute with only the needed channels active
    pub fn execute_add(&self, a: &RnsRational, b: &RnsRational) -> RnsRational
    pub fn execute_mul(&self, a: &RnsRational, b: &RnsRational) -> RnsRational

    // Report: what fraction of channels were idle for this operation?
    pub fn channel_efficiency(&self, plan: &DispatchPlan) -> f64
}
```

**The key insight to document:** When adding 1/6 + 1/4, the natural base is 12,
which requires only primes {2, 3}. Channels for primes 5, 7, 11, 13... are idle.
On the GPU, idle channels consume no power. The dispatcher is the hardware analog
of lazy evaluation — work only happens where the number's arithmetic structure
demands it.

---

## Module 6 — `gpu.rs` (feature = "gpu")

**Purpose:** GPU batch arithmetic using wgpu compute shaders.

### Workflow
1. Pack a batch of RnsRational values into flat GPU buffers
2. Dispatch a compute shader — one thread per (batch_item × channel)  
3. Read back results and unpack

### Buffer Layout
```
// For B rational numbers with K channels each:
// Layout: [B × K] flat, row-major (batch index outer, channel index inner)

a_numer:  [u32; B * K]    // numerator residues
a_denom:  [u32; B * K]    // denominator residues
b_numer:  [u32; B * K]
b_denom:  [u32; B * K]
moduli:   [u32; K]         // channel moduli (uniform or storage)
result_n: [u32; B * K]
result_d: [u32; B * K]
```

### Shader: `shaders/rns_add.wgsl`
```wgsl
// Each thread: one (batch, channel) pair
// gid.x = batch index, gid.y = channel index

@compute @workgroup_size(16, 16)
fn rns_rational_add(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b = gid.x;   // batch item
    let c = gid.y;   // channel

    if (b >= params.batch_size || c >= params.n_channels) { return; }

    let i    = b * params.n_channels + c;
    let m    = moduli[c];

    // p1/q1 + p2/q2 = (p1*q2 + p2*q1) / (q1*q2)  in each channel
    let p1   = a_numer[i];
    let q1   = a_denom[i];
    let p2   = b_numer[i];
    let q2   = b_denom[i];

    // Use intermediate u64 via two u32 packing to avoid overflow
    let cross1 = u32((u64(p1) * u64(q2)) % u64(m));
    let cross2 = u32((u64(p2) * u64(q1)) % u64(m));

    result_numer[i] = (cross1 + cross2) % m;
    result_denom[i] = u32((u64(q1) * u64(q2)) % u64(m));
}
```

Note: WGSL doesn't have native u64. Use either:
- Two u32s with manual carry (reference the shader trick for u64 emulation)
- Or work within the safe range by choosing moduli < 2^16 so products fit in u32

### Type: `GpuEngine`
```rust
pub struct GpuEngine {
    device:  wgpu::Device,
    queue:   wgpu::Queue,
    add_pipeline: wgpu::ComputePipeline,
    mul_pipeline: wgpu::ComputePipeline,
    channels: Channels,
}

impl GpuEngine {
    pub async fn new(channels: Channels) -> Result<Self, GpuError>
    pub fn batch_add(&self, a: &[RnsRational], b: &[RnsRational]) -> Vec<RnsRational>
    pub fn batch_mul(&self, a: &[RnsRational], b: &[RnsRational]) -> Vec<RnsRational>
}
```

For initialization, use `pollster::block_on` in tests/examples so the async
doesn't bleed into the library API.

---

## Examples

### `examples/engineering.rs`
Demonstrate real structural engineering fractions:
```
// Typical steel section dimensions and ratios
3/8 in + 1/4 in + 5/16 in    →  exact result in fractional inches
Safety factor: 1/1.5          →  2/3 exactly
Load ratio: 47_kips / 70_kips →  47/70 exactly (no float drift in combination checks)
AISC web area: tw * d         →  product of exact fractions
```

### `examples/benchmark_backends.rs`
Print a timing table across batch sizes and both backends:
```
batch_size │ cpu_rayon_µs │ gpu_µs │ winner
─────────────────────────────────────────────
         1 │          0.3 │  105.2 │  CPU
        16 │          1.1 │  106.4 │  CPU
       128 │          6.8 │  108.1 │  CPU (break-even ~here)
      1024 │         47.2 │  112.3 │  GPU
     16384 │        741.0 │  118.7 │  GPU
    262144 │      11840.0 │  195.4 │  GPU
```
The GPU column barely grows because the computation is embarrassingly parallel
and the upload/compute overlap hides latency at large batch sizes.
Use `std::time::Instant` for timing; run 10 warm-up iterations before measuring.
Side-by-side table showing:
- The expression
- Exact result (as fraction)
- f64 result
- Absolute error
- ULP error
Highlight: 0.1+0.2, sin(π), √2·√2, 1/3·3, etc.

---

## Test Requirements

Every module needs unit tests. The crate-level integration tests in `tests/` should cover:

```rust
// ── Backend correctness: CPU and GPU must match exactly ─────────────────────
// Run this for every batch size to catch any GPU shader bugs
let channels = Channels::standard(32);
let a_batch  = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, channels.clone()); 256]);
let b_batch  = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, channels.clone()); 256]);

let cpu_result = CpuBackend::new().batch_rns_add(&a_batch, &b_batch);
let gpu_result = GpuBackend::try_init().unwrap().batch_rns_add(&a_batch, &b_batch);
assert_eq!(cpu_result.data, gpu_result.data);   // bit-for-bit identical

// ── Level 0 & 1: Integer and Rational ──────────────────────────────────────
1/6 + 1/10 + 1/15   == 1/3
1/3 * 3             == 1
0.1 + 0.2           == 3/10
7/8 - 3/8           == 1/2

// Base alignment
RnsRational(1,6).natural_base()      == 6
RnsRational(1,12).natural_base()     == 6    // radical(12)=6, not 12
RnsRational(1,6).exact_in_base(6)    == true
RnsRational(1,6).exact_in_base(10)   == false
RnsRational(1,6).exact_in_base(30)   == true

// Garner CRT
garner_crt(&[2,3,2], &[3,5,7])  == 23   // classic example
garner_crt(&[0,1,0], &[2,3,5])  == 10   // 1/6+1/10+1/15 numerator check

// ── Level 2: Algebraic ─────────────────────────────────────────────────────
sqrt(2).min_poly.degree()           == 2
sqrt(2).min_poly.eval(rational(1,1)).sign() == Sign::Minus  // 1²-2 < 0
sqrt(2).interval.0 < sqrt(2).interval.1     // interval is valid

sqrt(2) * sqrt(2)   → reduces to Integer(2)     // min poly becomes degree 1 → rational → integer
sqrt(2) * sqrt(3)   → AlgebraicNumber with min_poly x²-6
sqrt(2) + sqrt(2)   → AlgebraicNumber with min_poly x²-8

// Sturm sequence sanity check
// x²-2 has exactly 2 real roots — one in (-2,-1), one in (1,2)
sturm_root_count(x²-2, -2.0, -1.0) == 1
sturm_root_count(x²-2,  1.0,  2.0) == 1
sturm_root_count(x²-2, -2.0,  2.0) == 2

// Level 2 precision: refine √2 interval to width < 1e-20
sqrt2.refine_interval(rational(1, 10^20))
(sqrt2.interval.1 - sqrt2.interval.0) < rational(1, 10^20)

// ── Level 3: Computable ────────────────────────────────────────────────────
// π to 10 decimal places
let pi10 = pi().evaluate(10);
assert!((pi10.to_f64() - std::f64::consts::PI).abs() < 1e-10);

// e to 15 places
let e15 = e().evaluate(15);
assert!((e15.to_f64() - std::f64::consts::E).abs() < 1e-15);

// Precision contract: evaluate(n) must be accurate to 10^(-n)
let pi_lo = pi().evaluate(5).to_f64();
let pi_hi = pi().evaluate(50).to_f64();
assert!((pi_lo - pi_hi).abs() < 1e-5);  // 5-digit result within its stated bound

// Exact rational passes through unchanged
let r = RnsRational::from_fraction(1, 3, channels());
let cr = ComputableReal::from_rational(r.clone());
assert_eq!(cr.evaluate(100), r);  // any precision demand → exact same rational

// ── Level 4: Symbolic identities ──────────────────────────────────────────
simplify(sin(Pi))        == Integer(0)
simplify(cos(Pi))        == Integer(-1)
simplify(sin(Pi/6))      == Rational(1, 2)
simplify(exp(Integer(0)))== Integer(1)
simplify(sqrt(2)*sqrt(2))== Integer(2)
simplify(x * zero)       == Integer(0)

// ── Tower: level routing and reduction ───────────────────────────────────
TowerValue::Rational(r).level()   == TowerLevel::Rational
TowerValue::Rational(r_int).reduce().level() == TowerLevel::Integer  // if denom=1
TowerValue::Algebraic(sqrt2).mul(TowerValue::Algebraic(sqrt2))
    .reduce().level()             == TowerLevel::Integer  // √2*√2 drops to 2

// Dispatcher efficiency
// Adding 1/6 + 1/4 (natural base 12 = 2×3) should only activate 2 of 16 channels
plan = dispatcher.plan_add(sixth, quarter)
plan.active_channels.len() == 2
dispatcher.channel_efficiency(&plan) == 2.0/16.0
```

---

## Implementation Order

Build bottom-up. Parallelism infrastructure first, then math layers.

**Phase 1 — Parallelism foundation (do this before any math)**
1. `primes.rs` — pure functions, no dependencies
2. `batch.rs` — `RnsBatch` flat buffer; test get/set/pack/unpack roundtrip
3. `cpu.rs` — `CpuBackend` with rayon; test `batch_rns_add` and `batch_rns_mul`
4. `gpu.rs` — `GpuBackend` with wgpu shaders; test same ops match CPU results exactly
5. `backend.rs` — `Executor` with auto-selection; test that CPU and GPU produce identical output

**Phase 2 — Number tower (each level depends on the one below)**

6. `rns.rs` — `RnsInt` using Executor for operations — **Level 0**
7. `rational.rs` — `RnsRational`, base-alignment queries — **Level 1**
8. `algebraic.rs` — `Polynomial`, `AlgebraicNumber`, Sturm, resultants — **Level 2** (hardest)
9. `computable.rs` — `ComputableReal`, Chudnovsky π, lazy eval — **Level 3**
10. `symbolic.rs` — `SymbolicExpr` DAG, `IdentityGraph` — **Level 4**
11. `tower.rs` — `TowerValue` unifying all levels, reduction rules — **ties it all together**
12. `dispatch.rs` — RNS channel routing, efficiency metrics

**Phase 3 — Examples and benchmarks**

13. `examples/engineering.rs` — real structural engineering fractions
14. `examples/float_comparison.rs` — exact vs f64 error table
15. `examples/benchmark_backends.rs` — CPU vs GPU timing at batch sizes 1, 16, 128, 1024, 65536

**Rules:**
- Phase 1 must be fully green before touching Phase 2.
- Level 2 (algebraic) is the hardest — resultant computation and Sturm sequences
  over `RnsRational` polynomials. Budget extra time.
- GPU and CPU results must match exactly (not just approximately) for all RNS ops.
  The benchmark example will catch any divergence.

---

## Notes for Cursor

**Crate naming convention:**
- `Cargo.toml`: `name = "adele-ring"` (hyphen — standard for crate names)
- Rust `use` statements: `use adele_ring::...` (underscore — Rust identifier rule)
- The two are the same crate; Cargo handles the mapping automatically.


**Parallelism:**
- The rayon threshold for single-item channel parallelism is `const RAYON_CHANNEL_THRESHOLD: usize = 16`.
  Below this, sequential is faster due to task overhead. Make it a named const, not a magic number.
- For batch operations, always go through `executor()`. Never call `CpuBackend` or `GpuBackend` directly
  from math modules — only `backend.rs` and the example benchmarks touch backends directly.
- rayon's `par_chunks_mut(k)` is the right primitive for batch items: each chunk is one item's
  channel row, and chunks are independent.

**GPU specifics:**
- All 32 primes are ≤ 131 < 2¹⁶, so residues fit in u16 and products fit in u32. WGSL's u32 is safe.
  If channels are ever extended past 65521 (largest prime < 2¹⁶), add u64 emulation in shaders.
- wgpu requires `bytemuck::Pod` on buffer types. The `RnsBatch::as_u32_bytes()` cast uses bytemuck;
  ensure `u32` slice alignment is correct.
- GPU round-trip (upload → dispatch → download) has ~100µs fixed overhead on discrete GPUs.
  The 128-item threshold for `Executor::select` is empirical — expose it as a public field so
  users can tune it for their hardware.
- Use `wgpu::include_wgsl!` macro to embed shaders at compile time so the binary is self-contained.

**BigInt:**
- All BigInt arithmetic uses `num-bigint`. Import `BigInt`, `BigUint` from it.
- Use `num_integer::Integer` trait for `.gcd()` on BigInt/BigUint where needed.
- Garner CRT reconstruction is always CPU-side — it's sequential by nature and k≤32 so it's fast.

**Channels:**
- `Channels` is `Arc<Vec<u64>>` wrapped in a newtype — cheap to clone, shared allocation.
- Moduli for practical use: 32 primes covers denominators whose prime factors are all ≤ 131.
  The product is ~5×10⁵⁰ — more than enough for any engineering or financial calculation.

**IdentityGraph:**
- A `Vec<Box<dyn Fn(&SymbolicExpr) -> Option<SymbolicExpr>>>` of rewrite rules applied
  in sequence until fixed-point is fine for v0.1. No need for a real graph structure yet.
