use std::{
    fmt,
    iter::{empty, repeat, repeat_n},
    ops::Add,
};
use itertools::{Either, Itertools};
use num::{traits::{Euclid, Pow}, Integer, Zero};
use crate::{
    divisible::{Composite, Divisible},
    error::{AdicError, AdicResult},
    local_num::{LocalOne, LocalZero},
    normed::{Valuation, ValuationRing},
    traits::{AdicInteger, AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigitDisplay, HasDigits},
    EAdic, IAdic, IntegerVariant, QAdic, RAdic, UAdic, ZAdic,
};
use super::{
    m_adic_digits::MAdicDigits,
    MAdic, PowAdic,
};



impl<A> LocalZero for PowAdic<A>
where A: AdicPrimitive {
    fn local_zero(&self) -> Self {
        PowAdic::new(A::zero(self.p()), self.power())
    }
    fn is_local_zero(&self) -> bool {
        self.adic_ref().is_local_zero()
    }
}

impl<A> LocalOne for PowAdic<A>
where A: AdicPrimitive {
    fn local_one(&self) -> Self {
        PowAdic::new(A::one(self.p()), self.power())
    }
    fn is_local_one(&self) -> bool {
        self.adic_ref().is_local_one()
    }
}


impl<A> LocalZero for MAdic<A>
where A: AdicPrimitive + HasApproximateDigits {
    fn local_zero(&self) -> Self {
        Self::new(self.pow_adics().map(PowAdic::local_zero))
    }
    fn is_local_zero(&self) -> bool {
        self.pow_adics().all(PowAdic::is_local_zero)
    }
}

impl<A> LocalOne for MAdic<A>
where A: AdicPrimitive + HasApproximateDigits {
    fn local_one(&self) -> Self {
        Self::new(self.pow_adics().map(PowAdic::local_one))
    }
    fn is_local_one(&self) -> bool {
        self.pow_adics().all(PowAdic::is_local_one)
    }
}


// Needs HasApproximateDigits because of certainty is needed to determine
//  whether to keep or discard the leftmost batch of digits when they don't align with the power
impl<A> HasDigits for PowAdic<A>
where A: HasApproximateDigits + AdicPrimitive {

    type DigitIndex = A::DigitIndex;

    fn base(&self) -> Composite {
        self.p_pow().into()
    }

    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        match self.adic.min_index() {
            Valuation::PosInf => Valuation::PosInf,
            Valuation::Finite(m) => {
                Valuation::Finite(m.div_euclid(&self.power_valuation()))
            },
        }
    }

    fn num_digits(&self) -> Valuation<usize> {
        match self.adic.num_digits() {
            Valuation::PosInf => Valuation::PosInf,
            Valuation::Finite(nd) => Valuation::Finite(nd / self.power_usize()),
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


impl<A> HasDigits for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive {

    type DigitIndex = A::DigitIndex;

    fn base(&self) -> Composite {
        Composite::new(self.pow_adics.values().map(PowAdic::p_pow))
    }

    fn min_index(&self) -> Valuation<Self::DigitIndex> {
        // Note: this should be right for negative valuation.
        // Remember we are rounding negatively on both sides.
        let base = self.base();
        self.pow_adics.values().map(|ap| match ap.adic_ref().min_index() {
            Valuation::PosInf => Valuation::PosInf,
            Valuation::Finite(mi) if mi < Self::DigitIndex::zero() => {
                let base_wo_p = base.without_p(ap.p());
                let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                let base_wo_p = Self::DigitIndex::try_from_usize(base_wo_p).expect("base conversion to valuation ring");
                Valuation::Finite(mi.div_euclid(&base_wo_p))
            },
            Valuation::Finite(_) => Valuation::Finite(Self::DigitIndex::zero()),
        }).min().unwrap_or(Valuation::PosInf)
    }

    fn num_digits(&self) -> Valuation<usize> {
        // Note: this should be right for negative valuation.
        // Remember we are rounding negatively on both sides.
        let base = self.base();
        self.pow_adics.values().map(|ap| {
            if let Valuation::Finite(nd) = ap.adic_ref().num_digits() {
                let base_wo_p = base.without_p(ap.p());
                let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                Valuation::Finite(nd.div_euclid(base_wo_p))
            } else {
                Valuation::PosInf
            }
        }).min().unwrap_or(Valuation::PosInf)
    }

    fn digit(&self, n: Self::DigitIndex) -> AdicResult<u32> {
        match self.min_index() {
            Valuation::Finite(mi) if n >= mi => {
                let n = (n - mi).try_into_usize().expect("convert valuation to usize");
                // TODO: Carefully handle the p_adic hashmap to multiply the right digits
                self.digits().nth(n).ok_or(AdicError::InappropriatePrecision(
                    "Not enough calculable digits in MAdic".to_string()
                ))
            }
            _ => Ok(0),
        }
    }

    fn digits(&self) -> impl Iterator<Item=u32> {

        if self.pow_adics.values().all(PowAdic::is_local_zero) {
            return Either::Left(empty());
        }

        let base = self.base();
        let Valuation::Finite(certainty) = self.certainty() else {
            panic!("No adic number uncertainty found; infinite n-adic digits");
        };

        let digit_vec = match self.min_index() {
            Valuation::PosInf => vec![],
            Valuation::Finite(mi) if certainty < mi => vec![],
            Valuation::Finite(mi) => {

                let significance = (certainty - mi);
                let sigusize = significance.try_into_usize().expect("convert valuation to usize");

                let mut n_adic_pieces = vec![];
                for (p, ap) in &self.pow_adics {

                    let base_wo_p = base.without_p(*p);
                    let base_wo_p = usize::try_from(base_wo_p).expect("composite conversion -> usize");
                    let base_wo_p = Self::DigitIndex::try_from_usize(base_wo_p).expect("convert usize to valuation");
                    let pow32 = ap.power();
                    let powval = ap.power_valuation();
                    let num_digits = significance * base_wo_p * powval;
                    let nd_usize = num_digits.try_into_usize().expect("convert valuation to usize");

                    let adjusted_digits = sandwich_with_zeros(ap);
                    let adjusted_ap = PowAdic::new(ZAdic::new_approx(*p, nd_usize, adjusted_digits.collect()), pow32);

                    let b_p = MAdicDigits::from_p_adic(&base, adjusted_ap)
                        .expect("problem converting from p-adic to n-adic");

                    // Multiply by the correct idempotent
                    // This is the product of all prime idempotents EXCEPT the one for p
                    let all_but_p_idempotent = MAdicDigits::idempotent_excluding(*p, &base, sigusize)
                        .expect("problem calculating idempotent");

                    n_adic_pieces.push((b_p * all_but_p_idempotent).into_truncation(sigusize));

                }

                let full_n_adic = n_adic_pieces.into_iter()
                    .reduce(MAdicDigits::add)
                    .unwrap_or(MAdicDigits::zero(base, sigusize))
                    .into_truncation(sigusize);
                full_n_adic.digits().collect::<Vec<_>>()

            }
        };

        Either::Right(digit_vec.into_iter())

    }

}


impl<A> CanTruncate for PowAdic<A>
where
A: HasApproximateDigits + CanTruncate + AdicPrimitive,
A::Quotient: HasApproximateDigits + AdicPrimitive,
A::Truncation: HasApproximateDigits + AdicPrimitive {
    type Quotient = PowAdic<A::Quotient>;
    type Truncation = PowAdic<A::Truncation>;
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        let adic = &self.adic;
        let power = self.pp.power();
        let power_usize = usize::try_from(power).expect("u32 -> usize conversion");
        let modified_n = n * Self::DigitIndex::try_from_usize(power_usize).expect("usize -> DigitIndex conversion");
        let adic_split = adic.split(modified_n);
        (PowAdic::new(adic_split.0, power), PowAdic::new(adic_split.1, power))
    }
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
        let adic = self.adic;
        let power = self.pp.power();
        let power_usize = usize::try_from(power).expect("u32 -> usize conversion");
        let modified_n = n * Self::DigitIndex::try_from_usize(power_usize).expect("usize -> DigitIndex conversion");
        let adic_split = adic.into_split(modified_n);
        (PowAdic::new(adic_split.0, power), PowAdic::new(adic_split.1, power))
    }
}


// Maybe we can implement CanTruncate, something like below.
// But currently, we don't really handle MAdic in a way that would make this work.
// It's probably fine if we can't truncate MAdic, just like it's fine we can't get its size.
//
// impl<A> CanTruncate for MAdic<A>
// where
// A: HasApproximateDigits + CanTruncate + AdicPrimitive,
// A::Quotient: HasApproximateDigits + AdicPrimitive,
// A::Truncation: HasApproximateDigits + AdicPrimitive {
//     type Quotient = MAdic<A::Quotient>;
//     type Truncation = MAdic<A::Truncation>;
//     fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
//         let base = self.base();
//         let new_pow_adics = self.pow_adics.values().map(|ap| {
//             let log_base = base.log_ceil(&ap.p_pow().into());
//             let log_base = usize::try_from(log_base).expect("convert u32 -> usize");
//             let log_base = A::DigitIndex::try_from_usize(log_base).expect("convert usize -> valuation ring");
//             let adjusted_n = n * log_base;
//             ap.split(adjusted_n)
//         });
//         let (quotient_pow_adics, truncation_pow_adics): (Vec<_>, Vec<_>) = new_pow_adics.unzip();
//         (MAdic::new(quotient_pow_adics), MAdic::new(truncation_pow_adics))
//     }
//     fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient) {
//         let base = self.base();
//         let new_pow_adics = self.pow_adics.into_values().map(|ap| {
//             let log_base = base.log_ceil(&ap.p_pow().into());
//             let log_base = usize::try_from(log_base).expect("convert u32 -> usize");
//             let log_base = A::DigitIndex::try_from_usize(log_base).expect("convert usize -> valuation ring");
//             let adjusted_n = n * log_base;
//             ap.into_split(adjusted_n)
//         });
//         let (quotient_pow_adics, truncation_pow_adics): (Vec<_>, Vec<_>) = new_pow_adics.unzip();
//         (MAdic::new(quotient_pow_adics), MAdic::new(truncation_pow_adics))
//     }
// }


impl<A> HasApproximateDigits for PowAdic<A>
where A: HasApproximateDigits + AdicPrimitive {
    fn certainty(&self) -> Valuation<A::DigitIndex> {
        match (self.adic.certainty(), self.adic.min_index()) {
            (Valuation::Finite(c), Valuation::Finite(mi)) => {
                Valuation::Finite((c - mi).div_euclid(&self.power_valuation()) + mi.div_euclid(&self.power_valuation()))
            },
            _ => Valuation::PosInf
        }
    }
}


impl<A> HasApproximateDigits for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive {
    fn certainty(&self) -> Valuation<A::DigitIndex> {
        let base = self.base();
        self.pow_adics.values().map(|ap| {
            if let Valuation::Finite(nd) = ap.adic_ref().certainty() {
                let log_base = base.log_ceil(&ap.p_pow().into());
                let log_base = usize::try_from(log_base).expect("convert u32 -> usize");
                let log_base = A::DigitIndex::try_from_usize(log_base).expect("convert usize -> valuation ring");
                let cert = nd / log_base;
                cert.into()
            } else {
                Valuation::PosInf
            }
        }).min().unwrap_or(Valuation::PosInf)
    }
}


impl<A> CanApproximate for PowAdic<A>
where A: HasApproximateDigits + CanApproximate + AdicPrimitive, A::Approximation: AdicPrimitive {
    type Approximation = PowAdic<A::Approximation>;
    fn approximation(&self, n: Self::DigitIndex) -> PowAdic<A::Approximation> {
        let c = match self.certainty() {
            Valuation::PosInf => n,
            Valuation::Finite(v) => std::cmp::min(v, n),
        };
        let adic = &self.adic;
        let power = self.pp.power();
        let power_usize = usize::try_from(power).expect("u32 -> usize conversion");
        let modified_c = c * Self::DigitIndex::try_from_usize(power_usize).expect("usize -> DigitIndex conversion");
        PowAdic::new(adic.approximation(modified_c), power)
    }
    fn into_approximation(self, n: Self::DigitIndex) -> PowAdic<A::Approximation> {
        let c = match self.certainty() {
            Valuation::PosInf => n,
            Valuation::Finite(v) => std::cmp::min(v, n),
        };
        let adic = self.adic;
        let power = self.pp.power();
        let power_usize = usize::try_from(power).expect("u32 -> usize conversion");
        let modified_c = c * Self::DigitIndex::try_from_usize(power_usize).expect("usize -> DigitIndex conversion");
        PowAdic::new(adic.into_approximation(modified_c), power)
    }
}


impl<A> CanApproximate for MAdic<A>
where A: HasApproximateDigits + CanApproximate + AdicPrimitive, A::Approximation: AdicPrimitive {
    type Approximation = MAdic<A::Approximation>;
    fn approximation(&self, n: Self::DigitIndex) -> Self::Approximation {
        let base = self.base();
        let new_pow_adics = self.pow_adics.values().map(|ap| {
            let log_base = base.log_ceil(&ap.p_pow().into());
            let log_base = usize::try_from(log_base).expect("convert u32 -> usize");
            let log_base = A::DigitIndex::try_from_usize(log_base).expect("convert usize -> valuation ring");
            let adjusted_c = n * log_base;
            ap.approximation(adjusted_c)
        });
        MAdic::new(new_pow_adics)
    }
    fn into_approximation(self, n: Self::DigitIndex) -> Self::Approximation {
        let base = self.base();
        let new_pow_adics = self.pow_adics.into_values().map(|ap| {
            let log_base = base.log_ceil(&ap.p_pow().into());
            let log_base = usize::try_from(log_base).expect("convert u32 -> usize");
            let log_base = A::DigitIndex::try_from_usize(log_base).expect("convert usize -> valuation ring");
            let adjusted_c = n * log_base;
            ap.into_approximation(adjusted_c)
        });
        MAdic::new(new_pow_adics)
    }
}



impl HasDigitDisplay for PowAdic<UAdic> {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        let pp = self.p_pow();
        let digits = self.digits().map(|d| pp.display_digit(d)).collect::<Vec<_>>();
        digits.into_iter().rev().join("")
    }
}

impl HasDigitDisplay for PowAdic<IAdic> {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
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

impl HasDigitDisplay for PowAdic<RAdic> {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {

        let pp = self.p_pow();
        let fix_d = self.adic.fixed_digits().collect::<Vec<_>>();
        let rep_d = self.adic.repeat_digits().collect::<Vec<_>>();
        if rep_d.is_empty() {
            return PowAdic::new(UAdic::new(self.p(), fix_d), self.power()).digit_display();
        }

        // Start at adjusted min_index, need to preload some zeros
        let pusize = self.power_usize();
        let num_zeros = match self.adic.min_index() {
            Valuation::Finite(adic_min) => adic_min.rem_euclid(pusize),
            Valuation::PosInf => 0,
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

impl HasDigitDisplay for PowAdic<EAdic> {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        match self.adic.raw() {
            IntegerVariant::Unsigned(u) => PowAdic::new(u.clone(), self.power()).digit_display(),
            IntegerVariant::Signed(i) => PowAdic::new(i.clone(), self.power()).digit_display(),
            IntegerVariant::Rational(r) => PowAdic::new(r.clone(), self.power()).digit_display(),
        }
    }
}

impl HasDigitDisplay for PowAdic<ZAdic> {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        let pp = self.p_pow();
        let z = self.adic_ref();
        match z.exact_variant_or_certainty() {
            Either::Left(var) => PowAdic::new(var.clone(), self.power()).digit_display(),
            Either::Right(c) => {
                let adjusted_cert = c / self.power_usize();
                let ds = self.digits().chain(repeat(0)).take(adjusted_cert).map(|d| pp.display_digit(d)).collect::<Vec<_>>();
                let digits = ds.into_iter().rev().join("");
                format!("...{digits}")
            },
        }
    }
}


impl<A> HasDigitDisplay for MAdic<A>
where Self: HasDigits, A: HasApproximateDigits + AdicPrimitive {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        let b = self.base();
        let ds = self.digits().map(|d| b.display_digit(d)).collect::<Vec<_>>();
        let digits = ds.into_iter().rev().join("");
        if self.is_certain() {
            if digits.is_empty() {
                "0".to_string()
            } else {
                digits
            }
        } else {
            format!("...{digits}")
        }
    }
}


impl<A> fmt::Display for PowAdic<A>
where A: AdicInteger, PowAdic<A>: HasDigitDisplay<DigitDisplay = String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pp = self.p_pow();
        let pp32 = u32::from(pp);
        let digits = self.digit_display();
        if digits.is_empty() {
            let zero = pp.display_zero();
            write!(f, "{zero}._{pp32}")
        } else {
            write!(f, "{digits}._{pp32}")
        }
    }
}


impl<A> fmt::Display for PowAdic<QAdic<A>>
where A: CanApproximate + AdicInteger, PowAdic<A>: HasDigitDisplay<DigitDisplay = String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pp = self.p_pow();
        let pp32 = u32::from(pp);
        match self.min_index() {
            Valuation::PosInf => {
                let zero = pp.display_zero();
                write!(f, "{zero}._{pp32}")
            },
            Valuation::Finite(mi) => {
                let num_frac = mi.unsigned_abs();
                let power = self.power();
                let frac_digits = self.digits().chain(repeat(0)).take(num_frac).map(|d| pp.display_digit(d)).collect::<Vec<_>>();
                let frac_str = frac_digits.into_iter().rev().join("");
                let int_power = PowAdic::new(self.adic.frac_and_int().1, power);
                let int_str = int_power.digit_display();
                let int_str = if int_str.is_empty() { self.p_pow().display_zero() } else { int_str };
                write!(f, "{int_str}.{frac_str}_{pp32}")
            }
        }
    }
}


impl<A> fmt::Display for MAdic<A>
where A: HasApproximateDigits + AdicInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.base();
        let b32 = u32::from(b.clone());
        let digits = self.digit_display();
        if digits.is_empty() {
            let zero = b.display_zero();
            write!(f, "{zero}._{b32}")
        } else {
            write!(f, "{digits}._{b32}")
        }
    }
}

impl<A> fmt::Display for MAdic<QAdic<A>>
where A: CanApproximate + AdicInteger, MAdic<A>: HasDigitDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = self.base();
        let b32 = u32::from(b.clone());
        match self.min_index() {
            Valuation::PosInf => {
                let zero = b.display_zero();
                write!(f, "{zero}._{b32}")
            },
            Valuation::Finite(mi) => {
                let num_frac = mi.unsigned_abs();
                let (frac_digits, int_digits) = self.digits()
                    .map(|d| b.display_digit(d))
                    .enumerate()
                    .partition::<Vec<_>, _>(|(i, _)| *i < num_frac);
                let frac_str = frac_digits.into_iter().map(|(_, d)| d).rev().join("");
                let mut int_str = int_digits.into_iter().map(|(_, d)| d).rev().join("");
                if int_str.is_empty() {
                    int_str = "0".to_string();
                }
                if self.is_certain() {
                    write!(f, "{int_str}.{frac_str}_{b32}")
                } else {
                    write!(f, "...{int_str}.{frac_str}_{b32}")
                }
            }
        }
    }
}



fn sandwich_with_zeros<A>(adic_power: &PowAdic<A>) -> impl Iterator<Item=u32> + '_
where A: HasApproximateDigits + AdicPrimitive {

    let pusize = adic_power.power_usize();
    let pval = adic_power.power_valuation();
    let min_index = adic_power.adic.min_index();
    let num_digits = adic_power.adic.num_digits();
    let certainty = adic_power.adic.certainty();

    // Start at adjusted min_index, need to preload and postload some zeros
    let num_zeros_before = match min_index {
        Valuation::Finite(adic_min) => {
            let min_rem = adic_min.rem_euclid(&pval);
            min_rem.try_into_usize().expect("convert valuation to usize")
        },
        Valuation::PosInf => 0,
    };
    let num_zeros_after = match num_digits {
        Valuation::Finite(nd) => pusize - 1 - (num_zeros_before + nd + 1).rem_euclid(pusize),
        Valuation::PosInf => 0,
    };

    let adjusted_digits = repeat_n(0, num_zeros_before)
        .chain(adic_power.adic.digits())
        .chain(repeat_n(0, num_zeros_after));

    // Truncate the iterator if it goes past certainty
    match (certainty, min_index) {
        (Valuation::Finite(c), Valuation::Finite(mi)) if c > mi => {
            let cdiff = (c - mi).try_into_usize().expect("convert valuation to usize");
            Either::Left(adjusted_digits.take(num_zeros_before + cdiff))
        },
        _ => Either::Right(adjusted_digits)
    }

}


#[cfg(test)]
mod tests {

    use crate::{
        normed::Valuation,
        num_adic::{MAdic, PowAdic},
        traits::{AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigits, PrimedFrom},
        EAdic, QAdic, UAdic, ZAdic,
    };

    #[test]
    fn display_adic_power() {

        let three_adic = uadic!(3, [1, 0, 2, 0, 0, 1, 1, 1, 2, 1, 0, 2]);
        assert_eq!("201211100201._3", three_adic.to_string());
        let nine_adic = PowAdic::new(three_adic.clone(), 2);
        assert_eq!("654321._9", nine_adic.to_string());
        let qnine_adic = PowAdic::new(qadic!(three_adic, -3), 2);
        assert_eq!("21740.63_9", qnine_adic.to_string());

        let three_adic = eadic_neg!(3, [1, 0, 2, 0, 0, 1, 1, 1, 2, 1, 0, 2, 1]);
        assert_eq!("(2)1201211100201._3", three_adic.to_string());
        let nine_adic = PowAdic::new(three_adic.clone(), 2);
        assert_eq!("(8)7654321._9", nine_adic.to_string());
        let qnine_adic = PowAdic::new(qadic!(three_adic, -3), 2);
        assert_eq!("(8)51740.63_9", qnine_adic.to_string());

        let three_adic = eadic_rep!(3, [1, 0, 2], [0, 1, 1, 1]);
        assert_eq!("(1110)201._3", three_adic.to_string());
        let nine_adic = PowAdic::new(three_adic.clone(), 2);
        assert_eq!("(14)21._9", nine_adic.to_string());
        let qnine_adic = PowAdic::new(qadic!(three_adic, -3), 2);
        assert_eq!("(43).63_9", qnine_adic.to_string());

        let three_adic = zadic_approx!(3, 13, [1, 0, 2, 0, 0, 1, 1, 1, 2, 1, 0, 2, 1]);
        assert_eq!("...1201211100201._3", three_adic.to_string());
        let nine_adic = PowAdic::new(three_adic.clone(), 2);
        assert_eq!("...654321._9", nine_adic.to_string());
        let qnine_adic = PowAdic::new(qadic!(three_adic, -3), 2);
        assert_eq!("...51740.63_9", qnine_adic.to_string());

        assert_eq!("0._9", PowAdic::new(UAdic::zero(3), 2).to_string());
        assert_eq!("0._9", PowAdic::new(QAdic::<UAdic>::zero(3), 2).to_string());
        assert_eq!("0.12_9", PowAdic::new(qadic!(uadic!(3, [2, 0, 1, 0]), -4), 2).to_string());
        assert_eq!("0.012_9", PowAdic::new(qadic!(uadic!(3, [2, 0, 1, 0]), -6), 2).to_string());

        assert_eq!("nb._25", apow!(uadic!(5, [1, 2, 3, 4]), 2).to_string());
        assert_eq!("[31].[15]_49", apow!(qadic!(uadic!(7, [1, 2, 3, 4]), -2), 2).to_string());
        assert_eq!("(o)nb._25", apow!(eadic_neg!(5, [1, 2, 3, 4]), 2).to_string());
        assert_eq!("([48])[31].[15]_49", apow!(qadic!(eadic_neg!(7, [1, 2, 3, 4]), -2), 2).to_string());
        assert_eq!("(a9)nb._25", apow!(eadic_rep!(5, [1, 2, 3, 4], [4, 1, 0, 2]), 2).to_string());
        assert_eq!("([14][11])[31].[15]_49", apow!(qadic!(eadic_rep!(7, [1, 2, 3, 4], [4, 1, 0, 2]), -2), 2).to_string());
        assert_eq!("(o)nb._25", apow!(ZAdic::from(eadic_neg!(5, [1, 2, 3, 4])), 2).to_string());
        assert_eq!("...nb._25", apow!(zadic_approx!(5, 4, [1, 2, 3, 4]), 2).to_string());
        assert_eq!("...[31].[15]_49", apow!(qadic!(zadic_approx!(7, 4, [1, 2, 3, 4]), -2), 2).to_string());
        assert_eq!("...00000000000000000000.00001_2", PowAdic::new(qadic!(zadic_approx!(2, 25, [1]), -5), 1).to_string());
        assert_eq!("...0000000000.02_5", PowAdic::new(qadic!(zadic_approx!(5, 12, [2]), -2), 1).to_string());

    }

    #[test]
    fn display_adic_composite() {

        let ac = MAdic::approx_from_i32(10, 1, 6).unwrap();
        assert_eq!("...000001._10".to_string(), ac.to_string());
        let ac = MAdic::approx_from_i32(10, 2, 6).unwrap();
        assert_eq!("...000002._10".to_string(), ac.to_string());
        let ac = MAdic::approx_from_i32(10, 5, 6).unwrap();
        assert_eq!("...000005._10".to_string(), ac.to_string());
        let ac = MAdic::approx_from_i32(10, 100, 6).unwrap();
        assert_eq!("...000100._10".to_string(), ac.to_string());
        let ac = MAdic::approx_from_i32(10, -1, 6).unwrap();
        assert_eq!("...999999._10".to_string(), ac.to_string());
        let ac = MAdic::approx_from_i32(10, -100, 6).unwrap();
        assert_eq!("...999900._10".to_string(), ac.to_string());
        let ac = MAdic::approx_from_i32(36, -1, 6).unwrap();
        assert_eq!("...zzzzzz._36".to_string(), ac.to_string());
        let ac = MAdic::approx_from_i32(37, -1, 6).unwrap();
        assert_eq!("...[36][36][36][36][36][36]._37".to_string(), ac.to_string());

        let ac = MAdic::new([
            PowAdic::new(zadic_approx!(2, 25, [1]), 1),
            PowAdic::new(zadic_approx!(5, 12, [2]), 1),
        ]);
        assert_eq!("...109377._10", ac.to_string());
        let ac = MAdic::new([
            PowAdic::new(qadic!(zadic_approx!(2, 25, [1]), -5), 1),
            PowAdic::new(qadic!(zadic_approx!(5, 12, [2]), -2), 1),
        ]);
        assert_eq!("...10937.7_10", ac.to_string());

    }

    #[test]
    fn truncation_adic_power() {

        let ap = PowAdic::new(uadic!(5, [1, 2, 3, 4, 0, 1, 2, 3, 4, 0]), 2);
        assert_eq!("4h5nb._25", ap.to_string());
        let (split0, split1) = ap.split(2);
        assert_eq!("nb._25", split0.to_string());
        assert_eq!("4h5._25", split1.to_string());
        let truncation = ap.truncation(2);
        let quotient = ap.quotient(2);
        assert_eq!(split0, truncation);
        assert_eq!(split1, quotient);
        let (split0, split1) = ap.split(6);
        assert_eq!(ap, split0);
        assert_eq!(PowAdic::new(UAdic::zero(5), 2), split1);

        let ap = PowAdic::new(qadic!(uadic!(5, [1, 2, 3, 4, 0, 1, 2, 3, 4, 0]), -3), 2);
        assert_eq!("nb4.h5_25", ap.to_string());
        let (split0, split1) = ap.split(2);
        assert_eq!("b4.h5_25", split0.to_string());
        assert_eq!("n._25", split1.to_string());
        let truncation = ap.truncation(2);
        let quotient = ap.quotient(2);
        assert_eq!(split0, truncation);
        assert_eq!(split1, quotient);
        let (split0, split1) = ap.split(3);
        assert_eq!(ap, split0);
        assert_eq!(PowAdic::new(UAdic::zero(5), 2), split1);

    }

    #[test]
    fn approximation_adic_power() {

        let ap = PowAdic::new(zadic_approx!(5, 7, [0, 1, 2, 3, 4, 0, 1]), 2);
        assert_eq!(Valuation::Finite(3), ap.certainty());
        let approx_ap = ap.approximation(3);
        assert_ne!(ap, approx_ap);
        let approx_ap2 = approx_ap.approximation(3);
        assert_ne!(ap, approx_ap2);
        assert_eq!(approx_ap, approx_ap2);

        let ap = PowAdic::new(EAdic::primed_from(5, -4), 3);
        assert_eq!(Valuation::PosInf, ap.certainty());
        assert_eq!(PowAdic::new(zadic_approx!(5, 9, [1, 4, 4, 4, 4, 4, 4, 4, 4]), 3), ap.approximation(3));
        assert_eq!(vec![121, 124, 124], ap.approximation(3).digits().collect::<Vec::<_>>());
        assert_eq!(PowAdic::new(zadic_approx!(5, 6, [1, 4, 4, 4, 4, 4]), 3), ap.approximation(2));
        assert_eq!(vec![121, 124], ap.approximation(2).digits().collect::<Vec::<_>>());
        assert_eq!(PowAdic::new(zadic_approx!(5, 3, [1, 4, 4]), 3), ap.approximation(1));
        assert_eq!(vec![121], ap.approximation(1).digits().collect::<Vec::<_>>());
        assert_eq!(PowAdic::new(ZAdic::empty(5), 3), ap.approximation(0));
        assert_eq!(None, ap.approximation(0).digits().next());

        let af = qadic!(zadic_approx!(2, 30, [1]), -5);
        let ap = PowAdic::new(af.clone(), 2);
        assert_eq!(Valuation::Finite(12), ap.certainty());
        let af_short = qadic!(zadic_approx!(2, 29, [1]), -5);
        let ap_short = PowAdic::new(af_short.clone(), 2);
        assert_eq!(Valuation::Finite(12), ap.certainty());
        assert_ne!(af, af_short);
        assert_ne!(af.digits().collect::<Vec<_>>(), af_short.digits().collect::<Vec<_>>());
        assert_ne!(ap, ap_short);
        assert_eq!(ap.digits().collect::<Vec<_>>(), ap_short.digits().collect::<Vec<_>>());

    }

    #[test]
    fn approximation_adic_composite() {

        let one_2 = PowAdic::new(EAdic::primed_from(2, 1), 1);
        let two_5 = PowAdic::new(EAdic::primed_from(5, 2), 1);
        let ac = MAdic::new([one_2.clone(), two_5.clone()]);
        assert_eq!(Valuation::PosInf, ac.certainty());
        let approx_ac = ac.approximation(6);
        assert_eq!(MAdic::new([one_2.approximation(24), two_5.approximation(12)]), approx_ac);
        assert_eq!(vec![7, 7, 3, 9, 0, 1], approx_ac.digits().collect::<Vec::<_>>());

        let ac = MAdic::approx_from_i32(10, 321, 6).unwrap();
        assert_eq!(Valuation::Finite(6), ac.certainty());
        let ac_2 = PowAdic::new(EAdic::primed_from(2, 321), 1).approximation(24);
        let ac_5 = PowAdic::new(EAdic::primed_from(5, 321), 1).approximation(12);
        assert_eq!(MAdic::new([ac_2, ac_5]), ac);
        assert_eq!(vec![1, 2, 3, 0, 0, 0], ac.approximation(6).digits().collect::<Vec::<_>>());

    }

}
