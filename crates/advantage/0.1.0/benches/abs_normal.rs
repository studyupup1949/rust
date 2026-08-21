#![feature(test)]

extern crate advantage as adv;
extern crate test;

use adv::prelude::*;
use test::Bencher;

adv_fn! {
    fn test_function(input: Vec<Scalar>) -> Scalar {
        input.iter()
            .map(|x| x.max(Scalar::zero()))
            .map(|x| x.min(Scalar::one()))
            .fold(Scalar::zero(), |a, b| a.max(b))
    }
}

fn test_function_tape() -> impl adv::Tape {
    let mut ctx = adv::AContext::new();
    let input = ctx.new_indep_vec(2_usize.pow(15), 0.0);
    let output = test_function(input);
    ctx.set_dep(&output);
    ctx.tape()
}

#[bench]
fn abs_normal_mul_left(b: &mut Bencher) {
    let mut tape = test_function_tape();
    tape.zero_order(&adv::DVector::from_element(tape.num_indeps(), 1.0));
    let s = tape.num_abs();

    let tape = test::black_box(Box::new(tape));
    let abs_tape = adv::drivers::AbsNormalTape::new(tape);
    let abs_l = adv::drivers::AbsNormalL::new(&abs_tape);
    let ybar = adv::DMatrix::from_element(1, s, 1.0);

    b.iter(|| {
        let ybar = test::black_box(&ybar);
        abs_l.mul_left(ybar)
    });
}

#[bench]
fn abs_normal_mul_right(b: &mut Bencher) {
    let mut tape = test_function_tape();
    tape.zero_order(&adv::DVector::from_element(tape.num_indeps(), 1.0));
    let s = tape.num_abs();

    let tape = test::black_box(Box::new(tape));
    let abs_tape = adv::drivers::AbsNormalTape::new(tape);
    let abs_l = adv::drivers::AbsNormalL::new(&abs_tape);
    let dx = adv::DMatrix::from_element(s, 1, 1.0);

    b.iter(|| {
        let dx = test::black_box(&dx);
        abs_l.mul_right(dx)
    });
}
