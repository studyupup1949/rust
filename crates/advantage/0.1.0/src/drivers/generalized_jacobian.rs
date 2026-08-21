use super::*;
use std::iter::Iterator;

#[derive(Debug, Clone, PartialEq)]
pub struct GeneralizedJacobian {
    pub homogenous: DMatrix<f64>,
    pub inhomogenous: DVector<f64>,
    pub multiplicity: usize,
}

fn bit_iter<'a>(bytes: &'a [u8]) -> impl Iterator<Item = bool> + 'a {
    let mut idx = 0;
    std::iter::from_fn(move || {
        let byte_idx = idx / 8;
        if byte_idx < bytes.len() {
            let bit_idx = idx % 8;
            let byte = bytes[byte_idx];
            idx += 1;
            Some(byte & (1 << bit_idx) > 0)
        } else {
            Some(false)
        }
    })
}

/// Derive the Generalized Jacobian of a function
pub fn generalized_jacobian(
    func: &dyn Function,
    x: &DVector<f64>,
    dx: &DVector<f64>,
    sign_bits: &[u8],
    next: Option<GeneralizedJacobian>,
) -> GeneralizedJacobian {
    assert_eq!(x.nrows(), func.n());
    let tape = func.tape(x);
    generalized_jacobian_tape(tape, dx, sign_bits, next)
}

/// Derive the Generalized Jacobian of a tape
#[allow(clippy::many_single_char_names)]
pub fn generalized_jacobian_tape(
    tape: Box<dyn Tape>,
    dx: &DVector<f64>,
    sign_bits: &[u8],
    next: Option<GeneralizedJacobian>,
) -> GeneralizedJacobian {
    let n = tape.num_indeps();
    let m = tape.num_deps();
    let s = tape.num_abs();
    assert_eq!(dx.nrows(), n);

    let abs_tape = AbsNormalTape::new(tape);

    // Create subtapes
    let z_tape = AbsNormalZ::new(&abs_tape);
    let l_tape = AbsNormalL::new(&abs_tape);
    let j_tape = AbsNormalJ::new(&abs_tape);
    let y_tape = AbsNormalY::new(&abs_tape);

    // Calculate a and b
    let z = abs_tape.z();
    let z_abs = z.abs();
    let a = &z - l_tape.mul_right(&z_abs);
    let b = -y_tape.mul_right(&z_abs);

    // Calculate Δz
    let dzt = &a + z_tape.mul_right(&dx);
    let mut dz = dzt.clone();
    for _ in 0..s {
        let dz_ = dz.clone();
        dz = &dzt + l_tape.mul_right(&dz.abs());
        if dz == dz_ {
            break;
        }
    }

    // Calculate σ and multiplicity
    let mut sigma = DVector::zeros(s);
    let mut multiplicity = 0_usize;
    let mut signs = bit_iter(sign_bits);
    for i in 0..s {
        if dz[i] < 0.0 {
            sigma[i] = -1.0;
        } else if dz[i] > 0.0 {
            sigma[i] = 1.0;
        } else {
            multiplicity += 1;
            sigma[i] = if signs.next().unwrap() { 1.0 } else { -1.0 };
        }
    }

    let (g2, gamma2, multiplicity) = if let Some(next) = next {
        (
            next.homogenous,
            next.inhomogenous,
            multiplicity + next.multiplicity,
        )
    } else {
        (DMatrix::identity(m, m), DVector::zeros(m), multiplicity)
    };

    // Calcuta YΣA
    let g2ysamat = {
        // YΣ
        let mut g2_ymat_sigma = y_tape.mul_left(&g2);
        for i in 0..g2.nrows() {
            for j in 0..s {
                g2_ymat_sigma[(i, j)] *= sigma[j];
            }
        }
        let g2_ymat_sigma = g2_ymat_sigma;
        // Initially u1 = YΣ
        let mut u1 = g2_ymat_sigma.clone();
        // Copy of u1 for comparison
        let mut u2 = u1.clone();
        // Main iteration
        for _ in 0..s {
            // u1 = u1*L = u2*L
            u1 = l_tape.mul_left(&u1);
            // u1 = u1*Σ = u2*LΣ
            for i in 0..g2.nrows() {
                for j in 0..s {
                    u1[(i, j)] *= sigma[j];
                }
            }
            // u1 = u1+YΣ = u2*LΣ + YΣ
            u1 += &g2_ymat_sigma;
            // No change -> terminate iteration early
            if u1 == u2 {
                break;
            }
            // Set u2 = u1 for next comparison
            u2 = u1.clone();
        }
        // This is our result
        u1
    };

    let homogenous = j_tape.mul_left(&g2) + z_tape.mul_left(&g2ysamat);
    let inhomogenous = gamma2 + &g2 * b + g2ysamat * a;
    GeneralizedJacobian {
        homogenous,
        inhomogenous,
        multiplicity,
    }
}

/// Derive the Generalized Jacobian of a chain of functions
pub fn generalized_jacobian_chain(
    chain: &FunctionChain,
    x: DVector<f64>,
    dx: DVector<f64>,
    ncheckpoints: Option<usize>,
) -> GeneralizedJacobian {
    let ncheckpoints = ncheckpoints.unwrap_or_else(|| chain.len());
    reverse_sequence(
        (0, x, dx),
        chain.len(),
        ncheckpoints,
        |(idx, x, dx)| {
            let input = DVector::from_vec(
                x.as_slice()
                    .iter()
                    .zip(dx.as_slice().iter())
                    .map(|(x, dx)| ADouble::new(*x, *dx))
                    .collect::<Vec<_>>(),
            );
            let func = chain.nth(idx);
            let output = func.eval(input);
            let (y, dy): (Vec<f64>, Vec<f64>) =
                output.into_iter().map(|y| (y.value(), y.dvalue())).unzip();
            let y = DVector::from_vec(y);
            let dy = DVector::from_vec(dy);
            (idx + 1, y, dy)
        },
        |(idx, x, dx), g2: GeneralizedJacobian| {
            let func = chain.nth(idx);
            generalized_jacobian(func, &x, &dx, &[0], Some(g2))
        },
        |(_idx, x, _dx)| GeneralizedJacobian {
            homogenous: DMatrix::identity(x.nrows(), x.nrows()),
            inhomogenous: DVector::zeros(x.nrows()),
            multiplicity: 0,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_iter() {
        let bits = vec![0b1000_0011, 0b1100_0111];
        let mut iter = bit_iter(&bits);

        assert_eq!(iter.next().unwrap(), true);
        assert_eq!(iter.next().unwrap(), true);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), true);

        assert_eq!(iter.next().unwrap(), true);
        assert_eq!(iter.next().unwrap(), true);
        assert_eq!(iter.next().unwrap(), true);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), false);
        assert_eq!(iter.next().unwrap(), true);
        assert_eq!(iter.next().unwrap(), true);
    }

    #[test]
    fn halfpipe_function_tape() {
        let func = adv_fn_obj!(halfpipe);

        for x1 in (0..10).map(|i| (i as f64) * 0.5) {
            for x2 in (0..10).map(|i| (i as f64) * 0.5) {
                for dx1 in (0..2).map(|i| (i as f64) * 0.5) {
                    for dx2 in (0..2).map(|i| (i as f64) * 0.5) {
                        let x = DVector::from_vec(vec![x1, x2]);
                        let dx = DVector::from_vec(vec![dx1, dx2]);
                        let jac_ref = halfpipe_jacobian(&x, &dx);
                        let jac = generalized_jacobian(&func, &x, &dx, &[0], None);
                        assert_eq!(jac, jac_ref);
                    }
                }
            }
        }
    }

    #[test]
    fn halfpipe_function_with_next() {
        let func = adv_fn_obj!(halfpipe);

        for x1 in (0..10).map(|i| (i as f64) * 0.5) {
            for x2 in (0..10).map(|i| (i as f64) * 0.5) {
                for dx1 in (0..2).map(|i| (i as f64) * 0.5) {
                    for dx2 in (0..2).map(|i| (i as f64) * 0.5) {
                        let next = GeneralizedJacobian {
                            homogenous: DMatrix::identity(1, 1),
                            inhomogenous: DVector::zeros(1),
                            multiplicity: 0,
                        };
                        let x = DVector::from_vec(vec![x1, x2]);
                        let dx = DVector::from_vec(vec![dx1, dx2]);
                        let jac_ref = halfpipe_jacobian(&x, &dx);
                        let jac = generalized_jacobian(&func, &x, &dx, &[0], Some(next));
                        assert_eq!(jac, jac_ref);
                    }
                }
            }
        }
    }

    #[test]
    fn halfpipe_function_chain() {
        let mut chain = FunctionChain::new(adv_fn_obj!(halfpipe_1));
        chain.append(adv_fn_obj!(halfpipe_2));

        for x1 in (0..10).map(|i| (i as f64) * 0.5) {
            for x2 in (0..10).map(|i| (i as f64) * 0.5) {
                for dx1 in (0..2).map(|i| (i as f64) * 0.5) {
                    for dx2 in (0..2).map(|i| (i as f64) * 0.5) {
                        let x = DVector::from_vec(vec![x1, x2]);
                        let dx = DVector::from_vec(vec![dx1, dx2]);
                        let val_ref = halfpipe(x.clone());
                        let val = halfpipe_2(halfpipe_1(x.clone()));
                        let jac_ref = halfpipe_jacobian(&x, &dx);
                        let jac = generalized_jacobian_chain(&chain, x.clone(), dx.clone(), None);
                        assert_eq!(val, val_ref);
                        assert_eq!(jac, jac_ref);
                    }
                }
            }
        }
    }
}
