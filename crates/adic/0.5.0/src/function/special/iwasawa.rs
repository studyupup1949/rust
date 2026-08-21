use num::traits::Pow;
use crate::{
    divisible::Divisible,
    error::{AdicError, AdicResult},
    normed::{Normed, UltraNormed},
    traits::{AdicPrimitive, CanApproximate, PrimedFrom},
    EAdic, QAdic, UAdic, ZAdic,
};


/// Iwasawa's p-adic Log analogue for all x in Q_p.
///
/// [Wikipedia](<https://en.wikipedia.org/wiki/P-adic_exponential_function#p-adic_logarithm_function>)
///
/// The normal series for ln(x) converges for x in Z_p^x, i.e. `|x|_p < 1`.
/// With Iwasawa's choice of Log(p) = 0, we can extend to all x in Q_p.
/// We can use the Log identity Log(xy) = Log(x) + Log(y) to see for any x in Q_p, Log(x) is determined entirely by its unit.
/// So here we calculate Log(x.unit()) and obtain a series that converges for all Q_p. See (Robert, p. 260) for the definition of Log(x).
///
/// ```
/// # use adic::{error::AdicError, function::special::iwasawa_log, UAdic};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert_eq!(iwasawa_log(UAdic::new(5, vec![1]), 6)?.to_string(), "...000000._5");
/// assert_eq!(iwasawa_log(UAdic::new(5, vec![2]), 6)?.to_string(), "...042320._5");
/// assert_eq!(iwasawa_log(UAdic::new(5, vec![0, 1]), 6)?.to_string(), "...000000._5");
/// let log_three = iwasawa_log(UAdic::new(5, vec![3]), 6)?;
/// let log_nine = iwasawa_log(UAdic::new(5, vec![4, 1]), 6)?;
/// assert_eq!(log_nine, log_three.clone() + log_three.clone());
/// let error = AdicError::IllDefined("Log of zero undefined".to_string());
/// assert_eq!(Err(error), iwasawa_log(UAdic::new(5, vec![]), 6));
/// # Ok(()) }
/// ```
pub fn iwasawa_log<Q>(x : Q, precision : usize) -> AdicResult<QAdic<ZAdic>>
where Q: Into<QAdic<ZAdic>> {
    let x = x.into();
    let p = x.p();
    let one = QAdic::<ZAdic>::one(p);
    let zero = QAdic::zero(p);
    let Some(unit) = x.unit() else {
        return Err(AdicError::IllDefined("Log of zero undefined".to_string()));
    };
    let unit = unit.into_approximation(precision);
    let mut sum = zero.clone();
    let mut numer = one.clone();
    let numer_step = ( one.clone() - QAdic::from_integer(unit.clone().pow(unit.p().m1())) );
    let a = UAdic::primed_from(x.p(), u32::try_from(precision)?);
    let stopping_index = precision + a.finite_num_digits();
    for k in 1..stopping_index {
        numer = numer * numer_step.clone();
        // We will approximate numer appropriately to give a ZAdic with the correct precision
        let denom = QAdic::primed_from(unit.p(), u32::try_from(k)?);
        let denom_val = denom.valuation().finite().ok_or(AdicError::Severe("Adic log series denominator zero; cannot divide".to_string()))?;
        let adj_precision = isize::try_from(precision)? + denom_val;
        sum = sum + numer.approximation(adj_precision) / denom;
    }
    let coeff = ZAdic::from(EAdic::new_repeating(unit.p(), vec![], vec![1]));
    let coeff = QAdic::from_integer(coeff);
    Ok(sum * coeff)
}



#[cfg(test)]
mod test {

    use assertables::assert_matches;
    use num::{traits::Pow, BigUint, Rational32};
    use crate::{
        combinatorics::factorial,
        normed::{UltraNormed, Valuation},
        traits::{AdicInteger, AdicPrimitive, CanApproximate, HasDigits, PrimedFrom},
        QAdic, UAdic, ZAdic,
    };
    use super::iwasawa_log;

    #[test]
    fn adic_log(){
        let t = iwasawa_log(qadic!(uadic!(5, [1]), 0), 10);
        assert_eq!(t.unwrap(), qadic!(zadic_approx!(5, 10, []), 0));
        let t = iwasawa_log(qadic!(uadic!(5, [0, 1]), 0), 9);
        assert_eq!(t.unwrap(), qadic!(zadic_approx!(5, 9, []), 0));
        let t = iwasawa_log(qadic!(uadic!(5, [0, 0, 1]), 0), 8);
        assert_eq!(t.unwrap(), qadic!(zadic_approx!(5, 8, []), 0));
        let t = iwasawa_log(qadic!(ZAdic::zero(5), 0), 10);
        assert_matches!(t, Err(_));

        assert_eq!("...000000._5", iwasawa_log(uadic!(5, [1]), 6).unwrap().to_string());
        assert_eq!("...042320._5", iwasawa_log(uadic!(5, [2]), 6).unwrap().to_string());
        assert_eq!("...431340._5", iwasawa_log(uadic!(5, [3]), 6).unwrap().to_string());
        assert_eq!("...140140._5", iwasawa_log(uadic!(5, [4]), 6).unwrap().to_string());
        assert_eq!("...000000._5", iwasawa_log(uadic!(5, [0, 1]), 6).unwrap().to_string());
        assert_eq!("...024210._5", iwasawa_log(uadic!(5, [1, 1]), 6).unwrap().to_string());
        assert_eq!("...434400._5", iwasawa_log(uadic!(5, [2, 1]), 6).unwrap().to_string());
        assert_eq!("...233010._5", iwasawa_log(uadic!(5, [3, 1]), 6).unwrap().to_string());
        assert_eq!("...413230._5", iwasawa_log(uadic!(5, [4, 1]), 6).unwrap().to_string());
        assert_eq!("...042320._5", iwasawa_log(uadic!(5, [0, 2]), 6).unwrap().to_string());

        for i in 1..10{
            let two = iwasawa_log(uadic!(5, [2]), i);
            let three = iwasawa_log(qadic!(uadic!(5, [3]), 0), i);
            let four = iwasawa_log(qadic!(uadic!(5, [4]), 0), i);
            let five = iwasawa_log(qadic!(uadic!(5, [0, 1]), 0), i);
            let six = iwasawa_log(qadic!(uadic!(5, [1, 1]), 0), i);
            let seven = iwasawa_log(qadic!(uadic!(5, [2, 1]), 0), i);
            let eight = iwasawa_log(qadic!(uadic!(5, [3, 1]), 0), i);
            let nine = iwasawa_log(qadic!(uadic!(5, [4, 1]), 0), i);
            let ten = iwasawa_log(qadic!(uadic!(5, [0, 2]), 0), i);
            //let eleven = factory::zadic_log(qadic!(uadic!(5, [1, 2]), 0), i);
            let twelve = iwasawa_log(qadic!(uadic!(5, [2, 2]), 0), i);
            //let thirteen = factory::zadic_log(qadic!(uadic!(5, [3, 2]), 0), i);
            let fourteen = iwasawa_log(qadic!(uadic!(5, [4, 2]), 0), i);
            let fifteen = iwasawa_log(qadic!(uadic!(5, [0, 3]), 0), i);
            let sixteen = iwasawa_log(qadic!(uadic!(5, [1, 3]), 0), i);
            let two_twenty_four = iwasawa_log(qadic!(uadic!(5, [4, 4, 3, 1]), 0), i);
            assert_eq!(four.clone().unwrap(), two.clone().unwrap() + two.clone().unwrap());
            assert_eq!(six.clone().unwrap(), two.clone().unwrap() + three.clone().unwrap());
            assert_eq!(eight.unwrap(), four.clone().unwrap() + two.clone().unwrap());
            assert_eq!(nine.clone().unwrap(), three.clone().unwrap() + three.clone().unwrap());
            assert_eq!(ten.unwrap(), five.clone().unwrap() + two.clone().unwrap());
            assert_eq!(twelve.unwrap(), six.clone().unwrap() + two.clone().unwrap());
            assert_eq!(fourteen.clone().unwrap(), seven.unwrap() + two.clone().unwrap());
            assert_eq!(fifteen.unwrap(), five.clone().unwrap() + three.clone().unwrap());
            assert_eq!(sixteen.clone().unwrap(), four.clone().unwrap() + four.clone().unwrap());
            assert_eq!(two_twenty_four.unwrap(), sixteen.unwrap() + fourteen.unwrap());
            let three_quarters = iwasawa_log(
                QAdic::<ZAdic>::primed_from(5, Rational32::new(3, 4)),
                i
            );
            assert_eq!(three_quarters.unwrap(), three.clone().unwrap() - four.clone().unwrap());
            let nine_squared = iwasawa_log(qadic!(uadic!(5, [1, 1, 3]), 0), i);
            assert_eq!(nine_squared.unwrap(), nine.clone().unwrap() + nine.clone().unwrap());
            //log(sqrt([4, 1])) = 1/2 * log ([4, 1])
            let half = QAdic::<ZAdic>::primed_from(5, Rational32::new(1, 2));
            assert_eq!(three.clone().unwrap(), half.clone() * nine.clone().unwrap());
            let two_fifths = iwasawa_log(qadic!(uadic!(5, [2]), -1), i);
            assert_eq!(two_fifths.unwrap(), two.clone().unwrap() - five.clone().unwrap());
            let variety = uadic!(5, [1, 1]).nth_root(2, i).unwrap();
            for root in variety.into_roots() {
                let sqrt_six = iwasawa_log(QAdic::from_integer(root), i);
                assert_eq!(sqrt_six.clone().unwrap(), half.clone() * six.clone().unwrap());
            }
        }

    }

    #[test]
    fn log_exp_inverse() {

        // The exponential and logarithm functions are pseudo-inverses of eachother
        // If v(x) > 0, then  exp(log(1 + x)) = 1 + x
        // If x is any p-adic integer, v is its valuation, and is its Teichmuller character,
        //  then x can be expressed as  x = p^v t (1 + y), where v(y) > 0,
        //  and so  exp(log(x)) = exp(log(p^v t (1 + y))) = exp(log(1 + y)) = 1 + y

        let p = 5;
        let prec = 6;
        let p_to_n = 125;
        for n in 1..p_to_n {

            // Calculate exp(log(a))
            let a = UAdic::primed_from(p, n);
            let log_a = iwasawa_log(a.clone(), prec).unwrap();
            // TODO: Replace this manual exp implementation with the official one
            let exp_log_a = (0..7).map(|i| {
                let qfact = QAdic::from(UAdic::primed_from(p, factorial::<BigUint>(i)));
                let val = qfact.valuation().finite().unwrap();
                log_a.clone().pow(i).approximation(prec as isize + val) / qfact
            }).fold(QAdic::zero(p), |acc, x| acc + x);

            // exp(log(a)) = exp(log(p^v * t * (1 + b))) = exp(log(1 + b)) = 1 + b = p^(-v) t^(-1) a
            let (Some(u), Valuation::Finite(v)) = a.unit_and_valuation() else {
                panic!("a should be nonzero");
            };
            let first_nonzero = u.digit0().unwrap();
            let variety = ZAdic::teichmuller(p, prec - v).unwrap();
            let t = variety.roots().nth(first_nonzero as usize).unwrap().clone();
            let qt = QAdic::from_integer(t) * QAdic::new(ZAdic::one(p), v as isize);

            // a = p^v * t * exp(log(a))
            assert_eq!(QAdic::from_integer(a.into_approximation(prec)), qt * exp_log_a);

        }

    }

}
