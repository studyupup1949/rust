use assertables::assert_in_delta;
use num::Rational32;
use crate::{
    mapping::IndexedMapping,
    sequence::{factory as sequence_factory, Sequence},
};
use super::{factory, PowerSeries};


#[test]
fn geometric() {
    // float 0.5
    let t = factory::geometric::<f64>().eval_finite(0.5, 10).unwrap();
    assert_in_delta!(t, 2.0, 0.01);
    // Rational32 1/2
    let t = factory::geometric::<Rational32>().eval_finite(Rational32::new(1, 2), 10).unwrap();
    assert_eq!(t, Rational32::new(1023, 512));
    // float -0.1
    let t = factory::geometric::<f64>().eval_finite(-0.1, 10).unwrap();
    assert_in_delta!(t, 0.90909, 0.001);
   // Rational32 -1/10
    let t = factory::geometric::<Rational32>().eval_finite(Rational32::new(-1, 10), 8).unwrap();
    assert_eq!(t, Rational32::new(9090909, 10000000));
}

#[test]
fn exp(){
    // float 1.0
    let t = factory::exp::<f64>().eval_finite(1.0, 10).unwrap();
    assert_in_delta!(t, 2.7182, 0.0001);
    // Rational32 1/1
    let t = factory::exp::<Rational32>().eval_finite(Rational32::new(1, 1), 10).unwrap();
    assert_eq!(t, Rational32::new(98641, 36288));
    // float 2.0
    let t = factory::exp::<f64>().eval_finite(2.0, 10).unwrap();
    assert_in_delta!(t, 7.389, 0.001);
    // Rational32 2/1
    let t = factory::exp::<Rational32>().eval_finite(Rational32::new(2, 1), 10).unwrap();
    assert_eq!(t, Rational32::new(20947, 2835));
}

#[test]
fn ln(){
    let t = factory::ln::<i32>().eval_finite(1, 10);
    assert_eq!(t.unwrap(), 0);
    let t = factory::ln::<f64>().eval_finite(1.1, 10);
    assert_in_delta!(t.unwrap(), 0.0953, 0.0001);
    let t = factory::ln::<f64>().eval_finite(0.5, 20);
    assert_in_delta!(t.unwrap(), -0.693147, 0.000001);
    let t = factory::ln::<Rational32>().eval_finite(Rational32::new(1, 2), 5);
    assert_eq!(t.unwrap(), Rational32::new(-131, 192));
    let t = factory::ln::<f64>().eval_finite(1.5, 20);
    assert_in_delta!(t.unwrap(), 0.405465, 0.000001);
}

#[test]
fn ln_from_atanh(){
    let t = factory::ln_from_atanh().eval_finite(1, 10);
    assert_eq!(t, Ok(0));
    let t = factory::ln_from_atanh::<f64>().eval_finite(1.1, 10);
    assert_in_delta!(t.unwrap(), 0.0953, 0.0001);
    let t = factory::ln_from_atanh::<f64>().eval_finite(0.5, 20);
    assert_in_delta!(t.unwrap(), -0.693147, 0.000001);
    let t = factory::ln_from_atanh::<f64>().eval_finite(1.5, 20);
    assert_in_delta!(t.unwrap(), 0.405465, 0.000001);
    let t = factory::ln_from_atanh::<f64>().eval_finite(2.5, 20);
    assert_in_delta!(t.unwrap(), 0.916291, 0.000001);
    let t = factory::ln_from_atanh::<f64>().eval_finite(10.0, 80);
    assert_in_delta!(t.unwrap(), 2.302585, 0.000001);
}


#[test]
fn power_series(){
    let a = sequence_factory::constant(1.0f64);
    assert_eq!(vec![1.0f64; 100], a.terms().take(100).collect::<Vec<_>>());
    let z = 0.5;
    let s = PowerSeries::new(a);
    assert_in_delta!(2.0, s.eval_finite(z, 100).unwrap(), 0.0001);
}
