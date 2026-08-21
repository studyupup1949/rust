//! This example solves least squares by QR against the full right-hand side and by Cholesky of the
//! normal equations against the caller-formed `Aᵀb`.
//!
//! Run with `cargo run --example least_squares`.

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use accelerate_sparse::{
        Attributes, FactorizationKind, SparseStructure, SymbolicFactorization,
    };

    // A tall 4-by-2 design matrix for a straight-line fit, in compressed-column form:
    //     A = [[1, 0], [1, 1], [1, 2], [1, 3]]
    let column_starts = [0i64, 4, 7];
    let row_indices = [0i32, 1, 2, 3, 1, 2, 3];
    let values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0];
    let structure =
        SparseStructure::from_csc(4, 2, &column_starts, &row_indices, Attributes::ordinary())?;

    let b = [1.0, 2.9, 5.1, 7.0];

    // QR takes b itself: the right-hand side has one row per matrix row.
    let qr = SymbolicFactorization::new(FactorizationKind::Qr, &structure)?;
    let x_qr = qr.factorize(&values)?.solve_vec(&b)?;

    // CholeskyAtA takes the reduced right-hand side A^T b, one row per matrix column, and nothing
    // in the crate forms it — the caller does. Passing b here would be rejected for its length on
    // this tall matrix, but on a square matrix it would be accepted and silently solve the wrong
    // system.
    let mut atb = vec![0.0; 2];
    for (column, atb_entry) in atb.iter_mut().enumerate() {
        for position in column_starts[column] as usize..column_starts[column + 1] as usize {
            *atb_entry += values[position] * b[row_indices[position] as usize];
        }
    }

    let ata = SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &structure)?;
    let x_ata = ata.factorize(&values)?.solve_vec(&atb)?;

    println!(
        "QR:          intercept {:+.6}, slope {:+.6}",
        x_qr[0], x_qr[1]
    );
    println!(
        "CholeskyAtA: intercept {:+.6}, slope {:+.6}",
        x_ata[0], x_ata[1]
    );
    assert!(
        x_qr.iter()
            .zip(&x_ata)
            .all(|(qr, ata)| (qr - ata).abs() < 1e-10),
        "the two methods disagree beyond round-off"
    );

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    // The solver API exists only when targeting macOS; elsewhere the crate is an empty library.
    eprintln!("this example needs macOS and the Accelerate framework");
}
