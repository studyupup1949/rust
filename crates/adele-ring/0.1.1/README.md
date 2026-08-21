# adele-ring

**Exact arithmetic carried at both kinds of place: the finite primes and the real line.**

`adele-ring` models numbers the way the adele ring does —
`𝔸_ℚ = ℝ × ∏′_p ℚ_p` — pairing the **finite places** (an adaptive Residue
Number System over a prime `Basis`) with the **infinite place** (a rigorous real
interval, `Ball`). The `Adelic` carrier holds both at once:

- The **finite part** does all the arithmetic — carry-free, local, embarrassingly
  parallel across CPU threads (rayon) and GPU threads (wgpu), free of big-integer
  work in the hot loop.
- The **infinite part** answers everything the prime channels constitutionally
  cannot — sign, comparison, magnitude, decimal output — by refining on demand.

The `Basis` is **adaptive**: it provisions as many primes as a computation's
a-priori height bound demands (`bounds`), then CRT + rational-reconstructs once at
the boundary. Overflow is therefore a *detected event* (`RangeError`), never a
silent aliasing mod `M`. All primes live in `(2^15, 2^16)`, so every channel op —
including `(a*b)` — fits a `u32` and is GPU-safe.

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
use adele_ring::{executor, Basis, RnsBatch, RnsInt};

let basis = Basis::standard();
let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, basis.clone()); 1024]);
let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, basis.clone()); 1024]);
let sum = executor().add(&a, &b); // auto CPU/GPU; add / sub / mul
```

Balanced (symmetric) residues mean subtraction is a channel-parallel
`(a + m - b) % m` with no sign-magnitude branch; the *sign* of a result is read
from the `Ball`, not the residues.

## Why it matters

```rust
use adele_ring::{Basis, RnsRational};
let ch = Basis::standard();
let f = |p, q| RnsRational::from_fraction(p, q, ch.clone());

assert_eq!(f(1, 10).add(&f(1, 5)), f(3, 10)); // 0.1 + 0.2 == 3/10 exactly
assert_eq!(f(1, 6).add(&f(1, 10)).add(&f(1, 15)), f(1, 3));
```

`sin(π)` is `0` by table lookup (not `1.2e-16`); √2·√2 is the integer `2`; π is an
oracle that yields a rational accurate to `10⁻ⁿ` on demand.

## Layout

```
src/
  error.rs       RangeError, BasisError, ChannelMismatch, GpuError
  ball.rs        the infinite place — rigorous rational interval (sign/cmp/output)
  basis.rs       adaptive multimodular prime basis (GPU-eligible, (2^15, 2^16))
  reconstruct.rs rational reconstruction (overflow → detected event)
  bounds.rs      Hadamard / Mignotte a-priori height bounds
  adelic.rs      Adelic<F> carrier (finite RNS × Ball); AdelicInt, AdelicRat, RnsFrac
  primes.rs      prime utilities, factorization, natural-base selection
  batch.rs       RnsBatch shared flat u32 buffer
  cpu.rs         CpuBackend (rayon): add / sub / mul / balanced CRT
  gpu.rs         GpuBackend (wgpu)
  backend.rs     ArithmeticBackend trait + Executor (auto-select)
  rns.rs         Level 0 — balanced RnsInt over Basis, Garner CRT
  rational.rs    Level 1 — RnsRational + base awareness
  algebraic.rs   Level 2 — Polynomial, Sturm, resultants, AlgebraicNumber
  computable.rs  Level 3 — ComputableReal (Ball-native enclose; π, e, sqrt, exp, ln)
  symbolic.rs    Level 4 — SymbolicExpr + IdentityGraph
  tower.rs       TowerValue unifying all levels
  dispatch.rs    base analyzer + adaptive-basis provisioner
shaders/         rns_add.wgsl, rns_sub.wgsl, rns_mul.wgsl
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
