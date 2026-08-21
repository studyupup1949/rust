use std::{
    iter::{once, repeat},
    ops,
};
use num::traits::{Inv, Pow};
use itertools::Itertools;
use overload::overload;

use crate::{
    AdicApproximate, AdicError, AdicNumber, AdicPower, AdicResult, AdicValuation,
    Composite, Divisible, HasDigits, Prime, ZAdic,
};


#[derive(Debug, Clone, PartialEq, Eq)]
/// Adic composite digits that can be added and multiplied
pub struct NAdicDigits {
    b: Composite,
    d: Vec<u32>,
}


impl NAdicDigits {

    pub fn new(b: Composite, d: Vec<u32>) -> Self {
        NAdicDigits { b, d }
    }

    pub fn from_p_adic<A>(b: Composite, ap: AdicPower<A>) -> AdicResult<Self>
    where A: AdicNumber, AdicPower<A>: AdicApproximate<DigitIndex = usize> {

        let p = ap.p();
        if !b.has_prime(p) {
            return Err(AdicError::IllDefined(format!(
                "NAdicDigits composite needs to include the prime: {} not in {}",
                ap.p(), b
            )));
        }

        let b_pow = b.prime_power(p);
        let matched_ap = AdicPower::new(ap.adic, b_pow.power());

        let AdicValuation::Finite(certainty) = matched_ap.certainty() else {
            return Err(AdicError::InappropriatePrecision(
                "No adic number uncertainty found; infinite n-adic digits".to_string()
            ));
        };

        // Let's use Horner's method to convert from radix p to radix n
        // 1213_6 = (( 1*6 + 2 )*6 + 1)*6 + 3 => 1*6 + 2 = 10)*6 = 60) + 1) = 61)*6 = 446) + 3 = 451_8
        //
        // Note: There's some efficiency to be gained here, e.g. avoiding cloning the Composite.
        // Consider doing the digit calculation manually, outside of NAdicDigits.
        let old_base = Self::new(b.clone(), vec![b_pow.value32()]);
        let mut nadic_ap = Self::zero(b.clone(), certainty);
        for d in matched_ap.into_digits().collect::<Vec<_>>().into_iter().rev() {
            nadic_ap = nadic_ap * old_base.clone() + Self::new(b.clone(), vec![d]);
        }

        // Multiply by the correct idempotent
        // This is the product of all prime idempotents EXCEPT the one for p
        let all_but_p_idempotent = Self::idempotent_excluding(p, &b, certainty)?;

        Ok((nadic_ap * all_but_p_idempotent).into_truncation(certainty))

    }


    pub fn base(&self) -> &Composite {
        &self.b
    }

    // TODO: Impl HasDigits
    pub fn digits(&self) -> impl Iterator<Item=u32> + use<'_> {
        self.d.iter().copied()
    }

    pub fn into_digits(self) -> impl Iterator<Item=u32> {
        self.d.into_iter()
    }

    pub fn len(&self) -> usize {
        self.d.len()
    }

    pub fn truncation(&self, n: usize) -> Self {
        self.clone().into_truncation(n)
    }

    pub fn into_truncation(self, n: usize) -> Self {
        let mut d = self.d;
        d.truncate(n);
        Self::new(self.b, d)
    }

    pub fn zero(b: Composite, prec: usize) -> Self {
        NAdicDigits { b, d: vec![0; prec] }
    }

    pub fn one(b: Composite, prec: usize) -> Self {
        NAdicDigits { b, d: once(1).chain(repeat(0)).take(prec).collect() }
    }


    // There are idempotents in multi-adics, zero-like and one-like numbers.
    // These satisfy the property T^2 = T.
    // For pure p-adics or p^n-adics, only 0 and 1 satisfy these.
    // For mixed n-adics, there are more.

    // For multi-adics, you can convert down from n-adic to p-adic, a surjective (onto) function.
    // E.g. the 10-adic 523._10 converts to 3 + 2*10 + 5*100 = 3 + 4*5 + 4*5^3 = 4043._5.
    // But again, this is SURJECTIVE, and e.g. multiple numbers convert to 0.
    // In particular, the number T_5 = lim_n->inf 5^(2^n) is a valid 10-adic but converts to the 5-adic 0.
    // This particular number also converts to the 2-adic 1, since 5^(2^n) -> 1 mod 2^m (Fermat/Euler?)
    // And subtracting T_5 from 1 gives the other idempotent: T_2 = 1 - T_5.

    // T_5 = ...59918212890625_10 -> 0._5 & 1._2
    // T_2 = ...40081787109376_10 -> 1._5 & 0._2

    // Note that these have the properties: T_2 * T_5 = 0; T_2 + T_5 = 1

    // In general, we want numbers that behave like 0 for one p-adic and 1 for the others.
    // This way, we get the idempotents:
    // - 0, 1
    // - T_p, for distinct prime p in n
    // - \prod_p T_p excluding the full product, since that gives 0

    // Or perhaps another way to think of it, the product of any number of T_p from the primes:
    // - The empty product gives 1
    // - The single product gives each "prime idempotent" separately, T_p
    // - The composite products multiply the prime idempotents once each, T_p1 * T_p2 * T_p3
    // - The full product gives 0

    // So we just need to calculate the prime idempotents and then multiply them together judiciously.
    // The prime idempotents should just be lim_n->inf (p1)^((p2*p3*...)^n)

    // Or more properly, if the adic has base b = p0^k0 p1^k1 p2^k2 ...
    // T_p0 = lim_N->inf (p0^k0)^carmichael( (p1^k1 p2^k2 ...)^N )


    /// The product of all idempotents except `T_p ~ p^carmichael((n / p^k)^infinity)`.
    pub fn idempotent_excluding<P>(p: P, c: &Composite, precision: usize) -> AdicResult<NAdicDigits>
    where P: Into<Prime> {

        // Actually, carmichael modpow is pretty inefficient.
        // Instead, here's a decently efficient algorithm using p-adic (b/p)^-n * (b/p)^n

        let p = p.into();
        let p_pow = c.prime_power(p).power();
        if p_pow == 0 {
            return Err(AdicError::IllDefined(format!("Prime {p} does not exist in composite {c}")));
        }
        let pow_prec = precision * usize::try_from(p_pow)?;

        let bval = c.value32();
        let b_wo_pp = Composite::new(c.prime_powers().filter(|qq| qq.p() != p)).value32();

        // Idea:
        //  Calculate the p-adic (b/p)^(-precision)
        //  Multiply (b/p) back in as an AdicComposite

        // z = (b/p)
        let z = ZAdic::from_u32(p, b_wo_pp);
        // r = p-adic z^(-1)
        let r = z.inv().zapprox(pow_prec)?;
        // rpow = p-power-adic z^(-precision)
        let rpow = r.pow(precision.try_into()?);
        let rpow = AdicPower::new(rpow, p_pow);

        let mut a = Vec::new();
        for d in rpow.into_digits() {
            a.push(d);
            let mut carry = 0;
            for da in &mut a {
                let new_d = b_wo_pp * (*da) + carry;
                carry = new_d / bval;
                *da = new_d % bval;
            }
        }

        Ok(Self::new(c.clone(), a))

    }

    /// The prime idempotent `T_p ~ p^carmichael((n / p^k)^infinity)`.
    pub fn prime_idempotent<P>(p: P, c: &Composite, precision: usize) -> AdicResult<NAdicDigits>
    where P: Into<Prime> {

        // Prime idempotent is 1 - idempotent without prime
        let nadic = Self::idempotent_excluding(p, c, precision)?;
        let mut a = nadic.into_digits();
        let first = a.next();
        let Some(f) = first else {
            return Ok(Self::zero(c.clone(), 0));
        };
        if [0, 1].contains(&f) {
            return Err(AdicError::Severe("idempotent should not start with 0 or 1; something is wrong".to_string()));
        }

        let mut b = vec![];
        b.push(c.value32() + 1 - f);
        for d in a {
            b.push(c.m1() - d);
        }

        Ok(Self::new(c.clone(), b))

    }

}


overload!((a: ?NAdicDigits) + (b: ?NAdicDigits) -> NAdicDigits {

    // Add the like digits one-by-one
    // Then reduce each to the [0, base) range and handle the carry

    assert_eq!(a.b, b.b, "Cannot add NAdicDigits with different bases");
    let bval = a.b.value32();

    let la = a.d.len();
    let lb = b.d.len();

    let mut summed_digits = Vec::with_capacity(std::cmp::max(la, lb) + 1);
    summed_digits.extend(
        a.d.iter().copied()
            .zip_longest(b.d.iter().copied())
            .map(|lr| lr.reduce(|l, r| l+r))
            .collect::<Vec<_>>()
    );

    let mut carry = 0;
    for digit in &mut summed_digits {
        let bigger_digit = *digit + carry;
        *digit = bigger_digit % bval;
        carry = bigger_digit / bval;
    }
    while carry > 0 {
        summed_digits.push(carry % bval);
        carry = carry / bval;
    }

    NAdicDigits::new(a.b.clone(), summed_digits)

});


overload!((a: ?NAdicDigits) * (b: ?NAdicDigits) -> NAdicDigits {

    // Turn b around and "drag it across" longer to create the multiplied digits one-by-one
    // Then reduce each to the [0, base) range and handle the carry

    assert_eq!(a.b, b.b, "Cannot mul NAdicDigits with different bases");
    let bval = a.b.value32();

    let la = a.d.len();
    let lb = b.d.len();
    if la * lb == 0 {
        return NAdicDigits { b: a.b.clone(), d: vec![] };
    }
    let lt = la + lb - 1;

    // Performance critical here!
    let a_digits = a.d.clone();
    let mut rev_b_digits = b.d.clone();
    rev_b_digits.reverse();
    let mut summed_digits = Vec::with_capacity(lt + 1);
    summed_digits.extend((0..lt).map(|digit_place| {
        let (a_skip, b_skip) = if (digit_place >= lb) {
            (digit_place + 1 - lb, 0)
        } else {
            (0, lb - digit_place - 1)
        };
        let mut d = 0;
        for (&ds, &dr) in a_digits[a_skip..].iter().zip(rev_b_digits[b_skip..].iter()) {
            d += ds * dr;
        }
        d
    }));

    let mut carry = 0;
    for digit in &mut summed_digits {
        let bigger_digit = *digit + carry;
        *digit = bigger_digit % bval;
        carry = bigger_digit / bval;
    }
    while carry > 0 {
        summed_digits.push(carry % bval);
        carry = carry / bval;
    }

    NAdicDigits::new(a.b.clone(), summed_digits)

});



#[cfg(test)]
mod tests {

    use crate::Composite;

    use super::NAdicDigits;


    #[test]
    fn two_primes() {

        let b = 10;
        let c = Composite::try_from(b).unwrap();
        let prec = 10;

        let two_to_inf = NAdicDigits::new(c.clone(), vec![6, 7, 3, 9, 0, 1, 7, 8, 7, 1]);
        let five_to_inf = NAdicDigits::new(c.clone(), vec![5, 2, 6, 0, 9, 8, 2, 1, 2, 8]);
        assert_eq!(Ok(two_to_inf.clone()), NAdicDigits::prime_idempotent(2, &c, prec));
        assert_eq!(Ok(five_to_inf.clone()), NAdicDigits::prime_idempotent(5, &c, prec));
        assert_eq!(Ok(two_to_inf.clone()), NAdicDigits::idempotent_excluding(5, &c, prec));
        assert_eq!(Ok(five_to_inf.clone()), NAdicDigits::idempotent_excluding(2, &c, prec));

        let zero = NAdicDigits::zero(c.clone(), prec);
        assert_eq!(zero, (two_to_inf * five_to_inf).into_truncation(prec));

    }

    #[test]
    fn three_primes() {

        let b = 30;
        let c = Composite::try_from(b).unwrap();
        let prec = 10;

        let two_to_inf = NAdicDigits::new(c.clone(), vec![16, 22, 3, 28, 15, 14, 26, 14, 23, 2]);
        let three_to_inf = NAdicDigits::new(c.clone(), vec![21, 26, 28, 12, 5, 22, 29, 26, 27, 5]);
        let five_to_inf = NAdicDigits::new(c.clone(), vec![25, 10, 27, 18, 8, 23, 3, 18, 8, 21]);
        assert_eq!(Ok(two_to_inf.clone()), NAdicDigits::prime_idempotent(2, &c, prec));
        assert_eq!(Ok(three_to_inf.clone()), NAdicDigits::prime_idempotent(3, &c, prec));
        assert_eq!(Ok(five_to_inf.clone()), NAdicDigits::prime_idempotent(5, &c, prec));

        let six_to_inf = NAdicDigits::new(c.clone(), vec![6, 19, 2, 11, 21, 6, 26, 11, 21, 8]);
        let ten_to_inf = NAdicDigits::new(c.clone(), vec![10, 3, 1, 17, 24, 7, 0, 3, 2, 24]);
        let fifteen_to_inf = NAdicDigits::new(c.clone(), vec![15, 7, 26, 1, 14, 15, 3, 15, 6, 27]);
        assert_eq!(Ok(six_to_inf.clone()), NAdicDigits::idempotent_excluding(5, &c, prec));
        assert_eq!(six_to_inf, (two_to_inf.clone() * three_to_inf.clone()).into_truncation(prec));
        assert_eq!(Ok(ten_to_inf.clone()), NAdicDigits::idempotent_excluding(3, &c, prec));
        assert_eq!(ten_to_inf, (two_to_inf.clone() * five_to_inf.clone()).into_truncation(prec));
        assert_eq!(Ok(fifteen_to_inf.clone()), NAdicDigits::idempotent_excluding(2, &c, prec));
        assert_eq!(fifteen_to_inf, (three_to_inf.clone() * five_to_inf.clone()).into_truncation(prec));

        assert_eq!(
            NAdicDigits::one(c.clone(), prec),
            (
                NAdicDigits::prime_idempotent(2, &c, prec).unwrap() +
                NAdicDigits::idempotent_excluding(2, &c, prec).unwrap()
            ).into_truncation(prec)
        );
        assert_eq!(
            NAdicDigits::one(c.clone(), prec),
            (
                NAdicDigits::prime_idempotent(3, &c, prec).unwrap() +
                NAdicDigits::idempotent_excluding(3, &c, prec).unwrap()
            ).into_truncation(prec)
        );
        assert_eq!(
            NAdicDigits::one(c.clone(), prec),
            (
                NAdicDigits::prime_idempotent(5, &c, prec).unwrap() +
                NAdicDigits::idempotent_excluding(5, &c, prec).unwrap()
            ).into_truncation(prec)
        );

        assert_eq!(
            NAdicDigits::zero(c.clone(), prec),
            (two_to_inf * three_to_inf * five_to_inf).into_truncation(prec)
        );

    }

    #[test]
    fn prime_power() {

        let b = 20;
        let c = Composite::try_from(b).unwrap();
        let prec = 10;

        let two_to_inf = NAdicDigits::new(c.clone(), vec![16, 8, 13, 8, 9, 18, 17, 18, 6, 12]);
        let five_to_inf = NAdicDigits::new(c.clone(), vec![5, 11, 6, 11, 10, 1, 2, 1, 13, 7]);
        assert_eq!(Ok(two_to_inf), NAdicDigits::idempotent_excluding(5, &c, prec));
        assert_eq!(Ok(five_to_inf), NAdicDigits::idempotent_excluding(2, &c, prec));

    }

    #[test]
    fn big_idempotent() {
        let b = 10;
        let c = Composite::try_from(b).unwrap();
        assert_eq!(10, NAdicDigits::prime_idempotent(2, &c, 10).unwrap().len());
        assert_eq!(100, NAdicDigits::prime_idempotent(2, &c, 100).unwrap().len());
        assert_eq!(1000, NAdicDigits::prime_idempotent(2, &c, 1000).unwrap().len());
        // Too slow:
        // assert_eq!(10000, NAdicDigits::prime_idempotent(2, &c, 10000).unwrap().len());
    }

}
