# adele-ring — Refactor Plan
**Correcting the truncation mistake: name the infinite place, make the finite places adaptive**

> Supersedes the broken portions of `adele-ring_plan.md`. Where this document and the
> original disagree, this one wins. Sections of the original that are still valid are
> called out explicitly in §10 (Migration).

---

## 0. The one-sentence diagnosis

The original engine modeled only the **finite places** of ℚ — and only a *fixed, truncated*
set of them (32 primes, one p-adic digit each = ℤ/Mℤ with M ≈ 10⁵⁰). The adele ring is

```
𝔸_ℚ  =  ℝ  ×  ∏′_p ℚ_p
        ▲            ▲
   infinite place    finite places  (RNS is the depth-1 truncation of this)
```

Two structural facts were ignored:

1. **The real place ℝ is irremovable.** Sign, comparison, ordering, magnitude, overflow,
   and decimal output are *Archimedean* questions. The p-adic channels measure divisibility,
   not size — they are constitutionally blind to all of them. The original asked
   Archimedean questions of a non-Archimedean representation, so every sign/compare/Sturm
   call silently reconstructed through BigInt and the RNS bought nothing upstairs.

2. **A fixed prime set is a fixed dynamic range.** Any value exceeding M aliases mod M with
   no signal → confidently wrong "exact" answers. Resultants (Level 2) and series partial
   sums (Level 3) blow past 10⁵⁰ almost immediately.

**The fix is not "use BigInt." The fix is two design moves:**

- **Make the finite places first-class *as a pair with the infinite place*.** Every number
  carries an RNS image (finite) *and* a real interval / ball (infinite), updated together.
  Exactness comes from the finite part; sign/order/output come from the infinite part. The
  sign problem dissolves — it was never an RNS problem, it was a missing-component problem.

- **Make the basis adaptive (multimodular discipline).** Provision *as many primes as the
  computation's height demands*, computed from an a-priori bound, then CRT + rational-
  reconstruct once at the boundary. This is what every CAS does for exact determinants and
  resultants, and it is genuinely adelic: the number of finite places scales with the problem.

BigInt does not disappear — it is **confined to the reconstruction boundary**, where it
exists for an instant on numbers of size ~M. That residue is a law, not a wart (see §9).

---

## 1. The core new abstraction — the adelic carrier

Replace "an RNS integer" with "a value carried at both kinds of place simultaneously."

```rust
/// A value living in the (truncated) adele ring: finite places + the real place.
/// Invariant (debug-checked): the finite reconstruction, when it succeeds, lies
/// inside `infinite`. This is the practical shadow of the product formula.
pub struct Adelic<F> {
    finite:   F,        // ∏ ℚ_p component — RnsInt or RnsFrac (balanced residues)
    infinite: Ball,     // ℝ component — a rational interval [lo, hi]
}

pub type AdelicInt = Adelic<RnsInt>;
pub type AdelicRat = Adelic<RnsFrac>;
```

- **Finite part** does the carry-free, parallel arithmetic. Pure RNS, no BigInt in the hot loop.
- **Infinite part** (`Ball`) answers *every* Archimedean query in O(1)-ish without reconstruction:
  `sign()`, `cmp()`, `contains_zero()`, "is |x| < ε", and "give me a float."
- Arithmetic updates **both** components. They are kept consistent; in debug builds the
  reconstructed finite value is asserted ∈ `infinite`.

This single type is the whole reframe. It is the data-structure form of 𝔸_ℚ = ℝ × ∏′ ℚ_p.

---

## 2. `Ball` — the real place made into a type

```rust
/// The Archimedean component. A rigorous enclosure, never a point estimate.
/// All ordering/sign decisions in the entire tower go through Ball.
pub struct Ball {
    lo: Rational,   // exact rational bounds (BigInt-rational, but tiny — see note)
    hi: Rational,
}

impl Ball {
    pub fn point(r: &Rational) -> Self                 // lo == hi == r
    pub fn sign(&self) -> Option<Sign>                 // None if it straddles 0 → refine
    pub fn cmp(&self, other: &Ball) -> Option<Ordering>// None if overlapping → refine
    pub fn contains_zero(&self) -> bool
    pub fn width(&self) -> Rational
    pub fn midpoint(&self) -> Rational
    pub fn to_f64(&self) -> f64                         // nearest representable, with direction
    // arithmetic with outward rounding (interval arithmetic):
    pub fn add(&self, b: &Ball) -> Ball
    pub fn mul(&self, b: &Ball) -> Ball
    pub fn recip(&self) -> Option<Ball>                // None if it contains 0
}
```

Two rules that make `sign`/`cmp` total in practice:

- When `sign()`/`cmp()` returns `None` (the ball straddles the boundary), the **caller is
  obligated to refine** — bisect the algebraic interval, or pull more digits from the
  computable real — and retry. This is the only honest way to decide sign of an exact real,
  and it replaces every place the original silently called `to_bigint()`.
- The `lo`/`hi` rationals stay *small*: they are low-precision enclosures, not the exact
  value. Their BigInt content is bounded by the requested precision, not by M.

> Out-of-the-box note: this is the same `Ball` the computable-real layer (Level 3) returns.
> Sign-of-an-algebraic-number and 50-digits-of-π are the *same operation* at different
> precisions — both are "refine the real place until the question resolves." Unify them.

---

## 3. Adaptive basis — multimodular discipline

`Channels` (fixed) → `Basis` (grows on demand).

```rust
/// An ordered, extensible set of pairwise-coprime moduli.
/// GPU-eligible primes live in (2^15, 2^16): products of two residues fit u32,
/// so WGSL naive mulmod is safe, and ~16 bits of range accrue per channel.
pub struct Basis {
    primes: Arc<Vec<u32>>,   // each in (2^15, 2^16)
}

impl Basis {
    pub fn with_bits(bits: u64) -> Self;     // smallest basis whose ∏ exceeds 2^bits
    pub fn capacity_bits(&self) -> u64;      // floor(log2 ∏ primes)
    pub fn extend_to_bits(&self, bits: u64) -> Basis;  // add primes (cheap, Arc share prefix)
    pub fn modulus(&self, i: usize) -> u32;  // <- the method the original used but never defined
    pub fn len(&self) -> usize;
}
```

**The multimodular contract** for any exact computation whose output is an integer or rational:

```
1. Bound H = a-priori bound on the bit-height of the result (§3a).
2. basis = Basis::with_bits(H + 2)      // +1 for sign (balanced), +1 slack
3. Compute the whole thing mod each prime, independently — RNS-native, parallel.
4. CRT once → x mod M.
5. If the result is rational: rational-reconstruct (§4). Else: balanced lift.
```

The channel count now **scales with the problem**. A 166-bit (10⁵⁰) range is ~11 channels;
a 10⁵⁰⁰ resultant is ~104 channels — and the GPU *likes* more channels. The fixed-32 ceiling
is gone.

### 3a. Height bounds to implement (`bounds.rs`, new module)

| Computation                | Bound to use                          | Formula sketch |
|---|---|---|
| Sum / product of integers  | exact (track bit-lengths)             | trivial |
| Determinant (n×n)          | Hadamard                              | ∏ row-2-norms |
| Resultant Res(f,g)         | Hadamard/Mahler on Sylvester matrix   | ‖f‖₂^deg g · ‖g‖₂^deg f |
| Integer factor of f        | Mignotte                              | ‖g‖∞ ≤ 2^deg g · ‖f‖₂ |
| Rational reconstruction OK | needs M > 2·max(|p|,|q|)²             | drives basis size |

These bounds are cheap to compute from coefficient sizes and turn "might overflow" into
"provision exactly enough channels, then prove it fit."

---

## 4. Rational reconstruction — overflow becomes a *detected event*

This single technique converts the original's worst failure mode (silent aliasing) into a
clean, checkable result.

```rust
/// Recover the unique reduced p/q with |p|,|q| ≤ sqrt(M/2) such that p ≡ q·x (mod M),
/// via the half-GCD / extended-Euclid on (M, x). Returns None when no such fraction
/// exists within the bound — which *means the basis was too small*: extend and retry.
pub fn rational_reconstruct(x: &BigUint, m: &BigUint) -> Option<(BigInt, BigInt)>;
```

Usage everywhere a finite (RNS) value must become an exact rational:

```rust
match rational_reconstruct(&crt_image, &basis.modulus_product()) {
    Some((p, q)) => RnsFrac::from_reduced(p, q, basis),
    None         => return Err(RangeError::ReconstructionFailed { have_bits, need_more }),
}
```

- For *closed* operations (resultants, det) you provisioned enough primes in §3, so this
  always succeeds — and if it ever returns `None`, that is a *bug-or-bound* alarm, not a
  wrong answer.
- For *open-ended* user arithmetic (Level 0/1 with no a-priori bound), `None` triggers
  `basis.extend_to_bits(...)` and a recompute. Growth is lazy and bounded by the true height.

**This is the fix for original Tier-1 #1.** No computation can silently exceed range again.

---

## 5. Balanced residues — signed arithmetic without sign-magnitude

Delete `negative: bool`. Adopt the **symmetric/balanced** convention throughout.

- A value v is represented in (−M/2, M/2].
- Each residue rᵢ is stored in (−pᵢ/2, pᵢ/2] (or as u32 with a documented balanced lift).
- Negation is channel-wise `(pᵢ − rᵢ) mod pᵢ`; subtraction is `add(a, neg(b))` — both
  fully channel-parallel, no magnitude comparison, no BigInt.
- The batch buffer now carries the *complete* signed value; nothing is dropped on pack.
- Sign of the result is read from the **`Ball`**, not from the residues (residues can't
  see sign — that was the whole §0 point).

Backend trait gains the missing operation:

```rust
pub trait ArithmeticBackend: Send + Sync {
    fn batch_add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch;
    fn batch_sub(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch;   // NEW
    fn batch_mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch;
    fn batch_crt(&self, a: &RnsBatch) -> Vec<BigInt>;             // BigInt (signed), not BigUint
    fn name(&self) -> &'static str;
}
```

**This is the fix for original Tier-1 #3.**

---

## 6. Level-by-level redesign

### Level 0 — `rns.rs` → `AdelicInt`
- Finite: balanced RNS over `Basis`. Infinite: `Ball::point` of the exact value when small,
  or a width-0 ball tracked through ops.
- BigInt appears only inside `batch_crt` / reconstruction at the boundary.

### Level 1 — `rational.rs` → `RnsFrac` + `AdelicRat`
- **Store one residue per channel**: the field element `numer · denom⁻¹ (mod pᵢ)` (a modular
  fraction), *not* separate numer/denom residues. Add/sub/mul are then identical to integer
  RNS — single-residue, channel-parallel. (Channel pᵢ that divides the denominator is the
  only subtlety: that fraction is non-p-adically-integral; mark such channels invalid for
  this value and exclude them from *that value's* reconstruction. This is a per-value mask,
  **not** the unsound "skip channels to save power" from the original — see §7.)
- Exact value recovered by `rational_reconstruct`. GCD reduction happens once, at the
  boundary, on the reconstructed pair.
- Infinite `Ball` gives compare/sign for free (kills the original's BigInt-per-compare).

### Level 2 — `algebraic.rs`
- **Resultants are multimodular** (§3): bound the height, provision the basis, compute the
  Sylvester determinant mod each prime in parallel, CRT + reconstruct. No monolithic BigInt
  resultant.
- Add **square-free factorization** (gcd(f, f′)) before Sturm — required for correct root
  isolation; was missing.
- **Honest naming.** Resultants give an *annihilating* polynomial, not necessarily minimal.
  Rename the field `annihilating_poly`. Provide `try_minimize()` that runs square-free +
  (optional, behind a feature) factorization over ℚ. Reductions that need degree-drop
  (`√2·√2 → 2`) go through `try_minimize`; document that without factorization they are
  best-effort, not guaranteed.
- **Root isolation uses the `Ball`/interval directly.** Bisection + Sturm sign-counts, with
  the refine-on-`None` discipline from §2. No sign-magnitude, no separate "sign()" hack.

### Level 3 — `computable.rs`
- **Native return type is `Ball`, not a single rational.** The trait becomes:
  ```rust
  pub trait Computable: Send + Sync {
      /// Return a Ball of width < 10^(-precision) containing self.
      fn enclose(&self, precision: u64) -> Ball;
  }
  ```
  Now `mul`, `recip`, `add` compose *rigorously* via interval arithmetic — the original's
  single-rational return could not certify error and was unsound near zero.
- **Series partial sums (Chudnovsky, exp, etc.) use binary splitting over BigInt rationals**,
  *not* fixed-RNS rationals. This is the one place BigInt is legitimately load-bearing
  (§9) — the partial-sum denominators are astronomically large by construction and that
  largeness is irreducible information, not an artifact. Optionally compute the split
  multimodularly, but the honest default is `num-bigint` here.
- Chudnovsky's `640320^(3k+3/2)` half-power is handled the standard way: factor out the
  single `√640320³`, run binary splitting on the rational part, take one square root at the
  end via `Ball`. (The original tried to keep the whole term rational — impossible.)

### Level 4 — `symbolic.rs`
- Add a **canonicalization pass** (flatten n-ary Add/Mul, sort by a total order, fold
  rationals) *before* identity-table lookup. Otherwise `sin(π/6)` — stored as
  `Sin(Mul([Rational(1,6), Pi]))` — never matches a literal pattern.
- **Drop or actually support `exp(iπ)+1 → 0`.** There is no imaginary unit in `SymbolicExpr`,
  so the rule is unrepresentable. Either remove it or add a `Complex`/`I` variant; do not
  ship a rule that can't be expressed.
- This layer does **not** touch RNS or `Basis`; it is purely structural.

---

## 7. `dispatch.rs` reframed — analyzer, not compute-skipper

The original premise ("activate only the primes dividing the natural base; idle channels
save power") is **mathematically unsound**: CRT reconstruction needs *all* channels, so
dropping channels 5,7,11,… makes the result recoverable only mod (small), i.e. useless.

Reframe the module's job:

- **Exactness / base analysis (keep, it's the good part).** Given operands, report the
  natural base = LCM of radicals of denominators, and `exact_in_base(b)`. This is a
  *classification* of the answer, not a way to skip arithmetic.
- **Basis provisioning (new, this is its real value).** Estimate the result's height (§3a)
  and call `Basis::extend_to_bits` so the multimodular pipeline has enough primes. The
  dispatcher becomes the front-end to the adaptive-basis discipline.
- **Per-value channel validity mask (distinct concept).** A *single* value whose denominator
  is divisible by pᵢ is non-integral at p — exclude pᵢ from *that value's* reconstruction.
  This is correctness bookkeeping for one number, never a global power optimization.

Kill `channel_efficiency` as a "% idle" power metric; replace with `provisioned_channels` /
`required_channels` as a *bound-tightness* diagnostic.

---

## 8. Backend / GPU consolidation

- **Delete the entire second GPU design** (`GpuEngine`, `feature = "gpu"`, the rational
  numer/denom shader). It contradicts the no-feature-flag Cargo decision, and its shader uses
  `u64(...)` casts that **do not exist in core WGSL** — it is uncompilable. Keep only the
  single `GpuBackend` / `ArithmeticBackend` design.
- **One shader file per op**, integer balanced-residue, primes in (2^15, 2^16) → naive
  `(a*b)%m` fits u32. Add `rns_sub.wgsl`.
- Fix the **Garner CRT underflow**: `c[i] = (c[i] - c[j]) * inv` must be
  `c[i] = ((c[i] + m[i] - (c[j] % m[i])) % m[i]) * inv % m[i]` to avoid u64 wrap.
- `as_u32_bytes`: store residues as `u32` natively so the bytemuck cast is actually
  zero-copy; drop the false "zero-copy" claim on the u64→u32 path.
- Make `CpuBackend` use its own pool consistently (`pool.install(|| …)`), or delete the
  custom pool and use the global one. Don't half-use it.
- Define the error types that `thiserror` was imported for: `RangeError`, `BasisError`,
  `GpuError`, `ChannelMismatch`. Validate operand `Basis` equality (debug assert + typed
  error).
- Computable-real cache: key by precision but **return any cached entry with key ≥ request**
  (a 50-digit enclosure satisfies a 10-digit ask). Pick one mutex (`parking_lot`) and use it.

---

## 9. What BigInt survives — and why it must (the honesty section)

BigInt is **removed from all working arithmetic** (Levels 0–2 ring ops, batch add/sub/mul).
It survives in exactly three places, each principled:

1. **Reconstruction boundary (CRT + rational reconstruction).** Touches numbers of size ~M
   for an instant when a finite value is globalized to an exact rational. A value of
   magnitude M carries ~log₂M bits of *irreducible* information; somewhere, at the moment you
   make it one contiguous number, those bits exist. You can distribute, defer, and
   parallelize them — you cannot make them weigh less. This is the product formula biting:
   what you saved at the finite places is paid at the infinite place on output.

2. **Series partial sums at Level 3** (binary splitting). The denominators are genuinely huge
   by construction; this is real information, not waste. Multimodular splitting can shard it,
   but the honest baseline is `num-bigint` rationals.

3. **`Ball` endpoints.** Tiny BigInt rationals bounded by *requested precision*, never by M.

If BigInt ever appears in an inner ring loop after this refactor, that's a regression to flag.

---

## 10. Migration order (phased, each phase green before the next)

**Phase 0 — Structural cleanup (low risk, unblocks everything).**
- Resolve duplicate "Module 5"/"Module 6" headings; delete the orphaned `GpuEngine` design.
- Define all error types; add `Basis::modulus`; fix Garner underflow.
- These can land before any architecture change and immediately de-risk the repo.

**Phase 1 — The adelic carrier (the heart).**
- New: `ball.rs` (`Ball` + interval arithmetic), `basis.rs` (adaptive `Basis`),
  `reconstruct.rs` (`rational_reconstruct`), `bounds.rs` (Hadamard/Mignotte).
- New: `adelic.rs` (`Adelic<F>`, `AdelicInt`, `AdelicRat`) with the debug invariant
  "finite reconstruction ∈ infinite ball."
- Convert RNS to **balanced residues**; add `batch_sub`.

**Phase 2 — Rebuild Levels 0/1 on the carrier.**
- `RnsInt` → balanced; `RnsFrac` → single-residue modular fractions + reconstruction.
- All compare/sign routed through `Ball`. Verify no BigInt in ring ops.

**Phase 3 — Level 2 multimodular.**
- Multimodular resultants via `bounds` + `Basis::with_bits`; square-free factorization;
  interval root isolation; rename `min_poly → annihilating_poly` + `try_minimize`.

**Phase 4 — Level 3 ball-native.**
- `Computable::enclose -> Ball`; binary-splitting series over BigInt; Chudnovsky √-factoring.

**Phase 5 — Level 4 canonicalization.**
- Canonical form before identity lookup; remove/repair the Euler rule.

**Phase 6 — dispatch reframe + GPU consolidation + benchmarks.**
- Dispatcher = analyzer + basis provisioner. Single GPU backend. Re-run CPU/GPU bit-identity
  tests over the balanced-residue ops.

Original sections still valid as-is: `primes.rs` (extend with the bounds helpers),
`RnsBatch` flat layout (now `u32`, balanced, with `batch_sub`), the Garner *structure*
(with the underflow fix), and the overall "build parallelism foundation first" ordering.

---

## 11. New / changed test requirements (the ones that would have caught the bug)

```text
# Overflow is DETECTED, never silent  ── the headline regression test
let basis = Basis::with_bits(40);                 // deliberately too small
let big   = AdelicInt::from_bigint(&(BigInt::from(1) << 60), basis);
assert!(matches!(big.try_exact(), Err(RangeError::ReconstructionFailed{..})));
let grown = big.with_basis(basis.extend_to_bits(80));
assert_eq!(grown.try_exact().unwrap(), BigInt::from(1) << 60);   // now exact

# Sign comes from the infinite place, correctly, with mixed signs
let a = AdelicInt::from_i64(-7, b());  let c = AdelicInt::from_i64(5, b());
assert_eq!(a.add(&c).sign(), Some(Sign::Minus));   // -2, was broken under sign-magnitude
assert_eq!(a.sub(&c).sign(), Some(Sign::Minus));   // -12, batch_sub exercised

# Balanced subtraction is channel-parallel and exact (no reconstruction in the op)
prop: for random i64 x,y:  (x - y) reconstructs to x - y across many random bases

# Multimodular resultant == direct BigInt resultant (oracle test)
prop: Res_multimodular(f, g) == Res_bigint(f, g)   for random small-degree f,g

# Rational reconstruction round-trips and respects its bound
prop: reconstruct(encode(p/q), M) == (p,q)  whenever 2*max(|p|,|q|)^2 < M
      and == None                            whenever the bound is violated

# Ball composition is rigorous (the Level-3 fix)
let r = pi().enclose(50).recip();     assert!(r.width() < 1e-40);
assert!(e().mul(&pi()).enclose(30).contains(E*PI));  // certified, not point-estimate

# sin(π/6) actually simplifies (canonicalization fix)
assert_eq!(simplify(parse("sin(pi/6)")), Rational(1,2));

# No BigInt in the hot loop (architectural guard, e.g. via a counting allocator in tests)
assert_eq!(bigint_alloc_count_during(|| batch_mul(&a,&b)), 0);
```

---

## 12. Cargo.toml deltas

```toml
# add
num-rational = "0.4"     # exact Ball endpoints + Level-3 series partial sums
half          = { optional via feature, if Ball f64 directed-rounding needs it }

# keep
num-bigint, num-traits, num-integer, thiserror, rayon, parking_lot, bytemuck, wgpu, pollster

# residues now u32 natively (GPU primes in (2^15, 2^16)); RnsBatch.data: Vec<u32>
# no "gpu" feature flag (unchanged decision) — wgpu probes at runtime
```

---

## 13. The reframe in one paragraph (for the crate's top-level doc)

`adele-ring` carries every number at two kinds of place at once. The **finite places** (an
adaptive residue-number system over a prime basis) do all the real arithmetic: carry-free,
local, embarrassingly parallel across CPU lanes and GPU threads, and free of big-integer
work. The **infinite place** (a rigorous real interval) answers every question the prime
channels constitutionally cannot — sign, comparison, magnitude, and decimal output — by
refining on demand. Exactness is recovered by reconstructing from the finite places, with the
basis grown to exactly the height the computation provably needs, so a result can never
silently exceed its range. Big integers appear only at that reconstruction boundary, for an
instant, on numbers as large as the answer's own information content — which is the product
formula reminding us that what we save among the primes we pay, once, at infinity.
