use super::*;

adv_fn! {
    pub fn halfpipe(x: [[2]]) -> [[1]] {
        DVector::from_vec(vec![(x[1] * x[1] - x[0].max(Scalar::zero())).max(Scalar::zero())])
    }
}

adv_fn! {
    pub fn halfpipe_1(x: [[2]]) -> [[2]] {
        DVector::from_vec(vec![x[0].max(Scalar::zero()), x[1] * x[1]])
    }
}

adv_fn! {
    pub fn halfpipe_2(x: [[2]]) -> [[1]] {
        DVector::from_vec(vec![(x[1] - x[0]).max(Scalar::zero())])
    }
}

pub fn halfpipe_anf(x: DVector<f64>) -> AbsNormalForm {
    let zmat = DMatrix::from_vec(2, 2, vec![1.0, -0.5, 0.0, 2.0 * x[1]]);

    let lmat = DMatrix::from_vec(2, 2, vec![0.0, -0.5, 0.0, 0.0]);

    let jmat = DMatrix::from_vec(1, 2, vec![-0.25, x[1]]);

    let ymat = DMatrix::from_vec(1, 2, vec![-0.25, 0.5]);

    let z = DVector::from_vec(vec![x[0], x[1] * x[1] - x[0] / 2.0 - x[0].abs() / 2.0]);

    let a = &z - &lmat * z.abs();
    let b = -&ymat * z.abs();
    AbsNormalForm {
        a,
        zmat,
        lmat,
        b,
        jmat,
        ymat,
    }
}

pub fn halfpipe_jacobian(x: &DVector<f64>, dx: &DVector<f64>) -> GeneralizedJacobian {
    let anf = halfpipe_anf(x.clone());
    let s = 2;

    let dz0 = anf.a[0] + anf.zmat[(0, 0)] * dx[0];
    let dz1 = anf.a[1]
        + anf.zmat[(1, 0)] * dx[0]
        + anf.zmat[(1, 1)] * dx[1]
        + anf.lmat[(1, 0)] * dz0.abs();

    let mut sigma = DMatrix::zeros(s, s);
    let mut multiplicity = 0_usize;
    sigma[(0, 0)] = if dz0 < 0.0 {
        -1.0
    } else if dz0 > 0.0 {
        1.0
    } else {
        multiplicity += 1;
        -1.0
    };
    sigma[(1, 1)] = if dz1 < 0.0 {
        -1.0
    } else if dz1 > 0.0 {
        1.0
    } else {
        multiplicity += 1;
        -1.0
    };

    let amat = (DMatrix::identity(s, s) - &anf.lmat * &sigma)
        .try_inverse()
        .unwrap();
    let homogenous = &anf.jmat + &anf.ymat * &sigma * &amat * &anf.zmat;
    let inhomogenous = &anf.b + &anf.ymat * &sigma * &amat * &anf.a;

    GeneralizedJacobian {
        homogenous,
        inhomogenous,
        multiplicity,
    }
}
