use super::*;
use std::fmt;
use std::iter::{DoubleEndedIterator, Iterator};

/// Evaluation procedure and intermediate values of a function evaluation
pub trait Tape: Send + Sync + fmt::Debug {
    /// Independent variable indices
    fn indeps(&self) -> &[usize];
    /// Dependent variable indices
    fn deps(&self) -> &[usize];
    /// Get intermediate values
    fn values(&self) -> &[f64];
    /// Get intermediate values (mutable)
    fn values_mut(&mut self) -> &mut [f64];
    /// Iterate through operations
    fn ops_iter<'a>(&'a self) -> Box<dyn DoubleEndedIterator<Item = Operation> + 'a>;

    /// Number of independents
    fn num_indeps(&self) -> usize {
        self.indeps().len()
    }

    /// Number of dependents
    fn num_deps(&self) -> usize {
        self.deps().len()
    }

    /// Number of abs-function operations
    fn num_abs(&self) -> usize {
        self.ops_iter()
            .filter(|op| op.opcode == OpCode::Abs)
            .count()
    }

    /// Maximum value idx on this tape
    fn max_id(&self) -> usize {
        let indep_max = self.indeps().iter().cloned().max().unwrap_or(0);
        let dep_max = self.deps().iter().cloned().max().unwrap_or(0);
        let op_max = self.ops_iter().map(|op| op.vid).max().unwrap_or(0);
        indep_max.max(dep_max).max(op_max)
    }

    /// Stored arguments to the function
    fn x(&self) -> DVector<f64> {
        let mut x = DVector::zeros(self.num_indeps());
        for (idx, vid) in self.indeps().iter().enumerate() {
            x[idx] = self.values()[*vid];
        }
        x
    }

    /// Stored result of the function
    fn y(&self) -> DVector<f64> {
        let mut y = DVector::zeros(self.num_deps());
        for (idx, vid) in self.deps().iter().enumerate() {
            y[idx] = self.values()[*vid];
        }
        y
    }

    /// Re-evaluate function from stored evaluation procedure
    fn zero_order(&mut self, x: &DVector<f64>) {
        assert_eq!(x.nrows(), self.num_indeps());
        let indeps = self.indeps().to_vec();
        let ops = self.ops_iter().collect::<Vec<_>>();
        let values = self.values_mut();
        for (idx, vid) in indeps.into_iter().enumerate() {
            values[vid] = x[idx];
        }
        for op in ops.into_iter() {
            op.zero_order(values);
        }
    }

    /// Calculate adjoint of Jacobian
    fn first_order_forward(&self, dx: &DVector<f64>) -> DVector<f64> {
        let v = self.values();
        let mut dv = vec![0.0; v.len()];
        for (idx, vid) in self.indeps().iter().enumerate() {
            dv[*vid] = dx[idx];
        }
        for op in self.ops_iter() {
            op.first_order(v, &mut dv);
        }
        let mut dy = DVector::zeros(self.num_deps());
        for (idx, vid) in self.deps().iter().enumerate() {
            dy[idx] = dv[*vid];
        }
        dy
    }

    /// Calculate reverse-adjoint of Jacobian
    fn first_order_reverse(&self, ybar: &DVector<f64>) -> DVector<f64> {
        let v = self.values();
        let mut vbar = vec![0.0; v.len()];
        for (idx, vid) in self.deps().iter().enumerate() {
            vbar[*vid] = ybar[idx];
        }
        for op in self.ops_iter().rev() {
            op.first_order_reverse(v, &mut vbar);
        }
        let mut xbar = DVector::zeros(self.num_indeps());
        for (idx, vid) in self.indeps().iter().enumerate() {
            xbar[idx] = vbar[*vid];
        }
        xbar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_order_arithmetic() {
        adv_fn! {
            fn arithmetic_test_function(x1: Scalar, x2: Scalar) -> Scalar {
                let v1 = x1 + 2.0;
                let v2 = x1 - 2.0;
                let v3 = x1 * 2.0;
                let v4 = x1 / 2.0;

                let v5 = 2.0 + x2;
                let v6 = 2.0 - x2;
                let v7 = 2.0 * x2;
                let v8 = 2.0 / x2;

                let v9 = v1 + v5;
                let v10 = v2 - v6;
                let v11 = v3 * v7;
                let v12 = v4 / v8;

                v9 + v10 + v11 + v12
            }
        }

        let mut tape = {
            let mut ctx = AContext::new();
            let x1 = ctx.new_indep(0.0);
            let x2 = ctx.new_indep(0.0);
            ctx.set_dep(&arithmetic_test_function(x1, x2));
            ctx.tape()
        };
        tape.zero_order(&DVector::from_vec(vec![2.0, 3.0]));

        let expected = arithmetic_test_function(2.0, 3.0);
        let actual = tape.y()[0];
        assert!((actual - expected).abs() < std::f64::EPSILON);
    }

    #[test]
    #[allow(clippy::let_and_return)]
    fn first_order_forward_arithmetic() {
        adv_fn! {
            fn test_function(x: Scalar) -> Scalar {
                let v1 = x + x - x * x / x;
                let v2 = (v1 + 2.0 - 2.0) * 2.0 / 2.0;
                let v3 = -(2.0 - (2.0 + v2));
                let v4 = 2.0 / (2.0 * v3);
                let v5 = 1.0 / v4;
                v5
            }
        }
        let tape = {
            let mut ctx = AContext::new();
            let x = ctx.new_indep(3.0);
            ctx.set_dep(&test_function(x));
            ctx.tape()
        };
        let dy = tape.first_order_forward(&DVector::from_element(1, 1.0));
        assert!((dy[0] - 1.0).abs() < std::f64::EPSILON);
    }

    #[test]
    #[allow(clippy::let_and_return)]
    fn first_order_reverse_arithmetic() {
        adv_fn! {
            fn test_function(x: Scalar) -> Scalar {
                let v1 = x + x - x * x / x;
                let v2 = (v1 + 2.0 - 2.0) * 2.0 / 2.0;
                let v3 = -(2.0 - (2.0 + v2));
                let v4 = 2.0 / (2.0 * v3);
                let v5 = 1.0 / v4;
                v5
            }
        }
        let tape = {
            let mut ctx = AContext::new();
            let x = ctx.new_indep(3.0);
            ctx.set_dep(&test_function(x));
            ctx.tape()
        };
        let xbar = tape.first_order_reverse(&DVector::from_element(1, 1.0));
        assert!((xbar[0] - 1.0).abs() < std::f64::EPSILON);
    }

    #[test]
    #[allow(clippy::redundant_closure_call)]
    fn first_order_forward_nonlinear_functions() {
        const EPS: f64 = 1e-5;
        macro_rules! test_case {
            ($func:ident, $dy:expr) => {
                let x = 0.5;
                let tape = {
                    let mut ctx = AContext::new();
                    let mut x = AFloat::<f64>::new(x, 0.0);
                    ctx.set_indep(&mut x);
                    let y = x.$func();
                    ctx.set_dep(&y);
                    ctx.tape()
                };
                let dx = DVector::from_element(1, 1.0);
                let dy = tape.first_order_forward(&dx);
                assert!((dy[0] - ($dy)(x)).abs() < EPS);
            };
        }
        test_case!(sin, |x: f64| x.cos());
        test_case!(cos, |x: f64| -x.sin());
        test_case!(tan, |x: f64| (1.0 / x.cos()).powi(2));
        test_case!(exp, |x: f64| x.exp());
        test_case!(ln, |x: f64| 1.0 / x);
        test_case!(sqrt, |x: f64| 0.5 / x.sqrt());
        test_case!(asin, |x: f64| 1.0 / (1.0 - x.powi(2)).sqrt());
        test_case!(acos, |x: f64| -1.0 / (1.0 - x.powi(2)).sqrt());
        test_case!(atan, |x: f64| 1.0 / (1.0 + x.powi(2)));
    }

    #[test]
    #[allow(clippy::redundant_closure_call)]
    fn first_order_reverse_nonlinear_functions() {
        const EPS: f64 = 1e-5;
        macro_rules! test_case {
            ($func:ident, $dy:expr) => {
                let x = 0.5;
                let tape = {
                    let mut ctx = AContext::new();
                    let mut x = AFloat::<f64>::new(x, 0.0);
                    ctx.set_indep(&mut x);
                    let y = x.$func();
                    ctx.set_dep(&y);
                    ctx.tape()
                };
                let ybar = DVector::from_element(1, 1.0);
                let xbar = tape.first_order_reverse(&ybar);
                assert!((xbar[0] - ($dy)(x)).abs() < EPS);
            };
        }
        test_case!(sin, |x: f64| x.cos());
        test_case!(cos, |x: f64| -x.sin());
        test_case!(tan, |x: f64| (1.0 / x.cos()).powi(2));
        test_case!(exp, |x: f64| x.exp());
        test_case!(ln, |x: f64| 1.0 / x);
        test_case!(sqrt, |x: f64| 0.5 / x.sqrt());
        test_case!(asin, |x: f64| 1.0 / (1.0 - x.powi(2)).sqrt());
        test_case!(acos, |x: f64| -1.0 / (1.0 - x.powi(2)).sqrt());
        test_case!(atan, |x: f64| 1.0 / (1.0 + x.powi(2)));
    }
}
