use super::*;
use rayon::prelude::*;

/// Create a jacobian using forward-mode automatic differentiation
#[allow(clippy::many_single_char_names)]
pub fn jacobian_forward(func: &dyn Function, x: &DVector<f64>) -> DMatrix<f64> {
    let n = func.n();
    let m = func.m();
    assert_eq!(x.nrows(), func.n());

    let columns = (0..n)
        .collect::<Vec<usize>>()
        .par_iter()
        .map(|j| {
            let mut dx = DVector::from_element(n, 0.0);
            dx[*j] = 1.0;
            let input = DVector::from_vec(
                x.as_slice()
                    .iter()
                    .cloned()
                    .zip(dx.into_iter())
                    .map(|(x, dx)| ADouble::new(x, *dx))
                    .collect(),
            );
            let output = func.eval(input);
            let dy = DVector::from_vec(output.as_slice().iter().map(|y| y.dvalue()).collect());

            (*j, dy)
        })
        .map(|x| vec![x])
        .reduce(Vec::new, |mut a, mut b| {
            a.append(&mut b);
            a
        });

    let mut jacobian = DMatrix::from_element(m, n, 0.0);
    for (j, dy) in columns {
        for i in 0..m {
            jacobian[(i, j)] = dy[i];
        }
    }

    jacobian
}

/// Create a jacobian using reverse-mode automatic differentiation
#[allow(clippy::many_single_char_names)]
pub fn jacobian_reverse(tape: &dyn Tape) -> DMatrix<f64> {
    let n = tape.num_indeps();
    let m = tape.num_deps();

    let rows = (0..m)
        .collect::<Vec<usize>>()
        .par_iter()
        .map(|i| {
            let mut ybar = DVector::from_element(m, 0.0);
            ybar[*i] = 1.0;

            let xbar = tape.first_order_reverse(&ybar);
            (*i, xbar)
        })
        .map(|x| vec![x])
        .reduce(Vec::new, |mut a, mut b| {
            a.append(&mut b);
            a
        });

    let mut jacobian = DMatrix::from_element(m, n, 0.0);
    for (i, xbar) in rows {
        for j in 0..n {
            jacobian[(i, j)] = xbar[j];
        }
    }

    jacobian
}

#[cfg(test)]
mod tests {
    use super::*;

    adv_fn! {
        fn test_function(input: [[2]]) -> [[2]] {
            let r = input[0];
            let phi = input[1];
            let mut output = DVector::zeros(2);
            output[0] = r * phi.cos();
            output[1] = r * phi.sin();
            output
        }
    }

    fn test_function_tape() -> impl Tape {
        let mut ctx = AContext::new();
        let input = DVector::from_vec(ctx.new_indep_vec(2, 0.0));
        let output = test_function(input);
        ctx.set_dep_slice(output.as_slice());
        ctx.tape()
    }

    fn reference_jacobian(polar: &DVector<f64>) -> DMatrix<f64> {
        let r = polar[0];
        let phi = polar[1];
        let mut result = DMatrix::from_element(2, 2, 0.0);
        result[(0, 0)] = phi.cos();
        result[(0, 1)] = -r * phi.sin();
        result[(1, 0)] = phi.sin();
        result[(1, 1)] = r * phi.cos();
        result
    }

    #[test]
    fn jacobian_forward_polar() {
        let mut polar = DVector::from_element(2, 0.0);
        polar[0] = 2.0;
        polar[1] = std::f64::consts::PI;

        let reference = reference_jacobian(&polar);
        let jacobian = jacobian_forward(&adv_fn_obj!(test_function), &polar);

        assert_eq!(jacobian, reference);
    }

    #[test]
    fn jacobian_reverse_polar() {
        let mut tape = test_function_tape();

        let mut polar = DVector::from_element(2, 0.0);
        polar[0] = 2.0;
        polar[1] = std::f64::consts::PI;
        tape.zero_order(&polar);

        let reference = reference_jacobian(&polar);
        let jacobian = jacobian_reverse(&tape);

        assert_eq!(jacobian, reference);
    }
}
