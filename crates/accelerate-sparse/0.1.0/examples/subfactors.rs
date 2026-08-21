//! This example reconstructs `A·x` from `P L Lᵀ Pᵀ` and reproduces a full solve from the `PLPS`
//! half-solve.
//!
//! Run with `cargo run --example subfactors`.

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use accelerate_sparse::{
        Attributes, FactorizationKind, SparseStructure, SubfactorKind, SymbolicFactorization,
        Triangle,
    };

    // A = [[4, 1, 0], [1, 3, 1], [0, 1, 2]], stored as its lower triangle.
    let column_starts = [0i64, 2, 4, 5];
    let row_indices = [0i32, 1, 1, 2, 2];
    let values = [4.0f64, 1.0, 3.0, 1.0, 2.0];
    let structure = SparseStructure::from_csc(
        3,
        3,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )?;

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure)?;
    let factorization = symbolic.factorize(&values)?;

    // The pieces are operators, not matrices: they apply to vectors but cannot be read entry by
    // entry. Accelerate's Cholesky is A = P L Lᵀ Pᵀ, so reconstructing A·x takes the permutation
    // as well as the factor.
    let p = factorization.subfactor(SubfactorKind::P)?;
    let l = factorization.subfactor(SubfactorKind::L)?;

    let x = [1.0, 2.0, 3.0];
    let ax = p.multiply_vec(
        &l.multiply_vec(
            &l.transpose()?
                .multiply_vec(&p.transpose()?.multiply_vec(&x)?)?,
        )?,
    )?;
    // A·[1, 2, 3] against the dense multiplication: [6, 10, 8].
    for (computed, expected) in ax.iter().zip([6.0, 10.0, 8.0]) {
        assert!((computed - expected).abs() < 1e-12, "got {computed}");
    }
    println!("P L L^T P^T x = {ax:.4?}");

    // PLPS bundles half of the symmetric solve. Solving with it and then with its transpose
    // reproduces the factorization's own solve, exposing the intermediate vector between the two
    // triangular sweeps.
    let b = [1.0, 2.0, 3.0];
    let plps = factorization.subfactor(SubfactorKind::Plps)?;
    let halfway = plps.solve_vec(&b)?;
    let x_by_halves = plps.transpose()?.solve_vec(&halfway)?;
    let x_direct = factorization.solve_vec(&b)?;
    for (half, direct) in x_by_halves.iter().zip(&x_direct) {
        assert!((half - direct).abs() < 1e-12);
    }
    println!("half-solve intermediate = {halfway:.4?}");
    println!("solution                = {x_by_halves:.4?}");

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    // The solver API exists only when targeting macOS; elsewhere the crate is an empty library.
    eprintln!("this example needs macOS and the Accelerate framework");
}
