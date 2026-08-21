//! This example analyses a sparsity pattern once, refactors values each iteration, and falls back
//! from a failed Cholesky to an indefinite factorization.
//!
//! Run with `cargo run --example newton_refactor`.

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use accelerate_sparse::{
        Attributes, FactorizationKind, SparseStructure, SymbolicFactorization, Triangle,
        error::Status,
    };

    const N: usize = 8;

    // A tridiagonal model problem: the 1-D Laplacian plus a shift on the diagonal, stored as its
    // lower triangle. The triplets are emitted column by column, diagonal first, so the assembled
    // values keep this order and later iterations can rewrite them without reassembling.
    let values_for = |shift: f64| -> Vec<f64> {
        let mut values = Vec::with_capacity(2 * N - 1);
        for column in 0..N {
            values.push(2.0 + shift);
            if column + 1 < N {
                values.push(-1.0);
            }
        }
        values
    };
    let mut row_indices = Vec::new();
    let mut column_indices = Vec::new();
    for column in 0..N {
        row_indices.push(column);
        column_indices.push(column);
        if column + 1 < N {
            row_indices.push(column + 1);
            column_indices.push(column);
        }
    }

    let (structure, values) = SparseStructure::from_coordinates(
        N,
        N,
        &row_indices,
        &column_indices,
        &values_for(1.0),
        Attributes::symmetric(Triangle::Lower),
    )?;

    // The analysis depends only on the pattern, so both factorization kinds are prepared once and
    // reused for every shift below.
    let cholesky = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure)?;
    let ldlt = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure)?;

    let b = vec![1.0; N];
    let mut factorization = cholesky.factorize(&values)?;

    for shift in [1.0, 0.0, -3.0] {
        let values = values_for(shift);
        match factorization.refactor(&values) {
            Ok(()) => {
                let x = factorization.solve_vec(&b)?;
                println!("shift {shift:+.1}: Cholesky, x[0] = {:.6}", x[0]);
            }
            // A failed factorization is a fact about the matrix, not a bug: with this shift it is
            // no longer positive definite. Fall back to LDL^T, which handles indefinite matrices,
            // and let its inertia report how many negative directions the iteration must treat.
            Err(error) if error.status() == Some(Status::FactorizationFailed) => {
                let fallback = ldlt.factorize(&values)?;
                let inertia = fallback.inertia()?;
                let x = fallback.solve_vec(&b)?;
                println!(
                    "shift {shift:+.1}: indefinite, LDL^T with {} negative eigenvalues, x[0] = {:.6}",
                    inertia.negative, x[0]
                );
            }
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    // The solver API exists only when targeting macOS; elsewhere the crate is an empty library.
    eprintln!("this example needs macOS and the Accelerate framework");
}
