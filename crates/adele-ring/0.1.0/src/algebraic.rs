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

use crate::primes::lcm as u64_lcm;
use crate::rational::RnsRational;
use crate::rns::Channels;

/// A univariate polynomial with exact rational coefficients.
///
/// `coeffs[i]` is the coefficient of `xⁱ`. Trailing zero coefficients are
/// trimmed so the last entry (if any) is the nonzero leading coefficient.
#[derive(Clone, Debug)]
pub struct Polynomial {
    pub coeffs: Vec<RnsRational>,
    pub channels: Channels,
}

impl Polynomial {
    /// Build from coefficients (ascending powers), trimming trailing zeros.
    pub fn new(coeffs: Vec<RnsRational>, channels: Channels) -> Self {
        let mut p = Polynomial { coeffs, channels };
        p.trim();
        p
    }

    /// Build from integer coefficients (ascending powers).
    pub fn from_int_coeffs(coeffs: &[i64], channels: Channels) -> Self {
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
    pub fn zero(channels: Channels) -> Self {
        Polynomial { coeffs: vec![], channels }
    }

    /// The constant polynomial `1`.
    pub fn one(channels: Channels) -> Self {
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
/// returned as a polynomial in `x`. Computed as the Sylvester determinant.
fn resultant_y(a: &BiPoly, b: &BiPoly, channels: &Channels) -> Polynomial {
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

    bareiss_det(mat, channels)
}

/// Fraction-free (Bareiss) determinant over the polynomial ring ℚ[x].
fn bareiss_det(mut m: Vec<Vec<Polynomial>>, channels: &Channels) -> Polynomial {
    let n = m.len();
    if n == 0 {
        return Polynomial::one(channels.clone());
    }
    let mut sign = 1i32;
    let mut prev = Polynomial::one(channels.clone());
    for k in 0..n - 1 {
        if m[k][k].is_zero() {
            // Pivot: find a row below with a nonzero entry in column k.
            let mut swap = None;
            for i in (k + 1)..n {
                if !m[i][k].is_zero() {
                    swap = Some(i);
                    break;
                }
            }
            match swap {
                Some(i) => {
                    m.swap(k, i);
                    sign = -sign;
                }
                None => return Polynomial::zero(channels.clone()),
            }
        }
        for i in (k + 1)..n {
            for j in (k + 1)..n {
                let term1 = m[i][j].mul(&m[k][k]);
                let term2 = m[i][k].mul(&m[k][j]);
                let numer = term1.sub(&term2);
                m[i][j] = numer.div_exact(&prev);
            }
            m[i][k] = Polynomial::zero(channels.clone());
        }
        prev = m[k][k].clone();
    }
    let det = m[n - 1][n - 1].clone();
    if sign < 0 {
        det.scalar_mul(&RnsRational::from_int(-1, channels.clone()))
    } else {
        det
    }
}

/// Lift a univariate `p(y)` to a `BiPoly` (constant-in-x coefficients).
fn lift_const(p: &Polynomial) -> BiPoly {
    p.coeffs.iter().map(|c| Polynomial::constant(c.clone())).collect()
}

/// Build `q(x - y)` as a `BiPoly` in `y`.
fn shift_sub(q: &Polynomial, channels: &Channels) -> BiPoly {
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
fn invert_scale(q: &Polynomial, channels: &Channels) -> BiPoly {
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

fn bi_add(a: &BiPoly, b: &BiPoly, channels: &Channels) -> BiPoly {
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

fn bi_mul(a: &BiPoly, b: &BiPoly, channels: &Channels) -> BiPoly {
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
    /// Minimal polynomial over ℚ (monic, irreducible for the cases we build).
    pub min_poly: Polynomial,
    /// Isolating interval `(lo, hi)` containing exactly one root of `min_poly`.
    pub interval: (RnsRational, RnsRational),
    pub channels: Channels,
}

impl AlgebraicNumber {
    /// The positive square root √n.
    pub fn sqrt(n: u64, channels: Channels) -> Self {
        // min poly x² - n, isolate the positive root in (0, n+1).
        let min_poly = Polynomial::from_int_coeffs(&[-(n as i64), 0, 1], channels.clone());
        let lo = RnsRational::from_int(0, channels.clone());
        let hi = RnsRational::from_int(n as i64 + 1, channels.clone());
        Self::from_min_poly_interval(min_poly, lo, hi, channels)
    }

    /// The real cube root ∛n.
    pub fn cbrt(n: u64, channels: Channels) -> Self {
        let min_poly = Polynomial::from_int_coeffs(&[-(n as i64), 0, 0, 1], channels.clone());
        let lo = RnsRational::from_int(0, channels.clone());
        let hi = RnsRational::from_int(n as i64 + 1, channels.clone());
        Self::from_min_poly_interval(min_poly, lo, hi, channels)
    }

    /// A rational as a degree-1 algebraic number.
    pub fn from_rational(r: RnsRational) -> Self {
        let channels = r.channels.clone();
        let min_poly = Polynomial::new(
            vec![r.neg(), RnsRational::from_int(1, channels.clone())],
            channels.clone(),
        );
        let lo = r.sub(&RnsRational::from_int(1, channels.clone()));
        let hi = r.add(&RnsRational::from_int(1, channels.clone()));
        AlgebraicNumber { min_poly, interval: (lo, hi), channels }
    }

    /// The `root_index`-th real root (ascending) of `p`.
    pub fn from_poly_root(p: Polynomial, root_index: usize, channels: Channels) -> Self {
        let roots = p.isolate_real_roots();
        let (lo, hi) = roots
            .get(root_index)
            .cloned()
            .expect("root_index out of range");
        // Attach the irreducible factor that owns this root.
        let factors = p.factor_over_q();
        let min_poly = Self::factor_for_interval(&factors, &lo, &hi).unwrap_or(p);
        AlgebraicNumber { min_poly, interval: (lo, hi), channels }
    }

    fn from_min_poly_interval(
        min_poly: Polynomial,
        lo: RnsRational,
        hi: RnsRational,
        channels: Channels,
    ) -> Self {
        let mut a = AlgebraicNumber { min_poly, interval: (lo, hi), channels };
        // Tighten a little so the interval cleanly isolates the root.
        let target = RnsRational::new(BigInt::one(), BigInt::from(1_000_000), a.channels.clone());
        a.refine_interval(&target);
        a
    }

    /// Degree of the minimal polynomial.
    pub fn degree(&self) -> usize {
        self.min_poly.degree()
    }

    /// `Some(r)` iff this number is actually rational (degree-1 min poly).
    pub fn to_rational(&self) -> Option<RnsRational> {
        if self.min_poly.degree() == 1 {
            // c1 x + c0 = 0  =>  x = -c0/c1
            let c0 = self.min_poly.coeffs[0].clone();
            let c1 = self.min_poly.coeffs[1].clone();
            Some(c0.neg().div(&c1))
        } else {
            None
        }
    }

    /// Refine the isolating interval until its width is `< target_width`.
    pub fn refine_interval(&mut self, target_width: &RnsRational) {
        let (mut lo, mut hi) = self.interval.clone();
        let sign_lo = self.min_poly.sign_at(&lo);
        // If an endpoint is exactly the root, collapse to it.
        if sign_lo == 0 {
            self.interval = (lo.clone(), lo);
            return;
        }
        if self.min_poly.sign_at(&hi) == 0 {
            self.interval = (hi.clone(), hi);
            return;
        }
        while hi.sub(&lo) >= *target_width {
            let mid = lo.midpoint(&hi);
            let sm = self.min_poly.sign_at(&mid);
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
        if self.min_poly.degree() == 1 && self.min_poly.coeffs[0].is_zero() {
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
            .min_poly
            .coeffs
            .iter()
            .enumerate()
            .map(|(i, c)| if i % 2 == 1 { c.neg() } else { c.clone() })
            .collect();
        let min_poly = Polynomial::new(coeffs, self.channels.clone()).monic();
        AlgebraicNumber {
            min_poly,
            interval: (self.interval.1.neg(), self.interval.0.neg()),
            channels: self.channels.clone(),
        }
    }

    /// Multiplicative inverse: reciprocal of the root (reverse the coefficients).
    pub fn recip(&self) -> Self {
        let mut coeffs = self.min_poly.coeffs.clone();
        coeffs.reverse();
        let min_poly = Polynomial::new(coeffs, self.channels.clone()).monic();
        let v = self.to_f64();
        Self::reconstruct(min_poly, 1.0 / v, self.channels.clone())
    }

    /// α + β.
    pub fn add(&self, other: &Self) -> Self {
        let channels = self.channels.clone();
        let a = lift_const(&self.min_poly);
        let b = shift_sub(&other.min_poly, &channels);
        let res = resultant_y(&a, &b, &channels);
        let value = self.to_f64() + other.to_f64();
        Self::reconstruct(res, value, channels)
    }

    /// α × β.
    pub fn mul(&self, other: &Self) -> Self {
        let channels = self.channels.clone();
        let a = lift_const(&self.min_poly);
        let b = invert_scale(&other.min_poly, &channels);
        let res = resultant_y(&a, &b, &channels);
        let value = self.to_f64() * other.to_f64();
        Self::reconstruct(res, value, channels)
    }

    /// From a resultant polynomial plus an approximate value, pick the
    /// irreducible factor and isolating interval that own the target root.
    fn reconstruct(res: Polynomial, approx: f64, channels: Channels) -> Self {
        let factors = res.factor_over_q();
        // Choose the factor whose nearest real root is closest to `approx`.
        let mut best: Option<(AlgebraicNumber, f64)> = None;
        for f in &factors {
            for (lo, hi) in f.isolate_real_roots() {
                let mut cand = AlgebraicNumber {
                    min_poly: f.clone(),
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

    fn ch() -> Channels {
        Channels::standard(32)
    }

    #[test]
    fn sqrt2_min_poly() {
        let s = AlgebraicNumber::sqrt(2, ch());
        assert_eq!(s.degree(), 2);
        // 1² - 2 < 0
        assert_eq!(
            s.min_poly.sign_at(&RnsRational::from_int(1, ch())),
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
        assert_eq!(p.min_poly, expected);
    }

    #[test]
    fn sqrt2_plus_sqrt2_is_2sqrt2() {
        let s = AlgebraicNumber::sqrt(2, ch());
        let p = s.add(&s);
        // 2√2 has min poly x² - 8.
        assert_eq!(p.degree(), 2);
        let expected = Polynomial::from_int_coeffs(&[-8, 0, 1], ch()).monic();
        assert_eq!(p.min_poly, expected);
    }

    #[test]
    fn refine_narrows() {
        let mut s = AlgebraicNumber::sqrt(2, ch());
        let target = RnsRational::new(BigInt::one(), BigInt::from(10).pow(20), ch());
        s.refine_interval(&target);
        assert!(s.interval.1.sub(&s.interval.0) < target);
    }
}
