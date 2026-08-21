# adele-ring

**Exact multi-base arithmetic engine via the Residue Number System (RNS).**

Instead of a single binary representation, numbers live simultaneously across
multiple prime *channels*. By the Chinese Remainder Theorem any integer up to
`M = ∏(channels)` is uniquely identified by its residue tuple — and because the
channels are mutually independent (no carry propagation), arithmetic is
embarrassingly parallel across both CPU threads (rayon) and GPU threads (wgpu).

> The crate is named `adele-ring` (hyphen); the Rust import path is `adele_ring`
> (underscore). Cargo maps between them automatically.

## The number tower

Every value is kept at the cheapest *exact* level it can occupy. Operations drop
down when a result simplifies (√2·√2 = 2) and rise only when forced.

| Level | Set   | Type                          | Module          |
|-------|-------|-------------------------------|-----------------|
| 0     | ℤ     | `RnsInt`                      | `rns`           |
| 1     | ℚ     | `RnsRational`                 | `rational`      |
| 2     | ℚ̄     | `AlgebraicNumber`, `Polynomial` | `algebraic`   |
| 3     | ℝ_c   | `ComputableReal`              | `computable`    |
| 4     | 𝒮     | `SymbolicExpr`, `IdentityGraph` | `symbolic`    |

`TowerValue` (`tower` module) unifies all five levels with automatic reduction
and level-routing for mixed-level arithmetic.

## Backends

Both backends are first-class and share one flat buffer layout (`RnsBatch`), so
there is no reformatting when switching between them:

- **CPU** (`cpu::CpuBackend`) — rayon work-stealing over batch items / channels.
- **GPU** (`gpu::GpuBackend`) — wgpu compute shaders (`shaders/*.wgsl`), one
  thread per `(batch_item × channel)`.

The `Executor` probes for a GPU at startup and falls back to CPU automatically.
Small batches stay on the CPU (GPU upload overhead dominates); large batches go
to the GPU. CRT reconstruction (Garner's algorithm) is always CPU-side.

```rust
use adele_ring::{executor, Channels, RnsBatch, RnsInt};

let ch = Channels::standard(32);
let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, ch.clone()); 1024]);
let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, ch.clone()); 1024]);
let sum = executor().add(&a, &b); // auto CPU/GPU
```

## Why it matters

```rust
use adele_ring::{Channels, RnsRational};
let ch = Channels::standard(32);
let f = |p, q| RnsRational::from_fraction(p, q, ch.clone());

assert_eq!(f(1, 10).add(&f(1, 5)), f(3, 10)); // 0.1 + 0.2 == 3/10 exactly
assert_eq!(f(1, 6).add(&f(1, 10)).add(&f(1, 15)), f(1, 3));
```

`sin(π)` is `0` by table lookup (not `1.2e-16`); √2·√2 is the integer `2`; π is an
oracle that yields a rational accurate to `10⁻ⁿ` on demand.

## Layout

```
src/
  primes.rs      prime utilities, factorization, natural-base selection
  batch.rs       RnsBatch shared flat buffer
  cpu.rs         CpuBackend (rayon)
  gpu.rs         GpuBackend (wgpu)
  backend.rs     ArithmeticBackend trait + Executor (auto-select)
  rns.rs         Level 0 — RnsInt, Channels, Garner CRT
  rational.rs    Level 1 — RnsRational + base awareness
  algebraic.rs   Level 2 — Polynomial, Sturm, resultants, AlgebraicNumber
  computable.rs  Level 3 — ComputableReal (π via Machin, e, sqrt, exp, ln)
  symbolic.rs    Level 4 — SymbolicExpr + IdentityGraph
  tower.rs       TowerValue unifying all levels
  dispatch.rs    base-aware channel dispatcher
shaders/         rns_add.wgsl, rns_mul.wgsl
examples/        engineering, float_comparison, benchmark_backends
tests/           integration.rs
```

## Build & test

```sh
cargo test                                   # unit + integration tests
cargo run --example engineering
cargo run --example float_comparison
cargo run --release --example benchmark_backends
```

## License

MIT OR Apache-2.0
