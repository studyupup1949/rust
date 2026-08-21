use crate::{
    error::{AdicError, AdicResult},
    normed::{Normed, UltraNormed},
    traits::{AdicPrimitive, CanTruncate},
    QAdic, ZAdic,
};
use super::iwasawa_log;


/// Adic digamma for x in Z_p
///
/// The [Digamma Function](<https://en.wikipedia.org/wiki/Digamma_function>) is the derivative of the
///  log of the Gamma function.
/// We calculate it using the [Volkenborn Integral](<https://en.wikipedia.org/wiki/Volkenborn_integral>)
///  of the [`iwasawa_log`](crate::function::special::iwasawa_log) function (Robert, p. 379).
///
/// The [Euler-Mascheroni constant](<https://en.wikipedia.org/wiki/Euler%27s_constant>) is equal to `-digamma(1)`.
/// This will not be the standard constant, it will be in Z_p, different for different p-adic numbers.
///
/// ```
/// # use adic::{function::special::adic_digamma, UAdic, ZAdic};
/// let digamma_five = adic_digamma(UAdic::new(5, vec![0, 1]), 4).unwrap();
/// let digamma_six = adic_digamma(UAdic::new(5, vec![1, 1]), 4).unwrap();
/// assert_eq!(digamma_six, digamma_five);
/// let euler_gamma_5 = ZAdic::new_approx(5, 4, vec![0, 1, 0, 3]);
/// assert_eq!(euler_gamma_5, -adic_digamma(UAdic::new(5, vec![1]), 4)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn adic_digamma<Z>(x: Z, stopping_index : usize) -> AdicResult<ZAdic>
where Z: Into<ZAdic> {

    let x = QAdic::<ZAdic>::from_integer(x.into());
    let p = x.p();
    let zero = QAdic::zero(p);
    let one = QAdic::one(p);
    let mut sum = zero.clone();
    let mut t = x;

    //See if we can extend this to Q_p

    let integration_stop = usize::try_from(u32::from(p))?.pow(u32::try_from(stopping_index)?);
    for _ in 0..integration_stop {
        //We're not sure how quickly this converges but 2 * stopping_index seems like a good starting point
        if t.is_unit(){
            sum = sum + iwasawa_log(t.clone(), 2 * stopping_index)?;
        }
        t = t + one.clone();
    }
    let p_neg_n = QAdic::new(ZAdic::one(p), -isize::try_from(stopping_index)?);
    sum = sum * p_neg_n;
    if (sum.valuation() < 0.into()){
        Err(AdicError::IllDefined("Returns a fractional component, not in Z_p".to_string()))
    } else {
        Ok(sum.into_split(0).1)
    }
}



#[cfg(test)]
mod test {

    use num::Rational32;
    use crate::{
        traits::{AdicPrimitive, CanApproximate, TryPrimedFrom},
        RAdic, ZAdic,
    };
    use super::adic_digamma;

    #[test]
    fn euler_mascheroni() {

        let eg_constant_3 = zadic_approx!(3, 6, [0, 2, 2, 1, 2, 1]);
        let eg_constant_5 = zadic_approx!(5, 4, [0, 1, 0, 3]);
        let eg_constant_7 = zadic_approx!(7, 4, [5, 2, 4, 6]);
        let eg_constant_11 = zadic_approx!(11, 3, [1, 10, 2]);
        let eg_constant_13 = zadic_approx!(13, 3, [0, 4, 0]);
        assert_eq!(Ok(-eg_constant_3), adic_digamma(uadic!(3, [1]), 6));
        assert_eq!(Ok(-eg_constant_5), adic_digamma(uadic!(5, [1]), 4));
        assert_eq!(Ok(-eg_constant_7), adic_digamma(uadic!(7, [1]), 4));
        assert_eq!(Ok(-eg_constant_11), adic_digamma(uadic!(11, [1]), 3));
        assert_eq!(Ok(-eg_constant_13), adic_digamma(uadic!(13, [1]), 3));

    }

    #[test]
    fn simple() {
        let digamma_one = adic_digamma(uadic!(5, [1]), 4).unwrap();
        let digamma_two = adic_digamma(uadic!(5, [2]), 4).unwrap();
        let digamma_three = adic_digamma(uadic!(5, [3]), 4).unwrap();
        let digamma_four = adic_digamma(uadic!(5, [4]), 4).unwrap();
        let digamma_five = adic_digamma(uadic!(5, [0, 1]), 4).unwrap();
        let digamma_six = adic_digamma(uadic!(5, [1, 1]), 4).unwrap();
        let digamma_seven = adic_digamma(uadic!(5, [2, 1]), 4).unwrap();
        let digamma_eight = adic_digamma(uadic!(5, [3, 1]), 4).unwrap();
        let digamma_nine = adic_digamma(uadic!(5, [4, 1]), 4).unwrap();
        let digamma_ten = adic_digamma(uadic!(5, [0, 2]), 4).unwrap();
        assert_eq!(digamma_two, zadic_approx!(5, 4, [1, 4, 4, 1]));
        assert_eq!(digamma_two.clone() - digamma_one, zadic_approx!(5, 4, [1, 0, 0, 0]));
        assert_eq!(
            digamma_three.clone() - digamma_two,
            RAdic::try_primed_from(5, Rational32::new(1, 2)).unwrap().into_approximation(4)
        );
        assert_eq!(
            digamma_four.clone() - digamma_three,
            RAdic::try_primed_from(5, Rational32::new(1, 3)).unwrap().into_approximation(4)
        );
        assert_eq!(
            digamma_five.clone() - digamma_four,
            RAdic::try_primed_from(5, Rational32::new(1, 4)).unwrap().into_approximation(4)
        );
        assert_eq!(digamma_six, digamma_five);
        assert_eq!(
            digamma_seven.clone() - digamma_six,
            RAdic::try_primed_from(5, Rational32::new(1, 6)).unwrap().into_approximation(4)
        );
        assert_eq!(
            digamma_eight.clone() - digamma_seven,
            RAdic::try_primed_from(5, Rational32::new(1, 7)).unwrap().into_approximation(4)
        );
        assert_eq!(
            digamma_nine.clone() - digamma_eight,
            RAdic::try_primed_from(5, Rational32::new(1, 8)).unwrap().into_approximation(4)
        );
        assert_eq!(
            digamma_ten.clone() - digamma_nine,
            RAdic::try_primed_from(5, Rational32::new(1, 9)).unwrap().into_approximation(4)
        );
        let di_ten_minus_five = digamma_ten - digamma_five;
        let fraction_sums = (6..10).map(
            |n| RAdic::try_primed_from(5, Rational32::new(1, n)).unwrap().into_approximation(4)
        ).fold(ZAdic::zero(5), |acc , r| acc + r);
        assert_eq!(di_ten_minus_five, fraction_sums);
    }

}
