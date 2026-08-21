//! Pins how Accelerate reads a symmetric matrix stored in blocks.
//!
//! Apple documents block CSC itself — the dimensions count blocks, and each stored block carries
//! `block_size * block_size` values as a dense column-major tile. How a block meets the symmetric
//! convention that only one triangle is read is documented for the coordinate-format converters
//! but not for this factorization path, which leaves two questions answered here by observation
//! rather than by inference:
//!
//! - what happens to the unselected half of a *diagonal* tile, and
//! - whether an off-diagonal tile is transposed when it is reflected into the other triangle.
//!
//! The oracle throughout is the same matrix stored as ordinary scalar CSC, a path the rest of the
//! suite already covers. Any disagreement is therefore in the block handling and nowhere else.
//! These guard library behaviour rather than this crate's code: if a future Accelerate changed
//! either answer, a solver built on the block path would quietly return different trajectories,
//! and this is what notices instead.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, FactorizationKind, SparseStructure, SymbolicFactorization, Triangle,
};

/// A dense symmetric positive-definite matrix, row-major, `n` by `n`. Diagonally dominant by
/// construction, so both factorization kinds succeed on it.
fn spd(n: usize) -> Vec<f64> {
    let mut a = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                a[i * n + j] = 20.0 + i as f64;
            } else if (i + j) % 3 == 0 {
                a[i * n + j] = 1.0 + ((i * 7 + j * 3) % 5) as f64 * 0.25;
            }
        }
    }
    for i in 0..n {
        for j in 0..i {
            a[j * n + i] = a[i * n + j];
        }
    }
    a
}

/// A dense symmetric banded matrix, row-major, `n` by `n`, diagonally dominant at every size so
/// it stays positive definite where [`spd`]'s denser off-diagonal pattern would not. Used for the
/// large cases, where the point is the matrix size rather than a rich off-diagonal structure.
fn spd_banded(n: usize) -> Vec<f64> {
    let mut a = vec![0.0; n * n];
    for i in 0..n {
        a[i * n + i] = 8.0;
        for (offset, value) in [(1usize, -1.0), (2, -0.5)] {
            if i + offset < n {
                a[i * n + (i + offset)] = value;
                a[(i + offset) * n + i] = value;
            }
        }
    }
    a
}

fn rhs(n: usize) -> Vec<f64> {
    (0..n).map(|i| 1.0 + i as f64 * 0.5).collect()
}

/// A matrix in block CSC, kept together with the values so a perturbed copy can be solved
/// against the same pattern.
struct Blocked {
    blocks: usize,
    column_starts: Vec<i64>,
    row_indices: Vec<i32>,
    values: Vec<f64>,
    block_size: u8,
    triangle: Triangle,
}

impl Blocked {
    /// Rebuilds a dense matrix as block CSC over one block triangle.
    ///
    /// With `block_size == 1` this is ordinary scalar CSC, which is what makes it usable as both
    /// the subject and the oracle.
    fn new(a: &[f64], n: usize, block_size: usize, triangle: Triangle) -> Self {
        Self::build(a, n, block_size, triangle, false)
    }

    /// Stores *every* block of the grid while still declaring one triangle, so that what lands in
    /// the ignored half can be given a value and its fate observed.
    fn full(a: &[f64], n: usize, block_size: usize, triangle: Triangle) -> Self {
        Self::build(a, n, block_size, triangle, true)
    }

    fn build(
        a: &[f64],
        n: usize,
        block_size: usize,
        triangle: Triangle,
        store_every_block: bool,
    ) -> Self {
        assert_eq!(
            n % block_size,
            0,
            "the matrix must divide into whole blocks"
        );
        let blocks = n / block_size;
        let mut column_starts = vec![0i64];
        let mut row_indices = Vec::new();
        let mut values = Vec::new();

        for block_column in 0..blocks {
            for block_row in 0..blocks {
                let selected = match triangle {
                    Triangle::Lower => block_row >= block_column,
                    Triangle::Upper => block_row <= block_column,
                };
                if !selected && !store_every_block {
                    continue;
                }
                row_indices.push(block_row as i32);
                for c in 0..block_size {
                    for r in 0..block_size {
                        values.push(
                            a[(block_row * block_size + r) * n + (block_column * block_size + c)],
                        );
                    }
                }
            }
            column_starts.push(row_indices.len() as i64);
        }

        Self {
            blocks,
            column_starts,
            row_indices,
            values,
            block_size: block_size as u8,
            triangle,
        }
    }

    fn structure(&self) -> SparseStructure<'_> {
        SparseStructure::from_csc(
            self.blocks,
            self.blocks,
            &self.column_starts,
            &self.row_indices,
            Attributes::symmetric(self.triangle),
        )
        .expect("the pattern is well formed")
        .with_block_size(self.block_size)
        .expect("the block size is non-zero")
    }

    fn solve(&self, kind: FactorizationKind, b: &[f64]) -> Vec<f64> {
        self.solve_values(&self.values, kind, b)
    }

    /// Solves this pattern with values other than its own, for the perturbation tests.
    fn solve_values(&self, values: &[f64], kind: FactorizationKind, b: &[f64]) -> Vec<f64> {
        let structure = self.structure();
        let symbolic = SymbolicFactorization::new(kind, &structure).unwrap();
        symbolic.factorize(values).unwrap().solve_vec(b).unwrap()
    }

    /// The values with one entry replaced.
    fn perturbed(&self, index: usize, value: f64) -> Vec<f64> {
        let mut values = self.values.clone();
        values[index] = value;
        values
    }
}

/// Solves `a` stored as scalar CSC: the oracle every block case is measured against.
fn scalar_solve(a: &[f64], n: usize, kind: FactorizationKind, b: &[f64]) -> Vec<f64> {
    Blocked::new(a, n, 1, Triangle::Lower).solve(kind, b)
}

fn assert_close(got: &[f64], want: &[f64], what: &str) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!(
            (g - w).abs() <= 1e-9 * g.abs().max(1.0),
            "{what}: component {i} is {g}, expected {w}"
        );
    }
}

fn kinds() -> [FactorizationKind; 2] {
    [FactorizationKind::Cholesky, FactorizationKind::LdltSbk]
}

/// The core equivalence: a matrix stored in blocks must solve to what the same matrix stored as
/// scalars solves to. Small sizes here, across both triangles and block sizes one through four;
/// [`block_storage_agrees_at_supernodal_size`] carries the same check to a size that reaches
/// Accelerate's supernodal path.
#[test]
fn block_storage_agrees_with_scalar_storage() {
    for n in [4usize, 12, 24] {
        let a = spd(n);
        let b = rhs(n);
        for kind in kinds() {
            let oracle = scalar_solve(&a, n, kind, &b);
            for block_size in [1usize, 2, 3, 4] {
                if n % block_size != 0 {
                    continue;
                }
                for triangle in [Triangle::Lower, Triangle::Upper] {
                    let blocked = Blocked::new(&a, n, block_size, triangle);
                    assert_close(
                        &blocked.solve(kind, &b),
                        &oracle,
                        &format!("n={n} b={block_size} {triangle:?} {kind:?}"),
                    );
                }
            }
        }
    }
}

/// The unselected half of a *diagonal* tile is ignored, exactly as the unselected triangle of the
/// matrix as a whole is. Perturbing it with a value large enough to dominate the matrix must not
/// move the answer; a tolerance-sized perturbation could hide inside rounding, `1e7` cannot.
#[test]
fn the_unselected_half_of_a_diagonal_tile_is_ignored() {
    let (n, block_size) = (12usize, 3usize);
    let a = spd(n);
    let b = rhs(n);

    let blocked = Blocked::new(&a, n, block_size, Triangle::Lower);
    // The first stored block is the diagonal tile of block column 0. Within a column-major tile,
    // (row 0, column 1) is strictly upper and sits at index `1 * block_size + 0`.
    let perturbed = blocked.perturbed(block_size, 1e7);

    for kind in kinds() {
        assert_close(
            &blocked.solve_values(&perturbed, kind, &b),
            &scalar_solve(&a, n, kind, &b),
            &format!("{kind:?}: upper half of a diagonal tile"),
        );
    }
}

/// Its mirror inside the same tile *is* read, which is what makes the test above evidence about
/// which half is selected rather than evidence that diagonal tiles are ignored wholesale.
#[test]
fn the_selected_half_of_a_diagonal_tile_is_read() {
    let (n, block_size) = (12usize, 3usize);
    let a = spd(n);
    let b = rhs(n);
    let kind = FactorizationKind::LdltSbk;

    let blocked = Blocked::new(&a, n, block_size, Triangle::Lower);
    let unperturbed = blocked.solve(kind, &b);

    // (row 1, column 0) of the same tile: strictly lower, column-major index 0 * block_size + 1.
    let perturbed = blocked.solve_values(&blocked.perturbed(1, 1e7), kind, &b);

    // The control is the *unperturbed block* solve, not the scalar oracle. Both go through the
    // block path, so a difference is caused by the changed value alone: were the block path
    // broken, it would move the unperturbed solve too and this would not register a change.
    let moved = perturbed
        .iter()
        .zip(&unperturbed)
        .any(|(g, w)| (g - w).abs() > 1e-9 * g.abs().max(1.0));
    assert!(
        moved,
        "perturbing the selected half of a diagonal tile must change the solution"
    );
}

/// An off-diagonal tile is placed as stored and reflected into the other triangle by
/// *transposition*, not verbatim. Pinned with a tile that is not symmetric, so the two readings
/// give different matrices: transposing the stored tile must change the answer.
#[test]
fn an_off_diagonal_tile_is_reflected_by_transposition() {
    // A 4x4 matrix as 2x2 blocks. The off-diagonal block is deliberately asymmetric.
    //   6 1 | 2 0
    //   1 8 | 3 1
    //   ----+----
    //   2 3 | 7 1
    //   0 1 | 1 4
    let n = 4;
    #[rustfmt::skip]
    let a = vec![
        6.0, 1.0, 2.0, 0.0,
        1.0, 8.0, 3.0, 1.0,
        2.0, 3.0, 7.0, 1.0,
        0.0, 1.0, 1.0, 4.0,
    ];
    let b = rhs(n);

    let blocked = Blocked::new(&a, n, 2, Triangle::Lower);
    // The off-diagonal tile is the second stored block, values[4..8], column-major
    // [[2,3],[0,1]] -> 2,0,3,1. Transposing it describes a different symmetric matrix.
    let mut transposed = blocked.values.clone();
    transposed[4..8].copy_from_slice(&[2.0, 3.0, 0.0, 1.0]);

    for kind in kinds() {
        let oracle = scalar_solve(&a, n, kind, &b);
        assert_close(
            &blocked.solve(kind, &b),
            &oracle,
            &format!("{kind:?}: as stored"),
        );

        let y = blocked.solve_values(&transposed, kind, &b);
        let moved = y
            .iter()
            .zip(&oracle)
            .any(|(g, w)| (g - w).abs() > 1e-9 * g.abs().max(1.0));
        assert!(
            moved,
            "{kind:?}: transposing an asymmetric off-diagonal tile must change the solution"
        );
    }
}

/// Blocks stored in the ignored triangle are dropped rather than folded into their mirror — the
/// same policy the scalar path follows, now pinned at block granularity.
#[test]
fn blocks_in_the_ignored_triangle_are_dropped() {
    let (n, block_size) = (12usize, 3usize);
    let blocks = n / block_size;
    let a = spd(n);
    let b = rhs(n);
    let oracle = scalar_solve(&a, n, FactorizationKind::Cholesky, &b);

    // Every block of the grid is stored, but only the lower triangle is declared. The strictly
    // upper blocks carry a value large enough to dominate the matrix if it were read; the rest
    // carry the true values, so a correct answer requires exactly that the junk be dropped.
    let mut full = a.clone();
    for row in 0..n {
        for column in 0..n {
            if row / block_size < column / block_size {
                full[row * n + column] = 1e7;
            }
        }
    }
    let blocked = Blocked::full(&full, n, block_size, Triangle::Lower);
    assert_eq!(
        blocked.row_indices.len(),
        blocks * blocks,
        "every block of the grid must be stored for this to test anything"
    );

    let x = blocked.solve(FactorizationKind::Cholesky, &b);
    assert_close(&x, &oracle, "junk stored in the ignored block triangle");
}

/// The refactor path reads blocks the way the factorize path does — the same tile layout and the
/// same triangle policy — through a different Accelerate entry point, the one an iterative solver
/// spends its time in. The pattern stores every block so the refactor values can carry junk in
/// the ignored triangle and its being dropped can be observed, not just the tile layout.
#[test]
fn the_refactor_path_reads_blocks_the_same_way() {
    let (n, block_size) = (12usize, 3usize);
    let a = spd(n);
    let b = rhs(n);
    let scaled: Vec<f64> = a.iter().map(|v| 2.0 * v).collect();

    // The refactor values are the scaled matrix with the strictly-upper blocks — the ones the
    // declared lower triangle ignores — overwritten with a value large enough to dominate the
    // matrix if it were read.
    let mut scaled_with_junk = scaled.clone();
    for row in 0..n {
        for column in 0..n {
            if row / block_size < column / block_size {
                scaled_with_junk[row * n + column] = 1e7;
            }
        }
    }

    // A full-grid pattern declaring the lower triangle, so the junk has somewhere to be stored.
    let base = Blocked::full(&a, n, block_size, Triangle::Lower);
    let refactor_values = Blocked::full(&scaled_with_junk, n, block_size, Triangle::Lower).values;

    for kind in kinds() {
        let oracle = scalar_solve(&scaled, n, kind, &b);

        let structure = base.structure();
        let symbolic = SymbolicFactorization::new(kind, &structure).unwrap();
        let mut factorization = symbolic.factorize(&base.values).unwrap();
        factorization.refactor(&refactor_values).unwrap();

        assert_close(
            &factorization.solve_vec(&b).unwrap(),
            &oracle,
            &format!("{kind:?}: refactor"),
        );
    }
}

/// The same equivalence as [`block_storage_agrees_with_scalar_storage`], at a size that reaches
/// Accelerate's supernodal path, where the algorithm differs from the small dense case.
#[test]
fn block_storage_agrees_at_supernodal_size() {
    let n = 400;
    let a = spd_banded(n);
    let b = rhs(n);
    for kind in kinds() {
        let oracle = scalar_solve(&a, n, kind, &b);
        for block_size in [1usize, 4, 5] {
            for triangle in [Triangle::Lower, Triangle::Upper] {
                let blocked = Blocked::new(&a, n, block_size, triangle);
                assert_close(
                    &blocked.solve(kind, &b),
                    &oracle,
                    &format!("n={n} b={block_size} {triangle:?} {kind:?}"),
                );
            }
        }
    }
}

/// `f32` reads blocks the same way, against its own scalar oracle.
#[test]
fn single_precision_reads_blocks_the_same_way() {
    let (n, block_size) = (12usize, 3usize);
    let a = spd(n);
    let b = rhs(n);

    let blocked = Blocked::new(&a, n, block_size, Triangle::Lower);
    let values: Vec<f32> = blocked.values.iter().map(|v| *v as f32).collect();
    let b32: Vec<f32> = b.iter().map(|v| *v as f32).collect();

    let structure = blocked.structure();
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    let x = symbolic
        .factorize(&values)
        .unwrap()
        .solve_vec(&b32)
        .unwrap();

    let oracle = scalar_solve(&a, n, FactorizationKind::Cholesky, &b);
    for (i, (g, w)) in x.iter().zip(&oracle).enumerate() {
        assert!(
            (f64::from(*g) - w).abs() < 1e-5,
            "component {i}: single precision {g}, double precision {w}"
        );
    }
}

/// The values slice is counted in scalars: one `block_size * block_size` tile per stored block.
#[test]
fn value_count_counts_tile_entries() {
    const CS: [i64; 3] = [0, 1, 2];
    const RI: [i32; 2] = [0, 1];
    let structure =
        SparseStructure::from_csc(2, 2, &CS, &RI, Attributes::symmetric(Triangle::Lower)).unwrap();
    assert_eq!(structure.stored_entries(), 2);
    assert_eq!(structure.value_count(), 2);

    let blocked = structure.with_block_size(3).unwrap();
    assert_eq!(blocked.stored_entries(), 2, "stored entries stay in blocks");
    assert_eq!(
        blocked.value_count(),
        2 * 9,
        "values carry a tile per block"
    );

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &blocked).unwrap();
    let _: usize = blocked.rows();
    let _: usize = blocked.columns();
    let _: usize = symbolic.rows();
    let _: usize = symbolic.columns();
    assert_eq!(symbolic.rows(), 2, "stored dimensions count blocks");
    assert_eq!(symbolic.columns(), 2, "stored dimensions count blocks");
    assert_eq!(symbolic.effective_rows(), 6);
    assert_eq!(symbolic.effective_columns(), 6);
    assert_eq!(symbolic.right_hand_side_rows(), 6);
    assert_eq!(symbolic.solution_rows(), 6);
    assert_eq!(symbolic.in_place_operand_rows(), 6);

    let mut values = vec![0.0; 18];
    for diagonal in [0, 4, 8, 9, 13, 17] {
        values[diagonal] = 1.0;
    }
    let factorization = symbolic.factorize(&values).unwrap();
    let _: usize = factorization.rows();
    let _: usize = factorization.columns();
    assert_eq!(factorization.rows(), 2);
    assert_eq!(factorization.columns(), 2);
    assert_eq!(factorization.effective_rows(), 6);
    assert_eq!(factorization.effective_columns(), 6);
    assert_eq!(factorization.right_hand_side_rows(), 6);
    assert_eq!(factorization.solution_rows(), 6);
    assert_eq!(factorization.in_place_operand_rows(), 6);
}
