use assertables::{assert_approx_eq, assert_ge, assert_lt};
use crate::{
    normed::{UltraNormed, Valuation},
    sequence::{factory, Sequence},
    traits::PrimedFrom,
    EAdic,
};


#[test]
fn constant() {

    let c = factory::constant(3);
    assert_eq!(vec![3, 3, 3, 3, 3], c.truncation(5));
    assert!(!c.is_finite_sequence());
    assert_eq!(vec![6, 6, 6, 6, 6], c.clone().term_add(c).truncation(5));

}

#[test]
fn linear(){
    let a = factory::linear(1);
    let v: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    assert_eq!(v, a.terms().take(10).collect::<Vec<_>>());
}

#[test]
fn geometric_progression(){
    let t = factory::geometric_progression(2);
    assert_eq!(vec![1, 2, 4, 8, 16, 32, 64, 128, 256, 512], t.truncation(10));
}

#[test]
fn exponential_progression(){

    let s = factory::exponential_progression::<f64>(1.0);
    let expected = vec![1.0, 1.0, 0.5, 0.16666666666666666, 0.041666666666666664, 0.008333333333333333];
    assert_eq!(s.truncation(6), expected);
    let s = factory::exponential_progression::<f64>(2.0);
    let expected = vec![1.0, 2.0, 2.0, 1.3333333333333333, 0.6666666666666666, 0.26666666666666666];
    assert_eq!(s.truncation(6), expected);

    let s = factory::exponential_progression::<f64>(2.0);
    assert_approx_eq!(s.terms().nth(4).unwrap(), 2.0f64.powf(4.0) / (1.0 * 2.0 * 3.0 * 4.0));

}

#[test]
fn power(){

    fn assert_critical_valuation_index<S> (sequence: S, num_elements: usize, index: usize, valuation: Valuation<usize>)
    where S: Sequence, S::Term: UltraNormed<ValuationRing = usize> {
        for (i, term) in sequence.enumerate().truncation(num_elements) {
            if i < index {
                assert_lt!(term.valuation(), valuation);
            } else {
                assert_ge!(term.valuation(), valuation);
            }
        }
    }

    fn assert_all_elements_valuation<S> (sequence: S, num_elements: usize, valuation: Valuation<usize>)
    where S: Sequence, S::Term: UltraNormed<ValuationRing = usize> {
        for term in sequence.truncation(num_elements) {
            assert_eq!(term.valuation(), valuation);
        }
    }

    let a = factory::power(1, 5, 1);
    assert_eq!(vec![1, 5, 25, 125, 625, 3125], a.truncation(6));

    let a = factory::power(1, 5, 2);
    assert_eq!(factory::power(1, 5, 1).term_map(|t| t * t).truncation(3), a.truncation(3));

    let a = factory::power(EAdic::primed_from(5, 1), EAdic::primed_from(5, 5), 1);
    assert_critical_valuation_index(a, 20, 6, 6.into());

    let a = factory::power(EAdic::primed_from(5, 1), EAdic::primed_from(5, 3), 1);
    assert_all_elements_valuation(a, 20, 0.into());

    let a = factory::power(EAdic::primed_from(5, 5), EAdic::primed_from(5, 3), 1);
    assert_all_elements_valuation(a, 20, 1.into());

    let a = factory::power(EAdic::primed_from(5, 1), EAdic::primed_from(5, 5), 2);
    assert_critical_valuation_index(a, 20, 3, 6.into());
}

#[test]
fn lucas() {
    let t = factory::lucas((2, 1));
    assert_eq!(vec![2, 1, 3, 4, 7, 11, 18, 29], t.truncation(8));
}

#[test]
fn fibonacci() {
    let t = factory::fibonacci();
    assert_eq!(t.terms().nth(10), Some(55));

    let t = factory::fibonacci();
    assert_eq!(vec![0u32, 1, 1, 2, 3, 5, 8, 13, 21, 34], t.truncation(10));
}

#[test]
fn collatz() {
    let t = factory::collatz(5);
    assert_eq!(vec![5u32, 16, 8, 4, 2, 1], t.truncation(6));
    assert_eq!(vec![5u32, 16, 8, 4, 2, 1, 4, 2, 1], t.truncation(9));

    let t = factory::collatz(0);
    assert_eq!(vec![0u32, 0, 0, 0], t.truncation(4));
}

#[test]
fn thue_morse() {
    let t = factory::thue_morse(0);
    assert_eq!(vec![0u32, 1, 1, 0, 1, 0, 0, 1], t.truncation(8));

    let t = factory::thue_morse(1);
    assert_eq!(vec![1u32, 0, 0, 1, 0, 1, 1, 0], t.truncation(8));

    let t0 = factory::thue_morse(0).term_map(|a| 1 - a).truncation(10);
    let t1 = factory::thue_morse(1).truncation(10);
    assert_eq!(t0, t1);
}

#[test]
fn composed_sequences() {
    let t = factory::thue_morse(0);
    let c = factory::collatz(5);
    let f = factory::fibonacci();

    let l = factory::linear(1);
    //   0  1  2  3  4  5  6  7
    // + 0  1  1  2  3  5  8 13
    //   0  2  3  5  7 10 14 20
    let c1 = f.term_add(l);

    assert_eq!(c1.truncation(8), vec![0, 2, 3, 5, 7, 10, 14, 20]);

    let c2 = t.clone().term_mul(c.clone(), false);
    assert_eq!(c2.truncation(8), vec![0, 16, 8, 0, 2, 0, 0, 2]);
    let c2 = t.clone().term_mul(c.clone(), true);
    assert_eq!(c2.truncation(8), vec![0, 16, 8, 0, 2, 0, 0, 2]);

    let c3 = t.clone().term_mul(vec![1, 2, 3], false);
    assert_eq!(c3.truncation(8), vec![0, 2, 3]);
    let c3 = t.clone().term_mul(vec![1, 2, 3], true);
    assert_eq!(c3.truncation(8), vec![0, 2, 3, 0, 1, 0, 0, 1]);

}
