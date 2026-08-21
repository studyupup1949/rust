//! Level 2 — ℚ̄. Exact algebraic numbers as (minimal polynomial, isolating
//! interval) pairs.
//!
//! We never store the decimal expansion of √2: we store that it is *the unique
//! root of `x² - 2` in `(1, 2)`*. Arithmetic on algebraic numbers is done with
//! **resultants** — the sum α+β is a root of `Res_y(p(y), q(x-y))` and the
//! product is a root of `Res_y(p(y), yᵈ q(x/y))` — followed by factorization
//! over ℚ and re-isolation of the relevant root.
//!
//! Resultants are computed as Sylvester-matrix determinants over the polynomial
//! ring ℚ[x], using the fraction-free Bareiss algorithm so every intermediate
//! division is exact.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::basis::Basis;
use crate::primes::{lcm as u64_lcm, mod_inverse};
use crate::rational::RnsRational;
use crate::rns::crt_balanced;

/// A univariate polynomial with exact rational coefficients.
///
/// `coeffs[i]` is the coefficient of `xⁱ`. Trailing zero coefficients are
/// trimmed so the last entry (if any) is the nonzero leading coefficient.
#[derive(Clone, Debug)]
pub struct Polynomial {
    pub coeffs: Vec<RnsRational>,
    pub channels: Basis,
}

impl Polynomial {
    /// Build from coefficients (ascending powers), trimming trailing zeros.
    pub fn new(coeffs: Vec<RnsRational>, channels: Basis) -> Self {
        let mut p = Polynomial { coeffs, channels };
        p.trim();
        p
    }

    /// Build from integer coefficients (ascending powers).
    pub fn from_int_coeffs(coeffs: &[i64], channels: Basis) -> Self {
        let c = coeffs
            .iter()
            .map(|&v| RnsRational::from_int(v, channels.clone()))
            .collect();
        Self::new(c, channels)
    }

    fn r_zero(&self) -> RnsRational {
        RnsRational::zero(self.channels.clone())
    }
    fn r_one(&self) -> RnsRational {
        RnsRational::from_int(1, self.channels.clone())
    }

    fn trim(&mut self) {
        while self.coeffs.len() > 0 && self.coeffs.last().unwrap().is_zero() {
            self.coeffs.pop();
        }
    }

    /// The zero polynomial.
    pub fn zero(channels: Basis) -> Self {
        Polynomial { coeffs: vec![], channels }
    }

    /// The constant polynomial `1`.
    pub fn one(channels: Basis) -> Self {
        Self::from_int_coeffs(&[1], channels)
    }

    /// A constant polynomial.
    pub fn constant(c: RnsRational) -> Self {
        let ch = c.channels.clone();
        Self::new(vec![c], ch)
    }

    /// `true` iff this is the zero polynomial.
    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    /// Degree; the zero polynomial has degree 0 by convention here (callers
    /// should also check [`Polynomial::is_zero`]).
    pub fn degree(&self) -> usize {
        self.coeffs.len().saturating_sub(1)
    }

    /// The leading (highest-power) coefficient, or zero for the zero polynomial.
    pub fn leading(&self) -> RnsRational {
        self.coeffs.last().cloned().unwrap_or_else(|| self.r_zero())
    }

    /// Evaluate at a rational point via Horner's scheme.
    pub fn eval(&self, x: &RnsRational) -> RnsRational {
        let mut acc = self.r_zero();
        for c in self.coeffs.iter().rev() {
            acc = acc.mul(x).add(c);
        }
        acc
    }

    /// Sign of `p(x)`: `-1`, `0`, or `+1`.
    pub fn sign_at(&self, x: &RnsRational) -> i32 {
        self.eval(x).signum()
    }

    /// Formal derivative.
    pub fn derivative(&self) -> Self {
        if self.coeffs.len() <= 1 {
            return Self::zero(self.channels.clone());
        }
        let coeffs = self
            .coeffs
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, c)| c.mul(&RnsRational::from_int(i as i64, self.channels.clone())))
            .collect();
        Self::new(coeffs, self.channels.clone())
    }

    /// Multiply by a rational scalar.
    pub fn scalar_mul(&self, s: &RnsRational) -> Self {
        Self::new(
            self.coeffs.iter().map(|c| c.mul(s)).collect(),
            self.channels.clone(),
        )
    }

    /// Polynomial addition.
    pub fn add(&self, other: &Self) -> Self {
        let n = self.coeffs.len().max(other.coeffs.len());
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let a = self.coeffs.get(i).cloned().unwrap_or_else(|| self.r_zero());
            let b = other.coeffs.get(i).cloned().unwrap_or_else(|| self.r_zero());
            out.push(a.add(&b));
        }
        Self::new(out, self.channels.clone())
    }

    /// Polynomial subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        self.add(&other.scalar_mul(&RnsRational::from_int(-1, self.channels.clone())))
    }

    /// Polynomial multiplication.
    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero(self.channels.clone());
        }
        let mut out = vec![self.r_zero(); self.coeffs.len() + other.coeffs.len() - 1];
        for (i, a) in self.coeffs.iter().enumerate() {
            for (j, b) in other.coeffs.iter().enumerate() {
                out[i + j] = out[i + j].add(&a.mul(b));
            }
        }
        Self::new(out, self.channels.clone())
    }

    /// Long division returning `(quotient, remainder)` over the rational field.
    /// Panics if `divisor` is zero.
    pub fn divmod(&self, divisor: &Self) -> (Self, Self) {
        assert!(!divisor.is_zero(), "polynomial division by zero");
        let mut rem = self.clone();
        let d_deg = divisor.degree();
        let d_lead = divisor.leading();
        if self.is_zero() || self.degree() < d_deg {
            return (Self::zero(self.channels.clone()), rem);
        }
        let mut quot = vec![self.r_zero(); self.degree() - d_deg + 1];
        while !rem.is_zero() && rem.degree() >= d_deg {
            let shift = rem.degree() - d_deg;
            let factor = rem.leading().div(&d_lead);
            quot[shift] = factor.clone();
            // rem -= factor * x^shift * divisor
            let mut sub_coeffs = vec![self.r_zero(); shift];
            for c in &divisor.coeffs {
                sub_coeffs.push(c.mul(&factor));
            }
            let sub = Self::new(sub_coeffs, self.channels.clone());
            rem = rem.sub(&sub);
        }
        (Self::new(quot, self.channels.clone()), rem)
    }

    /// Remainder of polynomial division.
    pub fn rem(&self, divisor: &Self) -> Self {
        self.divmod(divisor).1
    }

    /// Exact division (asserts the remainder is zero).
    pub fn div_exact(&self, divisor: &Self) -> Self {
        let (q, r) = self.divmod(divisor);
        debug_assert!(r.is_zero(), "div_exact: non-zero remainder");
        q
    }

    /// Make monic (leading coefficient 1). The zero polynomial is returned as-is.
    pub fn monic(&self) -> Self {
        if self.is_zero() {
            return self.clone();
        }
        let inv = self.r_one().div(&self.leading());
        self.scalar_mul(&inv)
    }

    /// Monic GCD of two polynomials (Euclidean algorithm over ℚ).
    pub fn gcd(a: &Self, b: &Self) -> Self {
        let mut x = a.clone();
        let mut y = b.clone();
        while !y.is_zero() {
            let r = x.rem(&y);
            x = y;
            y = r;
        }
        x.monic()
    }

    /// Square-free part: `p / gcd(p, p')`, made monic.
    pub fn squarefree(&self) -> Self {
        if self.is_zero() || self.degree() == 0 {
            return self.monic();
        }
        let g = Self::gcd(self, &self.derivative());
        self.div_exact(&g).monic()
    }

    /// The Sturm sequence `s0 = p, s1 = p', s_{i+1} = -rem(s_{i-1}, s_i)`.
    pub fn sturm_sequence(&self) -> Vec<Self> {
        let mut seq = vec![self.clone(), self.derivative()];
        while !seq.last().unwrap().is_zero() {
            let n = seq.len();
            let r = seq[n - 2].rem(&seq[n - 1]);
            if r.is_zero() {
                break;
            }
            seq.push(r.scalar_mul(&RnsRational::from_int(-1, self.channels.clone())));
        }
        seq
    }

    /// Count sign changes of a Sturm sequence evaluated at `x` (zeros ignored).
    pub fn sign_changes(seq: &[Self], x: &RnsRational) -> usize {
        let mut last = 0i32;
        let mut changes = 0usize;
        for s in seq {
            let sign = s.sign_at(x);
            if sign != 0 {
                if last != 0 && sign != last {
                    changes += 1;
                }
                last = sign;
            }
        }
        changes
    }

    /// Number of distinct real roots in the half-open interval `(a, b]`.
    pub fn sturm_root_count(&self, a: &RnsRational, b: &RnsRational) -> usize {
        let seq = self.sturm_sequence();
        let va = Self::sign_changes(&seq, a);
        let vb = Self::sign_changes(&seq, b);
        va.saturating_sub(vb)
    }

    /// A Cauchy bound `B` such that every real root lies in `(-B, B)`.
    pub fn root_bound(&self) -> RnsRational {
        if self.is_zero() || self.degree() == 0 {
            return self.r_one();
        }
        let lead = self.leading();
        let mut max_ratio = self.r_zero();
        for c in &self.coeffs[..self.coeffs.len() - 1] {
            let ratio = c.div(&lead).abs();
            if ratio > max_ratio {
                max_ratio = ratio;
            }
        }
        self.r_one().add(&max_ratio)
    }

    /// Isolate every real root into its own interval `(lo, hi]`, sorted ascending.
    /// `self` should be square-free for clean isolation.
    pub fn isolate_real_roots(&self) -> Vec<(RnsRational, RnsRational)> {
        let sf = self.squarefree();
        if sf.degree() == 0 {
            return Vec::new();
        }
        let seq = sf.sturm_sequence();
        let b = sf.root_bound();
        let lo = b.neg();
        let hi = b;
        let mut out = Vec::new();
        // Minimum width guard to avoid pathological recursion (simple roots only).
        let min_width = RnsRational::new(BigInt::one(), BigInt::one() << 80, sf.channels.clone());
        Self::isolate_rec(&seq, &lo, &hi, &min_width, &mut out);
        out.sort_by(|x, y| x.0.cmp(&y.0));
        out
    }

    fn isolate_rec(
        seq: &[Self],
        lo: &RnsRational,
        hi: &RnsRational,
        min_width: &RnsRational,
        out: &mut Vec<(RnsRational, RnsRational)>,
    ) {
        let cnt = Self::sign_changes(seq, lo).saturating_sub(Self::sign_changes(seq, hi));
        if cnt == 0 {
            return;
        }
        if cnt == 1 {
            out.push((lo.clone(), hi.clone()));
            return;
        }
        let width = hi.sub(lo);
        if width < *min_width {
            out.push((lo.clone(), hi.clone()));
            return;
        }
        let mid = lo.midpoint(hi);
        Self::isolate_rec(seq, lo, &mid, min_width, out);
        Self::isolate_rec(seq, &mid, hi, min_width, out);
    }

    // ── Factorization over ℚ (sufficient for low-degree results) ─────────────

    /// Clear denominators and the integer content, returning primitive integer
    /// coefficients (ascending powers).
    fn primitive_int_coeffs(&self) -> Vec<BigInt> {
        if self.is_zero() {
            return vec![BigInt::zero()];
        }
        // Common denominator across all coefficients.
        let mut denom_lcm = 1u64;
        let mut pairs = Vec::new();
        for c in &self.coeffs {
            let (p, q) = c.to_pair();
            let qu = q.to_u64().unwrap_or(1);
            denom_lcm = u64_lcm(denom_lcm, qu);
            pairs.push((p, q));
        }
        let big_lcm = BigInt::from(denom_lcm);
        let mut ints: Vec<BigInt> = pairs
            .iter()
            .map(|(p, q)| p * (&big_lcm / q))
            .collect();
        // Divide by integer content.
        let mut content = BigInt::zero();
        for v in &ints {
            content = content.gcd(v);
        }
        if !content.is_zero() && content != BigInt::one() {
            for v in &mut ints {
                *v /= &content;
            }
        }
        ints
    }

    /// Find one rational root via the rational-root theorem, or `None`.
    pub fn find_rational_root(&self) -> Option<RnsRational> {
        let ints = self.primitive_int_coeffs();
        if ints.len() <= 1 {
            return None;
        }
        let a0 = ints.first().unwrap().clone();
        let an = ints.last().unwrap().clone();
        // Zero is a root iff the constant term is zero.
        if a0.is_zero() {
            return Some(self.r_zero());
        }
        let p_divs = divisors(&a0.abs());
        let q_divs = divisors(&an.abs());
        for p in &p_divs {
            for q in &q_divs {
                for sign in [1i64, -1] {
                    let cand = RnsRational::new(
                        BigInt::from(sign) * p,
                        q.clone(),
                        self.channels.clone(),
                    );
                    if self.eval(&cand).is_zero() {
                        return Some(cand);
                    }
                }
            }
        }
        None
    }

    /// Factor into monic irreducible-over-ℚ factors (for the low degrees that
    /// arise from resultants of quadratics/cubics, this is exact; higher-degree
    /// factors that resist rational-root splitting are returned whole).
    pub fn factor_over_q(&self) -> Vec<Self> {
        let mut work = self.squarefree();
        let mut factors = Vec::new();
        loop {
            if work.degree() == 0 {
                break;
            }
            match work.find_rational_root() {
                Some(r) => {
                    // factor (x - r)
                    let lin = Self::new(
                        vec![r.neg(), work.r_one()],
                        self.channels.clone(),
                    );
                    work = work.div_exact(&lin);
                    factors.push(lin.monic());
                }
                None => break,
            }
        }
        if work.degree() >= 1 {
            factors.push(work.monic());
        }
        factors
    }
}

impl PartialEq for Polynomial {
    fn eq(&self, other: &Self) -> bool {
        self.coeffs == other.coeffs
    }
}
impl Eq for Polynomial {}

/// Positive integer divisors of `n` (assumes `n` fits in `u128`).
fn divisors(n: &BigInt) -> Vec<BigInt> {
    let mut out = vec![BigInt::one()];
    let n_u = match n.to_u128() {
        Some(v) if v > 0 => v,
        _ => return out,
    };
    let mut divs = Vec::new();
    let mut d = 1u128;
    while d * d <= n_u {
        if n_u % d == 0 {
            divs.push(d);
            if d != n_u / d {
                divs.push(n_u / d);
            }
        }
        d += 1;
    }
    out.clear();
    for v in divs {
        out.push(BigInt::from(v));
    }
    out
}

// ── Bivariate machinery for resultants ───────────────────────────────────────
//
// A bivariate polynomial in `y` is represented as `Vec<Polynomial>`, where entry
// `k` is the coefficient of `yᵏ` and is itself a polynomial in `x`.
type BiPoly = Vec<Polynomial>;

fn bi_degree(p: &BiPoly) -> usize {
    let mut d = 0;
    for (i, c) in p.iter().enumerate() {
        if !c.is_zero() {
            d = i;
        }
    }
    d
}

/// Resultant of two polynomials in `y` (coefficients in ℚ[x]) w.r.t. `y`,
/// returned as a polynomial in `x`, computed as the Sylvester determinant.
///
/// This is **multimodular** (refactor plan §3/§6): the Sylvester matrix is
/// cleared of denominators to integers, an a-priori coefficient bound sizes an
/// adaptive [`Basis`], then the determinant is computed modulo each prime *in
/// parallel* — each prime's determinant via evaluation/interpolation in `x`
/// (scalar Gaussian elimination over the field 𝔽_p, no fraction-free subtleties)
/// — and the coefficients are CRT-reconstructed. No monolithic BigInt
/// determinant is ever formed; `BigInt` appears only at the CRT boundary.
///
/// Row scaling during denominator-clearing multiplies the determinant by a
/// nonzero constant, which does not change the resultant's *roots* — all the
/// algebraic layer consumes (it factors / monicizes the result downstream).
fn resultant_y(a: &BiPoly, b: &BiPoly, channels: &Basis) -> Polynomial {
    let m = bi_degree(a);
    let n = bi_degree(b);
    let size = m + n;
    if size == 0 {
        return Polynomial::one(channels.clone());
    }
    let zero = Polynomial::zero(channels.clone());
    let mut mat = vec![vec![zero.clone(); size]; size];

    // Rows from `a`: coefficients high→low, shifted right by the row index.
    for i in 0..n {
        for j in 0..=m {
            mat[i][i + j] = a[m - j].clone();
        }
    }
    // Rows from `b`.
    for i in 0..m {
        for j in 0..=n {
            mat[n + i][i + j] = b[n - j].clone();
        }
    }

    let int_mat = clear_denominators(&mat);
    let det = multimodular_det(&int_mat);
    int_poly_to_polynomial(&det, channels)
}

/// An integer polynomial in `x`, ascending coefficients.
type IntPoly = Vec<BigInt>;

/// Clear denominators row-by-row, returning an integer Sylvester matrix. Each
/// row is scaled by its denominator LCM (a constant ⇒ only a constant change to
/// the determinant ⇒ identical roots).
fn clear_denominators(mat: &[Vec<Polynomial>]) -> Vec<Vec<IntPoly>> {
    mat.iter()
        .map(|row| {
            let mut l = BigInt::one();
            for entry in row {
                for c in &entry.coeffs {
                    let (_, q) = c.to_pair();
                    l = l.lcm(&q);
                }
            }
            row.iter()
                .map(|entry| {
                    entry
                        .coeffs
                        .iter()
                        .map(|c| {
                            let (p, q) = c.to_pair();
                            p * (&l / &q)
                        })
                        .collect::<IntPoly>()
                })
                .collect()
        })
        .collect()
}

/// Multimodular determinant of an `N×N` matrix of integer polynomials in `x`.
fn multimodular_det(mat: &[Vec<IntPoly>]) -> IntPoly {
    let n = mat.len();
    if n == 0 {
        return vec![BigInt::one()];
    }

    // Degree-in-x bound: the determinant's x-degree is at most the sum of the
    // per-row maximum entry degrees.
    let deg_x: usize = mat
        .iter()
        .map(|row| row.iter().map(|e| e.len().saturating_sub(1)).max().unwrap_or(0))
        .sum();
    let slots = deg_x + 1;

    // Coefficient height bound: |det coeff| ≤ ∏_i (Σ_j ‖entry_ij‖₁) — the
    // permanent of the 1-norm matrix, itself ≤ the product of row sums. Provision
    // a basis whose product M exceeds twice that (balanced lift needs M > 2|c|).
    let mut bits: u64 = 2; // sign + slack
    for row in mat {
        let row_sum: BigInt = row
            .iter()
            .map(|e| e.iter().map(|c| c.abs()).sum::<BigInt>())
            .sum();
        bits += row_sum.bits().max(1);
    }
    let basis = Basis::with_bits(bits);
    let moduli = basis.moduli().to_vec();

    // Determinant polynomial modulo each prime, in parallel.
    use rayon::prelude::*;
    let per_prime: Vec<Vec<u32>> = moduli
        .par_iter()
        .map(|&p| det_poly_mod_p(mat, deg_x, p as u64))
        .collect();

    // CRT each coefficient slot across all primes (balanced lift).
    (0..slots)
        .map(|s| {
            let residues: Vec<u32> = per_prime.iter().map(|poly| poly[s]).collect();
            crt_balanced(&residues, &moduli)
        })
        .collect()
}

/// One prime's determinant polynomial (mod `p`), via evaluation at `deg_x + 1`
/// points and interpolation. Returns `deg_x + 1` coefficients in `[0, p)`.
fn det_poly_mod_p(mat: &[Vec<IntPoly>], deg_x: usize, p: u64) -> Vec<u32> {
    let n = mat.len();
    let num_points = deg_x + 1;
    let mut ys = Vec::with_capacity(num_points);
    for t in 0..num_points {
        let tt = (t as u64) % p;
        let mut a = vec![vec![0u64; n]; n];
        for (i, row) in mat.iter().enumerate() {
            for (j, entry) in row.iter().enumerate() {
                a[i][j] = eval_int_poly_mod(entry, tt, p);
            }
        }
        ys.push(det_scalar_mod_p(a, p));
    }
    interpolate_mod_p(&ys, p).into_iter().map(|c| c as u32).collect()
}

/// Horner evaluation of an integer polynomial at `t`, modulo `p`.
fn eval_int_poly_mod(poly: &[BigInt], t: u64, p: u64) -> u64 {
    let pb = BigInt::from(p);
    let mut acc = 0u64;
    for c in poly.iter().rev() {
        let cm = c.mod_floor(&pb).to_u64().unwrap_or(0);
        acc = (acc * (t % p) + cm) % p;
    }
    acc
}

/// Determinant of a scalar matrix over the field 𝔽_p, by Gaussian elimination
/// with partial pivoting. Returns a value in `[0, p)`.
fn det_scalar_mod_p(mut a: Vec<Vec<u64>>, p: u64) -> u64 {
    let n = a.len();
    let mut det = 1u64;
    for k in 0..n {
        let pivot = (k..n).find(|&i| a[i][k] % p != 0);
        let piv = match pivot {
            Some(i) => i,
            None => return 0, // singular mod p ⇒ det ≡ 0
        };
        if piv != k {
            a.swap(piv, k);
            det = (p - det) % p; // row swap flips the sign
        }
        let inv = mod_inverse(a[k][k], p).expect("nonzero element of a field is invertible");
        det = det * (a[k][k] % p) % p;
        let pivot_row = a[k].clone();
        for row in a.iter_mut().skip(k + 1) {
            let factor = row[k] * inv % p;
            if factor == 0 {
                continue;
            }
            for (j, &pj) in pivot_row.iter().enumerate().skip(k) {
                let sub = factor * (pj % p) % p;
                row[j] = (row[j] + p - sub) % p;
            }
        }
    }
    det % p
}

/// Interpolate the polynomial (mod `p`) through points `(0, ys[0]), (1, ys[1]),
/// …` via Lagrange, returning monomial coefficients (ascending), `ys.len()` long.
fn interpolate_mod_p(ys: &[u64], p: u64) -> Vec<u64> {
    let np = ys.len();
    let mut coeffs = vec![0u64; np];
    for (s, &ys_s) in ys.iter().enumerate() {
        // numerator poly ∏_{t≠s} (x - t), and denominator ∏_{t≠s} (s - t).
        let mut num = vec![1u64];
        let mut denom = 1u64;
        for t in 0..np {
            if t == s {
                continue;
            }
            num = poly_mul_linear(&num, t as u64, p);
            let diff = (s as i64 - t as i64).rem_euclid(p as i64) as u64;
            denom = denom * diff % p;
        }
        let scale = ys_s % p * mod_inverse(denom, p).expect("distinct nodes are invertible") % p;
        for (k, &nc) in num.iter().enumerate() {
            coeffs[k] = (coeffs[k] + scale * nc) % p;
        }
    }
    coeffs
}

/// Multiply an ascending-coefficient polynomial by the linear factor `(x - t)`,
/// modulo `p`.
fn poly_mul_linear(c: &[u64], t: u64, p: u64) -> Vec<u64> {
    let mut r = vec![0u64; c.len() + 1];
    for (k, &ck) in c.iter().enumerate() {
        r[k + 1] = (r[k + 1] + ck) % p; // x · c[k]
        let neg = (p - (t * ck % p)) % p; // -t · c[k]
        r[k] = (r[k] + neg) % p;
    }
    r
}

/// Build a `Polynomial` (over `channels`) from integer coefficients.
fn int_poly_to_polynomial(coeffs: &[BigInt], channels: &Basis) -> Polynomial {
    let rcoeffs = coeffs
        .iter()
        .map(|c| RnsRational::new(c.clone(), BigInt::one(), channels.clone()))
        .collect();
    Polynomial::new(rcoeffs, channels.clone())
}

/// Lift a univariate `p(y)` to a `BiPoly` (constant-in-x coefficients).
fn lift_const(p: &Polynomial) -> BiPoly {
    p.coeffs.iter().map(|c| Polynomial::constant(c.clone())).collect()
}

/// Build `q(x - y)` as a `BiPoly` in `y`.
fn shift_sub(q: &Polynomial, channels: &Basis) -> BiPoly {
    // base = (x - y): as poly in y, coeff[0] = x, coeff[1] = -1.
    let x_poly = Polynomial::from_int_coeffs(&[0, 1], channels.clone());
    let neg_one = Polynomial::from_int_coeffs(&[-1], channels.clone());
    let base: BiPoly = vec![x_poly, neg_one];

    let mut acc: BiPoly = vec![Polynomial::zero(channels.clone())];
    let mut power: BiPoly = vec![Polynomial::one(channels.clone())]; // (x - y)^0
    for (j, c) in q.coeffs.iter().enumerate() {
        if j > 0 {
            power = bi_mul(&power, &base, channels);
        }
        let term = bi_scalar(&power, c);
        acc = bi_add(&acc, &term, channels);
    }
    acc
}

/// Build `yᵈ · q(x/y)` as a `BiPoly` in `y` (for products).
fn invert_scale(q: &Polynomial, channels: &Basis) -> BiPoly {
    let d = q.degree();
    let mut out: BiPoly = vec![Polynomial::zero(channels.clone()); d + 1];
    for (j, c) in q.coeffs.iter().enumerate() {
        // term c_j * x^j * y^(d-j)
        let mut xj = vec![0i64; j + 1];
        xj[j] = 1;
        let x_pow = Polynomial::from_int_coeffs(&xj, channels.clone());
        out[d - j] = x_pow.scalar_mul(c);
    }
    out
}

fn bi_add(a: &BiPoly, b: &BiPoly, channels: &Basis) -> BiPoly {
    let n = a.len().max(b.len());
    (0..n)
        .map(|i| {
            let za = a.get(i).cloned().unwrap_or_else(|| Polynomial::zero(channels.clone()));
            let zb = b.get(i).cloned().unwrap_or_else(|| Polynomial::zero(channels.clone()));
            za.add(&zb)
        })
        .collect()
}

fn bi_scalar(a: &BiPoly, s: &RnsRational) -> BiPoly {
    a.iter().map(|c| c.scalar_mul(s)).collect()
}

fn bi_mul(a: &BiPoly, b: &BiPoly, channels: &Basis) -> BiPoly {
    if a.is_empty() || b.is_empty() {
        return vec![Polynomial::zero(channels.clone())];
    }
    let mut out = vec![Polynomial::zero(channels.clone()); a.len() + b.len() - 1];
    for (i, ca) in a.iter().enumerate() {
        for (j, cb) in b.iter().enumerate() {
            out[i + j] = out[i + j].add(&ca.mul(cb));
        }
    }
    out
}

/// An exact real algebraic number (Level 2).
#[derive(Clone, Debug)]
pub struct AlgebraicNumber {
    /// An **annihilating** polynomial over ℚ: monic, with this number as a root.
    ///
    /// Resultants produce an annihilating polynomial that is not *necessarily*
    /// minimal. [`AlgebraicNumber::try_minimize`] reduces it toward the minimal
    /// polynomial (square-free + rational-root factorization); without full
    /// factorization over ℚ this is best-effort, not guaranteed.
    pub annihilating_poly: Polynomial,
    /// Isolating interval `(lo, hi)` containing exactly one root of `annihilating_poly`.
    pub interval: (RnsRational, RnsRational),
    pub channels: Basis,
}

impl AlgebraicNumber {
    /// The positive square root √n.
    pub fn sqrt(n: u64, channels: Basis) -> Self {
        // min poly x² - n, isolate the positive root in (0, n+1).
        let annihilating_poly = Polynomial::from_int_coeffs(&[-(n as i64), 0, 1], channels.clone());
        let lo = RnsRational::from_int(0, channels.clone());
        let hi = RnsRational::from_int(n as i64 + 1, channels.clone());
        Self::from_annihilating_poly_interval(annihilating_poly, lo, hi, channels)
    }

    /// The real cube root ∛n.
    pub fn cbrt(n: u64, channels: Basis) -> Self {
        let annihilating_poly = Polynomial::from_int_coeffs(&[-(n as i64), 0, 0, 1], channels.clone());
        let lo = RnsRational::from_int(0, channels.clone());
        let hi = RnsRational::from_int(n as i64 + 1, channels.clone());
        Self::from_annihilating_poly_interval(annihilating_poly, lo, hi, channels)
    }

    /// A rational as a degree-1 algebraic number.
    pub fn from_rational(r: RnsRational) -> Self {
        let channels = r.channels.clone();
        let annihilating_poly = Polynomial::new(
            vec![r.neg(), RnsRational::from_int(1, channels.clone())],
            channels.clone(),
        );
        let lo = r.sub(&RnsRational::from_int(1, channels.clone()));
        let hi = r.add(&RnsRational::from_int(1, channels.clone()));
        AlgebraicNumber { annihilating_poly, interval: (lo, hi), channels }
    }

    /// The `root_index`-th real root (ascending) of `p`.
    pub fn from_poly_root(p: Polynomial, root_index: usize, channels: Basis) -> Self {
        let roots = p.isolate_real_roots();
        let (lo, hi) = roots
            .get(root_index)
            .cloned()
            .expect("root_index out of range");
        // Attach the irreducible factor that owns this root.
        let factors = p.factor_over_q();
        let annihilating_poly = Self::factor_for_interval(&factors, &lo, &hi).unwrap_or(p);
        AlgebraicNumber { annihilating_poly, interval: (lo, hi), channels }
    }

    fn from_annihilating_poly_interval(
        annihilating_poly: Polynomial,
        lo: RnsRational,
        hi: RnsRational,
        channels: Basis,
    ) -> Self {
        let mut a = AlgebraicNumber { annihilating_poly, interval: (lo, hi), channels };
        // Tighten a little so the interval cleanly isolates the root.
        let target = RnsRational::new(BigInt::one(), BigInt::from(1_000_000), a.channels.clone());
        a.refine_interval(&target);
        a
    }

    /// Degree of the annihilating polynomial.
    pub fn degree(&self) -> usize {
        self.annihilating_poly.degree()
    }

    /// Reduce the annihilating polynomial toward the **minimal** polynomial by
    /// square-free + rational-root factorization over ℚ, keeping the irreducible
    /// factor whose real root matches this number. Best-effort: without full
    /// factorization, high-degree irreducible factors are returned whole.
    pub fn try_minimize(&self) -> AlgebraicNumber {
        let factors = self.annihilating_poly.squarefree().factor_over_q();
        let approx = self.to_f64();
        let mut best: Option<(Polynomial, f64)> = None;
        for f in &factors {
            for (lo, hi) in f.isolate_real_roots() {
                let mut cand = AlgebraicNumber {
                    annihilating_poly: f.clone(),
                    interval: (lo, hi),
                    channels: self.channels.clone(),
                };
                let target = RnsRational::new(BigInt::one(), BigInt::from(1_000_000), self.channels.clone());
                cand.refine_interval(&target);
                let dist = (cand.to_f64() - approx).abs();
                if best.as_ref().map(|(_, d)| dist < *d).unwrap_or(true) {
                    best = Some((f.clone(), dist));
                }
            }
        }
        match best {
            Some((f, _)) => AlgebraicNumber {
                annihilating_poly: f,
                interval: self.interval.clone(),
                channels: self.channels.clone(),
            },
            None => self.clone(),
        }
    }

    /// `Some(r)` iff this number is actually rational (degree-1 min poly).
    pub fn to_rational(&self) -> Option<RnsRational> {
        if self.annihilating_poly.degree() == 1 {
            // c1 x + c0 = 0  =>  x = -c0/c1
            let c0 = self.annihilating_poly.coeffs[0].clone();
            let c1 = self.annihilating_poly.coeffs[1].clone();
            Some(c0.neg().div(&c1))
        } else {
            None
        }
    }

    /// Refine the isolating interval until its width is `< target_width`.
    pub fn refine_interval(&mut self, target_width: &RnsRational) {
        let (mut lo, mut hi) = self.interval.clone();
        let sign_lo = self.annihilating_poly.sign_at(&lo);
        // If an endpoint is exactly the root, collapse to it.
        if sign_lo == 0 {
            self.interval = (lo.clone(), lo);
            return;
        }
        if self.annihilating_poly.sign_at(&hi) == 0 {
            self.interval = (hi.clone(), hi);
            return;
        }
        while hi.sub(&lo) >= *target_width {
            let mid = lo.midpoint(&hi);
            let sm = self.annihilating_poly.sign_at(&mid);
            if sm == 0 {
                lo = mid.clone();
                hi = mid;
                break;
            } else if sm == sign_lo {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.interval = (lo, hi);
    }

    /// An `f64` approximation (refines internally first).
    pub fn to_f64(&self) -> f64 {
        let mut clone = self.clone();
        let target = RnsRational::new(BigInt::one(), BigInt::one() << 60, self.channels.clone());
        clone.refine_interval(&target);
        clone.interval.0.midpoint(&clone.interval.1).to_f64()
    }

    /// Exact sign of the number: `-1`, `0`, or `+1`.
    pub fn sign(&self) -> i32 {
        // Zero is a root iff the constant term of an irreducible min poly is 0
        // (i.e. min poly is exactly x).
        if self.annihilating_poly.degree() == 1 && self.annihilating_poly.coeffs[0].is_zero() {
            return 0;
        }
        let mut clone = self.clone();
        let zero = RnsRational::zero(self.channels.clone());
        // Refine until the interval lies strictly on one side of 0.
        let mut target = RnsRational::new(BigInt::one(), BigInt::from(1024), self.channels.clone());
        for _ in 0..200 {
            if clone.interval.0 > zero {
                return 1;
            }
            if clone.interval.1 < zero {
                return -1;
            }
            clone.refine_interval(&target);
            target = target.mul(&RnsRational::from_fraction(1, 2, self.channels.clone()));
        }
        clone.interval.0.midpoint(&clone.interval.1).signum()
    }

    /// Additive inverse: negate the root (`x -> -x` in the min poly).
    pub fn neg(&self) -> Self {
        let coeffs = self
            .annihilating_poly
            .coeffs
            .iter()
            .enumerate()
            .map(|(i, c)| if i % 2 == 1 { c.neg() } else { c.clone() })
            .collect();
        let annihilating_poly = Polynomial::new(coeffs, self.channels.clone()).monic();
        AlgebraicNumber {
            annihilating_poly,
            interval: (self.interval.1.neg(), self.interval.0.neg()),
            channels: self.channels.clone(),
        }
    }

    /// Multiplicative inverse: reciprocal of the root (reverse the coefficients).
    pub fn recip(&self) -> Self {
        let mut coeffs = self.annihilating_poly.coeffs.clone();
        coeffs.reverse();
        let annihilating_poly = Polynomial::new(coeffs, self.channels.clone()).monic();
        let v = self.to_f64();
        Self::reconstruct(annihilating_poly, 1.0 / v, self.channels.clone())
    }

    /// α + β.
    pub fn add(&self, other: &Self) -> Self {
        let channels = self.channels.clone();
        let a = lift_const(&self.annihilating_poly);
        let b = shift_sub(&other.annihilating_poly, &channels);
        let res = resultant_y(&a, &b, &channels);
        let value = self.to_f64() + other.to_f64();
        Self::reconstruct(res, value, channels)
    }

    /// α × β.
    pub fn mul(&self, other: &Self) -> Self {
        let channels = self.channels.clone();
        let a = lift_const(&self.annihilating_poly);
        let b = invert_scale(&other.annihilating_poly, &channels);
        let res = resultant_y(&a, &b, &channels);
        let value = self.to_f64() * other.to_f64();
        Self::reconstruct(res, value, channels)
    }

    /// From a resultant polynomial plus an approximate value, pick the
    /// irreducible factor and isolating interval that own the target root.
    fn reconstruct(res: Polynomial, approx: f64, channels: Basis) -> Self {
        let factors = res.factor_over_q();
        // Choose the factor whose nearest real root is closest to `approx`.
        let mut best: Option<(AlgebraicNumber, f64)> = None;
        for f in &factors {
            for (lo, hi) in f.isolate_real_roots() {
                let mut cand = AlgebraicNumber {
                    annihilating_poly: f.clone(),
                    interval: (lo, hi),
                    channels: channels.clone(),
                };
                let target = RnsRational::new(BigInt::one(), BigInt::from(1_000_000), channels.clone());
                cand.refine_interval(&target);
                let dist = (cand.to_f64() - approx).abs();
                if best.as_ref().map(|(_, d)| dist < *d).unwrap_or(true) {
                    best = Some((cand, dist));
                }
            }
        }
        best.map(|(a, _)| a).expect("resultant had no real root near target")
    }

    fn factor_for_interval(
        factors: &[Polynomial],
        lo: &RnsRational,
        hi: &RnsRational,
    ) -> Option<Polynomial> {
        for f in factors {
            if f.sturm_root_count(lo, hi) >= 1 {
                return Some(f.monic());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch() -> Basis {
        Basis::standard()
    }

    #[test]
    fn sqrt2_annihilating_poly() {
        let s = AlgebraicNumber::sqrt(2, ch());
        assert_eq!(s.degree(), 2);
        // 1² - 2 < 0
        assert_eq!(
            s.annihilating_poly.sign_at(&RnsRational::from_int(1, ch())),
            -1
        );
        assert!(s.interval.0 < s.interval.1);
        assert!(s.to_rational().is_none());
        assert!((s.to_f64() - 2f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn from_rational_roundtrip() {
        let r = RnsRational::from_fraction(3, 5, ch());
        let a = AlgebraicNumber::from_rational(r.clone());
        assert_eq!(a.to_rational(), Some(r));
    }

    #[test]
    fn sturm_counts() {
        // x² - 2 has one root in (-2,-1], one in (1,2], two in (-2,2].
        let p = Polynomial::from_int_coeffs(&[-2, 0, 1], ch());
        let r = |a: i64, b: i64| {
            p.sturm_root_count(
                &RnsRational::from_int(a, ch()),
                &RnsRational::from_int(b, ch()),
            )
        };
        assert_eq!(r(-2, -1), 1);
        assert_eq!(r(1, 2), 1);
        assert_eq!(r(-2, 2), 2);
    }

    #[test]
    fn sqrt2_times_sqrt2_is_two() {
        let s = AlgebraicNumber::sqrt(2, ch());
        let p = s.mul(&s);
        // √2·√2 = 2  =>  degree-1 min poly, rational value 2.
        assert_eq!(p.degree(), 1);
        assert_eq!(p.to_rational(), Some(RnsRational::from_int(2, ch())));
    }

    #[test]
    fn sqrt2_times_sqrt3_is_sqrt6() {
        let s2 = AlgebraicNumber::sqrt(2, ch());
        let s3 = AlgebraicNumber::sqrt(3, ch());
        let p = s2.mul(&s3);
        assert_eq!(p.degree(), 2);
        // min poly x² - 6
        let expected = Polynomial::from_int_coeffs(&[-6, 0, 1], ch()).monic();
        assert_eq!(p.annihilating_poly, expected);
    }

    #[test]
    fn sqrt2_plus_sqrt2_is_2sqrt2() {
        let s = AlgebraicNumber::sqrt(2, ch());
        let p = s.add(&s);
        // 2√2 has min poly x² - 8.
        assert_eq!(p.degree(), 2);
        let expected = Polynomial::from_int_coeffs(&[-8, 0, 1], ch()).monic();
        assert_eq!(p.annihilating_poly, expected);
    }

    #[test]
    fn refine_narrows() {
        let mut s = AlgebraicNumber::sqrt(2, ch());
        let target = RnsRational::new(BigInt::one(), BigInt::from(10).pow(20), ch());
        s.refine_interval(&target);
        assert!(s.interval.1.sub(&s.interval.0) < target);
    }

    // ── Multimodular-determinant oracle tests (refactor plan §11) ────────────

    /// Direct fraction-free (Bareiss) determinant over ℤ — the trusted oracle.
    fn det_bigint(mut m: Vec<Vec<BigInt>>) -> BigInt {
        let n = m.len();
        if n == 0 {
            return BigInt::one();
        }
        let mut sign = 1i32;
        let mut prev = BigInt::one();
        for k in 0..n - 1 {
            if m[k][k].is_zero() {
                let swap = (k + 1..n).find(|&i| !m[i][k].is_zero());
                match swap {
                    Some(i) => {
                        m.swap(k, i);
                        sign = -sign;
                    }
                    None => return BigInt::zero(),
                }
            }
            for i in k + 1..n {
                for j in k + 1..n {
                    let num = &m[i][j] * &m[k][k] - &m[i][k] * &m[k][j];
                    m[i][j] = num / &prev;
                }
                m[i][k] = BigInt::zero();
            }
            prev = m[k][k].clone();
        }
        let det = m[n - 1][n - 1].clone();
        if sign < 0 {
            -det
        } else {
            det
        }
    }

    fn eval_int_poly(poly: &[BigInt], x: &BigInt) -> BigInt {
        let mut acc = BigInt::zero();
        for c in poly.iter().rev() {
            acc = acc * x + c;
        }
        acc
    }

    /// Tiny deterministic LCG so the oracle tests need no `rand` dependency.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.0 >> 33
        }
        /// Signed integer in `[-range, range]`.
        fn coeff(&mut self, range: i64) -> i64 {
            (self.next() as i64).rem_euclid(2 * range + 1) - range
        }
    }

    #[test]
    fn multimodular_det_matches_bigint_scalar() {
        let mut rng = Lcg(0x1234_5678_9abc_def0);
        for n in 1..=6usize {
            for _ in 0..20 {
                // A random scalar (constant-in-x) integer matrix.
                let scalar: Vec<Vec<BigInt>> = (0..n)
                    .map(|_| (0..n).map(|_| BigInt::from(rng.coeff(50))).collect())
                    .collect();
                let int_mat: Vec<Vec<IntPoly>> = scalar
                    .iter()
                    .map(|row| row.iter().map(|c| vec![c.clone()]).collect())
                    .collect();

                let expected = det_bigint(scalar);
                let got = multimodular_det(&int_mat);
                // Constant polynomial: only the x⁰ slot is significant.
                let got0 = got.first().cloned().unwrap_or_else(BigInt::zero);
                assert_eq!(got0, expected, "n={n}");
                assert!(got.iter().skip(1).all(|c| c.is_zero()));
            }
        }
    }

    #[test]
    fn multimodular_det_matches_bigint_polynomial() {
        let mut rng = Lcg(0xfeed_face_cafe_d00d);
        for n in 1..=5usize {
            for _ in 0..20 {
                // Each entry a random degree-≤1 integer polynomial in x.
                let int_mat: Vec<Vec<IntPoly>> = (0..n)
                    .map(|_| {
                        (0..n)
                            .map(|_| {
                                vec![
                                    BigInt::from(rng.coeff(20)),
                                    BigInt::from(rng.coeff(20)),
                                ]
                            })
                            .collect()
                    })
                    .collect();

                let det_poly = multimodular_det(&int_mat);

                // Certify by evaluating the determinant polynomial at several
                // integer points against a direct scalar BigInt determinant.
                for x in [-3i64, -1, 0, 2, 5, 11] {
                    let xb = BigInt::from(x);
                    let scalar: Vec<Vec<BigInt>> = int_mat
                        .iter()
                        .map(|row| row.iter().map(|e| eval_int_poly(e, &xb)).collect())
                        .collect();
                    let expected = det_bigint(scalar);
                    let got = eval_int_poly(&det_poly, &xb);
                    assert_eq!(got, expected, "n={n}, x={x}");
                }
            }
        }
    }
}
