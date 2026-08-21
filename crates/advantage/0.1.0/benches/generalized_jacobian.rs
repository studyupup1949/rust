#![feature(test)]

extern crate advantage as adv;
extern crate test;

use adv::prelude::*;
use test::Bencher;

adv_fn! {
    fn test_function(input: [[128]]) -> [[1]] {
        adv::DVector::from_element(
            1,
            input
                .map(|x| x.max(Scalar::zero()))
                .map(|x| x.min(Scalar::one()))
                .as_slice()
                .iter()
                .fold(Scalar::zero(), |a, b| a.max(*b)),
        )
    }
}

adv_fn! {
    fn propagate(input: [[128]]) -> [[128]] {
        input.map(|x| x.max(Scalar::zero()))
    }
}

#[bench]
fn generalized_jacobian(b: &mut Bencher) {
    let func = adv_fn_obj!(test_function);
    let x = adv::DVector::from_element(func.n(), 1.0);
    let dx = adv::DVector::from_element(func.n(), 1.0);

    b.iter(|| {
        let func = test::black_box(&func);
        let x = test::black_box(&x);
        let dx = test::black_box(&dx);
        adv::drivers::generalized_jacobian(func, x, dx, &[0], None)
    });
}

#[bench]
fn generalized_jacobian_chain(b: &mut Bencher) {
    let mut chain = adv::FunctionChain::new(adv_fn_obj!(propagate));
    chain.append(adv_fn_obj!(propagate));
    chain.append(adv_fn_obj!(propagate));
    chain.append(adv_fn_obj!(test_function));

    let x = adv::DVector::from_element(chain.n(), 1.0);
    let dx = adv::DVector::from_element(chain.n(), 1.0);

    b.iter(|| {
        let chain = test::black_box(&chain);
        let x = test::black_box(x.clone());
        let dx = test::black_box(dx.clone());
        adv::drivers::generalized_jacobian_chain(chain, x, dx, None)
    });
}
