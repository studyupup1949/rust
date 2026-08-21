use std::{
    fmt,
    iter::{empty, repeat, repeat_n},
    ops::Add,
};
use itertools::{Either, Itertools};
use num::{traits::{Euclid, Pow}, Integer, Zero};
use crate::{
    AdicApproximate, AdicError, AdicFraction, AdicInteger, AdicNumber, AdicResult, AdicValuation, AdicValuationRing,
    Composite, Divisible, ExactIntegerVariant, HasDigits, IAdic, QAdic, RAdic, UAdic, ZAdic
};
use super::{
    n_adic_digits::NAdicDigits,
    AdicComposite, AdicPower,
};


impl<A> HasDigits for AdicPower<A>
where A: AdicApproximate + AdicNumber + HasDigits {

    type DigitIndex = A::DigitIndex;

    fn base(&self) -> Composite {
        self.p_pow().into()
    }

    fn min_index(&self) -> AdicValuation<Self::DigitIndex> {
        match self.adic.min_index() {
            AdicValuation::PosInf => AdicValuation::PosInf,
            AdicValuation::Finite(m) => {
                AdicValuation::Finite(m.div_euclid(&self.power_valuation()))
            },
        }
    }

    fn num_digits(&self) -> AdicValuation<usize> {
        match self.adic.num_digits() {
            AdicValuation::PosInf => AdicValuation::PosInf,
            AdicValuation::Finite(nd) => AdicValuation::Finite(nd / self.power_usize()),
        }
    }

    fn digit(&self, n: Self::DigitIndex) -> AdicResult<u32> {
        let power = self.power();
        let pusize = self.power_usize();
        let pval = self.power_valuation();
        let digit_subset = (0..pusize).map(|i| {
            let ival = Self::DigitIndex::try_from_usize(i)?;
            HasDigits::digit(&self.adic, n * pval + ival)
        }).collect::<AdicResult<Vec<_>>>()?;
        Ok(digit_subset.into_iter().zip(0..power).map(
            |(d, i)| d * u32::from(self.p().pow(i))
        ).sum())
    }

    fn digits(&self) -> impl Iterator<Item=u32> {

        // Calculates the digits of the `p^n`-adic expansion, using the stored `p`-adic

        let p = self.p();
        let pusize = self.power_usize();

        let adjusted_digits = sandwich_with_zeros(self.clone());

        // Do the power summation in batches
        adjusted_digits.batching(move |it| {
            let digits = it.take(pusize).collect::<Vec<_>>();
            if digits.len() < pusize {
                return None;
            }
            let big_digit = digits.into_iter().enumerate().map(|(i, d)| {
                d * u32::from(p.pow(i.try_into().expect("usize -> u32 error")))
            }).sum();
            Some(big_digit)
        })

    }

    fn into_digits(self) -> impl Iterator<Item=u32> {

        // Calculates the digits of the `p^n`-adic expansion, using the stored `p`-adic

        let p = self.p();
        let pusize = self.power_usize();

        let adjusted_digits = sandwich_with_zeros(self);

        // Do the power summation in batches
        adjusted_digits.batching(move |it| {
            let digits = it.take(pusize).collect::<Vec<_>>();
            if digits.len() < pusize {
                return None;
            }
            let big_digit = digits.into_iter().enumerate().map(|(i, d)| {
                d * u32::from(p.pow(i.try_into().expect("usize -> u32 error")))
            }).sum();
            Some(big_digit)
        })

    }

}


impl<A> HasDigits for AdicComposite<A>
where A: AdicApproximate + AdicNumber + HasDigits {

    type DigitIndex = A::DigitIndex;

    fn base(&self) -> Composite {
        Composite::new(self.p_adics.values().map(AdicPower::p_pow))
    }

    fn min_index(&self) -> AdicValuation<Self::DigitIndex> {
        // Note: this should be right for negative valuation.
        // Remember we are rounding negatively on both sides.
        let base = self.base();
        self.p_adics.values().map(|ap| match ap.adic_ref().min_index() {
            AdicValuation::PosInf => AdicValuation::PosInf,
            AdicValuation::Finite(mi) if mi < Self::DigitIndex::zero() => {
                let base_wo_p = base.without_p(ap.p());
                let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                let base_wo_p = Self::DigitIndex::try_from_usize(base_wo_p).expect("base conversion to valuation ring");
                AdicValuation::Finite(mi.div_euclid(&base_wo_p))
            },
            AdicValuation::Finite(_) => AdicValuation::Finite(Self::DigitIndex::zero()),
        }).min().unwrap_or(AdicValuation::PosInf)
    }

    fn num_digits(&self) -> AdicValuation<usize> {
        // Note: this should be right for negative valuation.
        // Remember we are rounding negatively on both sides.
        let base = self.base();
        self.p_adics.values().map(|ap| {
            if let AdicValuation::Finite(nd) = ap.adic_ref().num_digits() {
                let base_wo_p = base.without_p(ap.p());
                let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                AdicValuation::Finite(nd.div_euclid(base_wo_p))
            } else {
                AdicValuation::PosInf
            }
        }).min().unwrap_or(AdicValuation::PosInf)
    }

    fn digit(&self, n: Self::DigitIndex) -> AdicResult<u32> {
        match self.min_index() {
            AdicValuation::Finite(mi) if n >= mi => {
                let n = (n - mi).try_into_usize().expect("convert valuation to usize");
                // TODO: Carefully handle the p_adic hashmap to multiply the right digits
                self.digits().nth(n).ok_or(AdicError::InappropriatePrecision(
                    "Not enough calculable digits in AdicComposite".to_string()
                ))
            }
            _ => Ok(0),
        }
    }

    fn digits(&self) -> impl Iterator<Item=u32> {

        if self.p_adics.values().all(AdicPower::is_zero) {
            return Either::Left(empty());
        }

        let base = self.base();
        let AdicValuation::Finite(certainty) = self.certainty() else {
            panic!("No adic number uncertainty found; infinite n-adic digits");
        };

        let digit_vec = match self.min_index() {
            AdicValuation::PosInf => vec![],
            AdicValuation::Finite(mi) if certainty < mi => vec![],
            AdicValuation::Finite(mi) => {

                let significance = (certainty - mi);
                let sigusize = significance.try_into_usize().expect("convert valuation to usize");

                let mut n_adic_pieces = vec![];
                for (p, ap) in &self.p_adics {

                    let base_wo_p = base.without_p(*p);
                    let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                    let base_wo_p = Self::DigitIndex::try_from_usize(base_wo_p).expect("convert usize to valuation");
                    let pow32 = ap.power();
                    let powval = ap.power_valuation();
                    let num_digits = significance * base_wo_p * powval;
                    let nd_usize = num_digits.try_into_usize().expect("convert valuation to usize");

                    let adjusted_digits = sandwich_with_zeros(ap.clone());
                    let adjusted_ap = AdicPower::new(ZAdic::new_approx(*p, nd_usize, adjusted_digits.collect()), pow32);

                    let b_p = NAdicDigits::from_p_adic(base.clone(), adjusted_ap)
                        .expect("problem converting from p-adic to n-adic");

                    // Multiply by the correct idempotent
                    // This is the product of all prime idempotents EXCEPT the one for p
                    let all_but_p_idempotent = NAdicDigits::idempotent_excluding(*p, &base, sigusize)
                        .expect("problem calculating idempotent");

                    n_adic_pieces.push((b_p * all_but_p_idempotent).into_truncation(sigusize));

                }

                let full_n_adic = n_adic_pieces.into_iter()
                    .reduce(NAdicDigits::add)
                    .unwrap_or(NAdicDigits::zero(base, sigusize))
                    .into_truncation(sigusize);
                full_n_adic.into_digits().collect::<Vec<_>>()

            }
        };

        Either::Right(digit_vec.into_iter())

    }

    fn into_digits(self) -> impl Iterator<Item=u32> {

        if self.p_adics.values().all(AdicPower::is_zero) {
            return Either::Left(empty());
        }

        let base = self.base();
        let AdicValuation::Finite(certainty) = self.certainty() else {
            panic!("No adic number uncertainty found; infinite n-adic digits");
        };

        let digit_vec = match self.min_index() {
            AdicValuation::PosInf => vec![],
            AdicValuation::Finite(mi) if certainty < mi => vec![],
            AdicValuation::Finite(mi) => {

                let significance = (certainty - mi);
                let sigusize = significance.try_into_usize().expect("convert valuation to usize");

                let mut n_adic_pieces = vec![];
                for (p, ap) in self.p_adics {

                    let base_wo_p = base.without_p(p);
                    let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                    let base_wo_p = Self::DigitIndex::try_from_usize(base_wo_p).expect("convert usize to valuation");
                    let pow32 = ap.power();
                    let powval = ap.power_valuation();
                    let num_digits = significance * base_wo_p * powval;
                    let nd_usize = num_digits.try_into_usize().expect("convert valuation to usize");

                    let adjusted_digits = sandwich_with_zeros(ap);
                    let adjusted_ap = AdicPower::new(ZAdic::new_approx(p, nd_usize, adjusted_digits.collect()), pow32);

                    let b_p = NAdicDigits::from_p_adic(base.clone(), adjusted_ap)
                        .expect("problem converting from p-adic to n-adic");

                    // Multiply by the correct idempotent
                    // This is the product of all prime idempotents EXCEPT the one for p
                    let all_but_p_idempotent = NAdicDigits::idempotent_excluding(p, &base, sigusize)
                        .expect("problem calculating idempotent");

                    n_adic_pieces.push((b_p * all_but_p_idempotent).into_truncation(sigusize));

                }

                let full_n_adic = n_adic_pieces.into_iter()
                    .reduce(NAdicDigits::add)
                    .unwrap_or(NAdicDigits::zero(base, sigusize))
                    .into_truncation(sigusize);
                full_n_adic.into_digits().collect::<Vec<_>>()

            }
        };

        Either::Right(digit_vec.into_iter())

    }

}



trait HasDigitStr {
    /// This trait creates a string representing the digits of this `AdicPower`.
    /// It is internal, just used to Display it.
    fn digit_str(&self) -> String;
}


impl HasDigitStr for AdicPower<IAdic> {

    fn digit_str(&self) -> String {
        let pp = self.p_pow();
        if self.adic_ref().is_non_negative() {
            // Finite digits
            let ds = self.digits().map(|d| pp.display_digit(d)).collect::<Vec<_>>();
            ds.into_iter().rev().join("")
        } else {
            // "Infinite" digits, show (p^n-1) and then the finite part
            let num_non_trailing = self.adic_ref().num_non_trailing().div_ceil(self.power_usize());
            let pm1_symbol = pp.display_digit(pp.m1());
            let ds = self.digits().take(num_non_trailing).map(|d| pp.display_digit(d)).collect::<Vec<_>>();
            let digits = ds.into_iter().rev().join("");
            format!("({pm1_symbol}){digits}")
        }
    }

}

impl HasDigitStr for AdicPower<RAdic> {
    fn digit_str(&self) -> String {

        let pp = self.p_pow();
        let fix_d = self.adic.fixed_digits().collect::<Vec<_>>();
        let rep_d = self.adic.repeat_digits().collect::<Vec<_>>();
        if rep_d.is_empty() {
            return AdicPower::new(UAdic::new(self.p(), fix_d), self.power()).digit_str();
        }

        // Start at adjusted min_index, need to preload some zeros
        let pusize = self.power_usize();
        let num_zeros = match self.adic.min_index() {
            AdicValuation::Finite(adic_min) => adic_min.rem_euclid(pusize),
            AdicValuation::PosInf => 0,
        };
        let num_fixed = (num_zeros + fix_d.len()).div_ceil(pusize);
        let num_repeat = rep_d.len() / rep_d.len().gcd(&pusize);

        let fix_digits = self.digits().take(num_fixed).map(|d| pp.display_digit(d)).collect::<Vec<_>>();
        let fix_str = fix_digits.into_iter().rev().join("");
        let rep_digits = self.digits().skip(num_fixed).take(num_repeat).map(|d| pp.display_digit(d)).collect::<Vec<_>>();
        let rep_str = rep_digits.into_iter().rev().join("");
        format!("({rep_str}){fix_str}")

    }
}

impl HasDigitStr for AdicPower<UAdic> {
    fn digit_str(&self) -> String {
        let pp = self.p_pow();
        let digits = self.digits().map(|d| pp.display_digit(d)).collect::<Vec<_>>();
        digits.into_iter().rev().join("")
    }
}

impl HasDigitStr for AdicPower<ZAdic> {
    fn digit_str(&self) -> String {
        let pp = self.p_pow();
        let z = self.adic_ref();
        match z.exact_variant_or_certainty() {
            Either::Left(var) => match var {
                ExactIntegerVariant::Unsigned(u) => AdicPower::new(u.clone(), self.power()).digit_str(),
                ExactIntegerVariant::Signed(i) => AdicPower::new(i.clone(), self.power()).digit_str(),
                ExactIntegerVariant::Rational(r) => AdicPower::new(r.clone(), self.power()).digit_str(),
            },
            Either::Right(c) => {
                let adjusted_cert = c / self.power_usize();
                let ds = self.digits().chain(repeat(0)).take(adjusted_cert).map(|d| pp.display_digit(d)).collect::<Vec<_>>();
                let digits = ds.into_iter().rev().join("");
                format!("...{digits}")
            },
        }
    }
}


impl<A> HasDigitStr for AdicComposite<A>
where Self: HasDigits, A: AdicApproximate + AdicNumber {
    fn digit_str(&self) -> String {
        let b = self.base();
        let ds = self.digits().map(|d| b.display_digit(d)).collect::<Vec<_>>();
        let digits = ds.into_iter().rev().join("");
        if self.is_certain() {
            if digits.is_empty() {
                "0".to_string()
            } else {
                digits.to_string()
            }
        } else {
            format!("...{digits}")
        }
    }
}


impl<A> fmt::Display for AdicPower<A>
where A: AdicInteger, AdicPower<A>: HasDigitStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pp = self.p_pow();
        let pp32 = u32::from(pp);
        let digits = self.digit_str();
        if digits.is_empty() {
            let zero = pp.display_zero();
            write!(f, "{zero}._{pp32}")
        } else {
            write!(f, "{digits}._{pp32}")
        }
    }
}


impl<A> fmt::Display for AdicPower<QAdic<A>>
where A: AdicApproximate + AdicInteger, AdicPower<A>: HasDigitStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pp = self.p_pow();
        let pp32 = u32::from(pp);
        match self.min_index() {
            AdicValuation::PosInf => {
                let zero = pp.display_zero();
                write!(f, "{zero}._{pp32}")
            },
            AdicValuation::Finite(mi) => {
                let num_frac = mi.unsigned_abs();
                let power = self.power();
                let frac_digits = self.digits().chain(repeat(0)).take(num_frac).map(|d| pp.display_digit(d)).collect::<Vec<_>>();
                let frac_str = frac_digits.into_iter().rev().join("");
                let int_power = AdicPower::new(self.adic.frac_and_int().1, power);
                let int_str = int_power.digit_str();
                let int_str = if int_str.is_empty() { self.p_pow().display_zero() } else { int_str };
                write!(f, "{int_str}.{frac_str}_{pp32}")
            }
        }
    }
}


impl<A> fmt::Display for AdicComposite<A>
where A: AdicApproximate + AdicInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.base();
        let b32 = u32::from(b.clone());
        let digits = self.digit_str();
        if digits.is_empty() {
            let zero = b.display_zero();
            write!(f, "{zero}._{b32}")
        } else {
            write!(f, "{digits}._{b32}")
        }
    }
}

impl<A> fmt::Display for AdicComposite<QAdic<A>>
where A: AdicApproximate + AdicInteger, AdicComposite<A>: HasDigitStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = self.base();
        let b32 = u32::from(b.clone());
        match self.min_index() {
            AdicValuation::PosInf => {
                let zero = b.display_zero();
                write!(f, "{zero}._{b32}")
            },
            AdicValuation::Finite(mi) => {
                let num_frac = mi.unsigned_abs();
                let frac_digits = self.digits().chain(repeat(0)).take(num_frac).map(|d| b.display_digit(d)).collect::<Vec<_>>();
                let frac_str = frac_digits.into_iter().rev().join("");
                let int_composite = AdicComposite::new(self.p_adics.values().map(|ap| {
                    let power = ap.power();
                    AdicPower::new(ap.adic.frac_and_int().1, power)
                }));
                let int_str = int_composite.digit_str();
                let int_str = if int_str.is_empty() { b.display_zero() } else { int_str };
                write!(f, "{int_str}.{frac_str}_{b32}")
            }
        }
    }
}



fn sandwich_with_zeros<A>(adic_power: AdicPower<A>) -> impl Iterator<Item=u32>
where A: AdicApproximate + AdicNumber + HasDigits {

    let pusize = adic_power.power_usize();
    let pval = adic_power.power_valuation();
    let min_index = adic_power.adic.min_index();
    let num_digits = adic_power.adic.num_digits();
    let certainty = adic_power.adic.certainty();

    // Start at adjusted min_index, need to preload and postload some zeros
    let num_zeros_before = match min_index {
        AdicValuation::Finite(adic_min) => {
            let min_rem = adic_min.rem_euclid(&pval);
            min_rem.try_into_usize().expect("convert valuation to usize")
        },
        AdicValuation::PosInf => 0,
    };
    let num_zeros_after = match num_digits {
        AdicValuation::Finite(nd) => pusize - 1 - (num_zeros_before + nd + 1).rem_euclid(pusize),
        AdicValuation::PosInf => 0,
    };

    let adjusted_digits = repeat_n(0, num_zeros_before)
        .chain(adic_power.adic.into_digits())
        .chain(repeat_n(0, num_zeros_after));

    // Truncate the iterator if it goes past certainty
    match (certainty, min_index) {
        (AdicValuation::Finite(c), AdicValuation::Finite(mi)) if c > mi => {
            let cdiff = (c - mi).try_into_usize().expect("convert valuation to usize");
            Either::Left(adjusted_digits.take(num_zeros_before + cdiff))
        },
        _ => Either::Right(adjusted_digits)
    }

}



#[cfg(test)]
mod tests {

    use crate::{
        apow, iadic_neg, qadic, radic, uadic, zadic_approx, zadic_exact,
        AdicComposite, AdicNumber, AdicPower, QAdic, UAdic,
    };

    #[test]
    fn display_adic_power() {

        let three_adic = uadic!(3, [1, 0, 2, 0, 0, 1, 1, 1, 2, 1, 0, 2]);
        assert_eq!("201211100201._3", three_adic.to_string());
        let nine_adic = AdicPower::new(three_adic.clone(), 2);
        assert_eq!("654321._9", nine_adic.to_string());
        let qnine_adic = AdicPower::new(qadic!(three_adic, -3), 2);
        assert_eq!("21740.63_9", qnine_adic.to_string());

        let three_adic = iadic_neg!(3, [1, 0, 2, 0, 0, 1, 1, 1, 2, 1, 0, 2, 1]);
        assert_eq!("(2)1201211100201._3", three_adic.to_string());
        let nine_adic = AdicPower::new(three_adic.clone(), 2);
        assert_eq!("(8)7654321._9", nine_adic.to_string());
        let qnine_adic = AdicPower::new(qadic!(three_adic, -3), 2);
        assert_eq!("(8)51740.63_9", qnine_adic.to_string());

        let three_adic = radic!(3, [1, 0, 2], [0, 1, 1, 1]);
        assert_eq!("(1110)201._3", three_adic.to_string());
        let nine_adic = AdicPower::new(three_adic.clone(), 2);
        assert_eq!("(14)21._9", nine_adic.to_string());
        let qnine_adic = AdicPower::new(qadic!(three_adic, -3), 2);
        assert_eq!("(43).63_9", qnine_adic.to_string());

        let three_adic = zadic_approx!(3, 13, [1, 0, 2, 0, 0, 1, 1, 1, 2, 1, 0, 2, 1]);
        assert_eq!("...1201211100201._3", three_adic.to_string());
        let nine_adic = AdicPower::new(three_adic.clone(), 2);
        assert_eq!("...654321._9", nine_adic.to_string());
        let qnine_adic = AdicPower::new(qadic!(three_adic, -3), 2);
        assert_eq!("...51740.63_9", qnine_adic.to_string());

        assert_eq!("0._9", AdicPower::new(UAdic::zero(3), 2).to_string());
        assert_eq!("0._9", AdicPower::new(QAdic::<UAdic>::zero(3), 2).to_string());
        assert_eq!("0.12_9", AdicPower::new(qadic!(uadic!(3, [2, 0, 1, 0]), -4), 2).to_string());
        assert_eq!("0.012_9", AdicPower::new(qadic!(uadic!(3, [2, 0, 1, 0]), -6), 2).to_string());

        assert_eq!("nb._25", apow!(uadic!(5, [1, 2, 3, 4]), 2).to_string());
        assert_eq!("[31].[15]_49", apow!(qadic!(uadic!(7, [1, 2, 3, 4]), -2), 2).to_string());
        assert_eq!("(o)nb._25", apow!(iadic_neg!(5, [1, 2, 3, 4]), 2).to_string());
        assert_eq!("([48])[31].[15]_49", apow!(qadic!(iadic_neg!(7, [1, 2, 3, 4]), -2), 2).to_string());
        assert_eq!("(a9)nb._25", apow!(radic!(5, [1, 2, 3, 4], [4, 1, 0, 2]), 2).to_string());
        assert_eq!("([14][11])[31].[15]_49", apow!(qadic!(radic!(7, [1, 2, 3, 4], [4, 1, 0, 2]), -2), 2).to_string());
        assert_eq!("(o)nb._25", apow!(zadic_exact!(iadic_neg!(5, [1, 2, 3, 4])), 2).to_string());
        assert_eq!("...nb._25", apow!(zadic_approx!(5, 4, [1, 2, 3, 4]), 2).to_string());
        assert_eq!("...[31].[15]_49", apow!(qadic!(zadic_approx!(7, 4, [1, 2, 3, 4]), -2), 2).to_string());

    }

    #[test]
    fn display_adic_composite() {

        let ac = AdicComposite::approx_from_i32(10, 1, 6).unwrap();
        assert_eq!("...000001._10".to_string(), ac.to_string());
        let ac = AdicComposite::approx_from_i32(10, 2, 6).unwrap();
        assert_eq!("...000002._10".to_string(), ac.to_string());
        let ac = AdicComposite::approx_from_i32(10, 5, 6).unwrap();
        assert_eq!("...000005._10".to_string(), ac.to_string());
        let ac = AdicComposite::approx_from_i32(10, 100, 6).unwrap();
        assert_eq!("...000100._10".to_string(), ac.to_string());
        let ac = AdicComposite::approx_from_i32(10, -1, 6).unwrap();
        assert_eq!("...999999._10".to_string(), ac.to_string());
        let ac = AdicComposite::approx_from_i32(10, -100, 6).unwrap();
        assert_eq!("...999900._10".to_string(), ac.to_string());
        let ac = AdicComposite::approx_from_i32(36, -1, 6).unwrap();
        assert_eq!("...zzzzzz._36".to_string(), ac.to_string());
        let ac = AdicComposite::approx_from_i32(37, -1, 6).unwrap();
        assert_eq!("...[36][36][36][36][36][36]._37".to_string(), ac.to_string());

    }

}
