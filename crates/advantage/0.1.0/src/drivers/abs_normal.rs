use super::*;
use nalgebra::{Dim, Matrix};
use rayon::prelude::*;

/// Wraps a tape containing abs-calls and decomposes it for algorithms specific to piecewise-smooth functions
#[derive(Debug)]
pub struct AbsNormalTape {
    inner: Box<dyn Tape>,
    indeps: Vec<usize>,
    deps: Vec<usize>,
    n: usize,
    m: usize,
    s: usize,
}

impl AbsNormalTape {
    pub fn new(inner: Box<dyn Tape>) -> Self {
        // Save dimensions
        let n = inner.num_indeps();
        let m = inner.num_deps();
        let s = inner.num_abs();

        // Calculate independent indices
        let indeps = {
            let x_iter = inner.indeps().iter().cloned();
            let z_iter = inner
                .ops_iter()
                .filter(|op| op.opcode == OpCode::Abs)
                .map(|op| op.vid);
            x_iter.chain(z_iter).collect()
        };

        // Calculate dependent indices
        let deps = {
            let y_iter = inner.deps().iter().cloned();
            let z_iter = inner
                .ops_iter()
                .filter(|op| op.opcode == OpCode::Abs)
                .map(|op| op.arg1.unwrap());
            z_iter.chain(y_iter).collect()
        };

        Self {
            inner,
            indeps,
            deps,
            n,
            m,
            s,
        }
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn m(&self) -> usize {
        self.m
    }

    pub fn s(&self) -> usize {
        self.s
    }

    pub fn x(&self) -> DVector<f64> {
        DVector::from_vec(
            self.inner
                .indeps()
                .iter()
                .cloned()
                .map(|id| self.values()[id])
                .collect(),
        )
    }

    pub fn y(&self) -> DVector<f64> {
        DVector::from_vec(
            self.inner
                .deps()
                .iter()
                .cloned()
                .map(|id| self.values()[id])
                .collect(),
        )
    }

    pub fn z(&self) -> DVector<f64> {
        DVector::from_vec(
            self.inner
                .ops_iter()
                .filter(|op| op.opcode == OpCode::Abs)
                .map(|op| op.arg1.unwrap())
                .map(|id| self.values()[id])
                .collect(),
        )
    }
}

impl Tape for AbsNormalTape {
    fn indeps(&self) -> &[usize] {
        &self.indeps
    }

    fn deps(&self) -> &[usize] {
        &self.deps
    }

    fn ops_iter<'a>(&'a self) -> Box<dyn DoubleEndedIterator<Item = Operation> + 'a> {
        Box::new(self.inner.ops_iter().map(|op| {
            if op.opcode == OpCode::Abs {
                Operation::nop()
            } else {
                op
            }
        }))
    }

    fn values(&self) -> &[f64] {
        self.inner.values()
    }

    fn values_mut(&mut self) -> &mut [f64] {
        self.inner.values_mut()
    }
}

macro_rules! anf_matrix {
    ($name:ident, $indep:ident, $dep:ident, $doc:expr) => {
        #[derive(Debug, Clone)]
        #[doc=$doc]
        pub struct $name<'a> {
            inner: &'a AbsNormalTape,
            indeps: Vec<usize>,
            deps: Vec<usize>,
        }

        impl<'a> $name<'a> {
            pub fn new(inner: &'a AbsNormalTape) -> Self {
                let indeps = inner.indeps().iter().cloned().$indep(inner.n()).collect();
                let deps = inner.deps().iter().cloned().$dep(inner.s()).collect();
                Self {
                    inner,
                    indeps,
                    deps,
                }
            }

            pub fn nrows(&self) -> usize {
                self.num_deps()
            }

            pub fn ncols(&self) -> usize {
                self.num_indeps()
            }

            pub fn mul_right<R: Dim, C: Dim, S>(&self, rhs: &Matrix<f64, R, C, S>) -> DMatrix<f64>
            where
                S: nalgebra::storage::Storage<f64, R, C> + Sync,
            {
                assert_eq!(self.ncols(), rhs.nrows());
                let cols = (0..rhs.ncols())
                    .into_par_iter()
                    .map(|j| {
                        let mut dx = DVector::zeros(rhs.nrows());
                        for i in 0..rhs.nrows() {
                            dx[i] = rhs[(i, j)];
                        }
                        let dy = self.first_order_forward(&dx);
                        let mut result = DMatrix::zeros(self.nrows(), 1);
                        for i in 0..dy.nrows() {
                            result[(i, 0)] = dy[i];
                        }
                        vec![(j, result)]
                    })
                    .reduce(
                        Vec::new,
                        |mut a, mut b| {
                            a.append(&mut b);
                            a
                        },
                    );
                let mut result = DMatrix::zeros(self.nrows(), rhs.ncols());
                for (j, col) in cols.iter() {
                    result.column_mut(*j).copy_from(col);
                }
                result
            }

            pub fn mul_left<R: Dim, C: Dim, S>(&self, lhs: &Matrix<f64, R, C, S>) -> DMatrix<f64>
            where
                S: nalgebra::storage::Storage<f64, R, C> + Sync,
            {
                assert_eq!(lhs.ncols(), self.nrows());
                let rows = (0..lhs.nrows())
                    .into_par_iter()
                    .map(|i| {
                        let mut ybar = DVector::zeros(lhs.ncols());
                        for j in 0..lhs.ncols() {
                            ybar[j] = lhs[(i, j)];
                        }
                        let xbar = self.first_order_reverse(&ybar);
                        let mut result = DMatrix::zeros(1, self.ncols());
                        for j in 0..xbar.nrows() {
                            result[(0, j)] = xbar[j];
                        }
                        vec![(i, result)]
                    })
                    .reduce(
                        Vec::new,
                        |mut a, mut b| {
                            a.append(&mut b);
                            a
                        },
                    );
                let mut result = DMatrix::zeros(lhs.nrows(), self.ncols());
                for (i, row) in rows.iter() {
                    result.row_mut(*i).copy_from(row);
                }
                result
            }

            pub fn row(&self, i: usize) -> DMatrix<f64> {
                let mut ind = DMatrix::zeros(1, self.num_deps());
                ind[(0, i)] = 1.0;
                self.mul_left(&ind)
            }

            pub fn column(&self, i: usize) -> DMatrix<f64> {
                let mut ind = DMatrix::zeros(1, self.num_indeps());
                ind[i] = 1.0;
                self.mul_right(&ind)
            }
        }

        impl<'b> Tape for $name<'b> {
            fn indeps(&self) -> &[usize] {
                &self.indeps
            }

            fn deps(&self) -> &[usize] {
                &self.deps
            }

            fn ops_iter<'a>(&'a self) -> Box<dyn DoubleEndedIterator<Item = Operation> + 'a> {
                self.inner.ops_iter()
            }

            fn values(&self) -> &[f64] {
                self.inner.values()
            }

            fn values_mut(&mut self) -> &mut [f64] {
                panic!("{}::values_mut not supported", stringify!($name));
            }
        }
    };
}

anf_matrix!(
    AbsNormalZ,
    take,
    take,
    "Representation of the Z matrix in an Abs-Normal Form"
);
anf_matrix!(
    AbsNormalL,
    skip,
    take,
    "Representation of the L matrix in an Abs-Normal Form"
);
anf_matrix!(
    AbsNormalJ,
    take,
    skip,
    "Representation of the J matrix in an Abs-Normal Form"
);
anf_matrix!(
    AbsNormalY,
    skip,
    skip,
    "Representation of the Y matrix in an Abs-Normal Form"
);

/// Dense representation of an Abs-Normal Form
#[derive(Debug, Clone, PartialEq)]
pub struct AbsNormalForm {
    pub a: DVector<f64>,
    pub zmat: DMatrix<f64>,
    pub lmat: DMatrix<f64>,
    pub b: DVector<f64>,
    pub jmat: DMatrix<f64>,
    pub ymat: DMatrix<f64>,
}

/// Derive a dense Abs-Normal form from a function
#[allow(clippy::many_single_char_names)]
pub fn abs_normal(func: &dyn Function, x: &DVector<f64>) -> AbsNormalForm {
    let tape = func.tape(x);
    let abs_tape = AbsNormalTape::new(tape);
    let n = abs_tape.n();
    let m = abs_tape.m();
    let s = abs_tape.s();

    let z_tape = AbsNormalZ::new(&abs_tape);
    let l_tape = AbsNormalL::new(&abs_tape);
    let j_tape = AbsNormalJ::new(&abs_tape);
    let y_tape = AbsNormalY::new(&abs_tape);

    let zmat = if n < s {
        z_tape.mul_right(&DMatrix::identity(n, n))
    } else {
        z_tape.mul_left(&DMatrix::identity(s, s))
    };

    let lmat = l_tape.mul_left(&DMatrix::identity(s, s));

    let jmat = if n < m {
        j_tape.mul_right(&DMatrix::identity(n, n))
    } else {
        j_tape.mul_left(&DMatrix::identity(m, m))
    };

    let ymat = if s < m {
        y_tape.mul_right(&DMatrix::identity(s, s))
    } else {
        y_tape.mul_left(&DMatrix::identity(m, m))
    };

    let z = abs_tape.z();
    let z_abs = z.abs();
    let a = &z - &lmat * &z_abs;
    let b = -&ymat * &z_abs;

    AbsNormalForm {
        a,
        zmat,
        lmat,
        b,
        jmat,
        ymat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::traits::Zero;

    #[test]
    fn abs_normal_matrices() {
        let x = DVector::from_vec(vec![1.0, 2.0]);
        let tape = adv_fn_obj!(halfpipe).tape(&x);
        let abs_ref = halfpipe_anf(x.clone());

        let abs_tape = AbsNormalTape::new(tape);
        let zmat_tape = AbsNormalZ::new(&abs_tape);
        let lmat_tape = AbsNormalL::new(&abs_tape);
        let jmat_tape = AbsNormalJ::new(&abs_tape);
        let ymat_tape = AbsNormalY::new(&abs_tape);

        assert_eq!(zmat_tape.mul_right(&DMatrix::identity(2, 2)), abs_ref.zmat);
        assert_eq!(lmat_tape.mul_right(&DMatrix::identity(2, 2)), abs_ref.lmat);
        assert_eq!(jmat_tape.mul_right(&DMatrix::identity(2, 2)), abs_ref.jmat);
        assert_eq!(ymat_tape.mul_right(&DMatrix::identity(2, 2)), abs_ref.ymat);

        assert_eq!(zmat_tape.mul_left(&DMatrix::identity(2, 2)), abs_ref.zmat);
        assert_eq!(lmat_tape.mul_left(&DMatrix::identity(2, 2)), abs_ref.lmat);
        assert_eq!(jmat_tape.mul_left(&DMatrix::identity(1, 1)), abs_ref.jmat);
        assert_eq!(ymat_tape.mul_left(&DMatrix::identity(1, 1)), abs_ref.ymat);

        let z = abs_tape.z();
        let a = &z - lmat_tape.mul_right(&z.abs());
        assert_eq!(a, abs_ref.a);

        let b = -ymat_tape.mul_right(&z.abs()).column(0).into_owned();
        assert_eq!(b, abs_ref.b);
    }

    #[test]
    fn halfpipe_function() {
        let func = adv_fn_obj!(halfpipe);

        for x1 in (0..10).map(|i| (i as f64) * 0.5) {
            for x2 in (0..10).map(|i| (i as f64) * 0.5) {
                let x = DVector::from_vec(vec![x1, x2]);
                let anf_ref = halfpipe_anf(x.clone());
                let anf = abs_normal(&func, &x);
                assert_eq!(anf, anf_ref);
            }
        }
    }

    adv_fn! {
        fn consistency_test_func(input: [[2]]) -> [[1]] {
            let x1 = input[0];
            let x2 = input[1];
            let y = x1.abs() + x2;
            DVector::from_vec(vec![y])
        }
    }

    #[test]
    fn anf_consistency() {
        let anf = abs_normal(
            &adv_fn_obj!(consistency_test_func),
            &DVector::from_vec(vec![2.0, 3.0]),
        );

        assert_eq!(anf.a.nrows(), 1);
        assert!((anf.a[0] - 2.0).abs() < std::f64::EPSILON);

        assert_eq!(anf.zmat.nrows(), 1);
        assert_eq!(anf.zmat.ncols(), 2);
        assert!((anf.zmat[(0, 0)] - 1.0).abs() < std::f64::EPSILON);
        assert!((anf.zmat[(0, 1)] - 0.0).abs() < std::f64::EPSILON);

        assert_eq!(anf.lmat.nrows(), 1);
        assert_eq!(anf.lmat.ncols(), 1);
        assert!((anf.lmat[(0, 0)] - 0.0).abs() < std::f64::EPSILON);

        assert_eq!(anf.b.nrows(), 1);
        assert!((anf.b[0] + 2.0).abs() < std::f64::EPSILON);

        assert_eq!(anf.jmat.nrows(), 1);
        assert_eq!(anf.jmat.ncols(), 2);
        assert!((anf.jmat[(0, 0)] - 0.0).abs() < std::f64::EPSILON);
        assert!((anf.jmat[(0, 1)] - 1.0).abs() < std::f64::EPSILON);

        assert_eq!(anf.ymat.nrows(), 1);
        assert_eq!(anf.ymat.ncols(), 1);
        assert!((anf.ymat[(0, 0)] - 1.0).abs() < std::f64::EPSILON);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn abs_decompose() {
        let tape = {
            let mut ctx = AContext::new();
            let mut a = ADouble::zero();
            let mut b = ADouble::zero();
            ctx.set_indep(&mut a);
            ctx.set_indep(&mut b);
            // ||sin(a)| + cos(b)|
            let c = (a.sin().abs() + b.cos()).abs();
            ctx.set_dep(&c);
            ctx.tape()
        };
        let mut indep_iter = tape.indeps().iter().cloned();
        assert_eq!(indep_iter.next().unwrap(), 0);
        assert_eq!(indep_iter.next().unwrap(), 1);
        assert_eq!(indep_iter.next(), None);
        std::mem::drop(indep_iter);

        let mut dep_iter = tape.deps().iter().cloned();
        assert_eq!(dep_iter.next().unwrap(), 6);
        assert_eq!(dep_iter.next(), None);
        std::mem::drop(dep_iter);

        let mut ops_iter = tape.ops_iter();
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Sin);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Abs);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Cos);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Add);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Abs);
        assert_eq!(ops_iter.next(), None);
        std::mem::drop(ops_iter);

        let abs_tape = AbsNormalTape::new(Box::new(tape));
        let z_tape = AbsNormalZ::new(&abs_tape);
        let l_tape = AbsNormalL::new(&abs_tape);
        let j_tape = AbsNormalJ::new(&abs_tape);
        let y_tape = AbsNormalY::new(&abs_tape);
        assert_eq!(z_tape.indeps().to_vec(), vec![0, 1]);
        assert_eq!(l_tape.indeps().to_vec(), vec![3, 6]);
        assert_eq!(j_tape.indeps().to_vec(), vec![0, 1]);
        assert_eq!(y_tape.indeps().to_vec(), vec![3, 6]);

        assert_eq!(z_tape.deps().to_vec(), vec![2, 5]);
        assert_eq!(l_tape.deps().to_vec(), vec![2, 5]);
        assert_eq!(j_tape.deps().to_vec(), vec![6]);
        assert_eq!(y_tape.deps().to_vec(), vec![6]);

        let mut ops_iter = abs_tape.ops_iter();
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Sin);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Nop);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Cos);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Add);
        assert_eq!(ops_iter.next().unwrap().opcode, OpCode::Nop);
        assert_eq!(ops_iter.next(), None);
    }
}
