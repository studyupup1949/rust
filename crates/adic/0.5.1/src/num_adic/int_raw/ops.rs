use std::{
    cmp::max,
    iter::{once, repeat, repeat_n},
    ops,
};
use itertools::{EitherOrBoth, Itertools};
use num::{
    integer::lcm,
    traits::Pow,
    Integer,
};
use crate::{
    error::validate_matching_p,
    combinatorics::carmichael,
    divisible::{Divisible, Modulus},
    local_num::LocalZero,
    traits::{AdicPrimitive, HasDigits, PrimedFrom},
};
use super::{IAdic, RAdic, Sign, UAdic};



macro_rules! impl_mul_u32 {
    ( $AdicInt:ty ) => {

        impl std::ops::Mul<$AdicInt> for u32 {
            type Output=$AdicInt;
            fn mul(self, adic_int: $AdicInt) -> $AdicInt {
                let p = adic_int.p();
                <$AdicInt>::primed_from(p, self) * adic_int
            }
        }

    };
}



// UAdic

impl ops::Add for UAdic {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {

        // Add the like digits one-by-one
        // Then reduce each to the [0, p) range and handle the carry

        validate_matching_p([self.p, rhs.p]);
        let p = self.p;

        let la = self.d.len();
        let lb = rhs.d.len();

        let mut summed_digits = Vec::with_capacity(std::cmp::max(la, lb) + 1);
        summed_digits.extend(
            self.d.iter()
                .zip_longest(rhs.d.iter())
                .map(|lr| match lr {
                    EitherOrBoth::Both(&da, &db) => da + db,
                    EitherOrBoth::Left(&da) => da,
                    EitherOrBoth::Right(&db) => db,
                })
                .collect::<Vec<_>>()
        );

        let mut carry = 0;
        for digit in &mut summed_digits {
            let bigger_digit = *digit + carry;
            *digit = bigger_digit % p;
            carry = bigger_digit / p;
        }
        while carry > 0 {
            summed_digits.push(carry % p);
            carry = carry / p;
        }

        UAdic::new(p, summed_digits)

    }
}


impl ops::Mul for UAdic {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {

        // Turn b around and "drag it across" longer to create the multiplied digits one-by-one
        // Then reduce each to the [0, p) range and handle the carry

        validate_matching_p([self.p, rhs.p]);
        let p = self.p;

        let la = self.d.len();
        let lb = rhs.d.len();
        if la * lb == 0 {
            return UAdic::zero(p);
        }
        let lt = la + lb - 1;

        // Performance critical here!
        let a_digits = self.d.clone();
        let mut rev_b_digits = rhs.d.clone();
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
            *digit = bigger_digit % p;
            carry = bigger_digit / p;
        }
        while carry > 0 {
            summed_digits.push(carry % p);
            carry = carry / p;
        }

        UAdic::new(p, summed_digits)

    }
}


impl_add_assign_from_add!(UAdic);
impl_mul_assign_from_mul!(UAdic);
impl_pow_by_squaring!(UAdic);
impl_mul_u32!(UAdic);



// IAdic

impl ops::Add for IAdic {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {

        validate_matching_p([self.p, rhs.p]);
        let p = self.p();

        let mut out_pos = true;
        let mut new_digits = Vec::with_capacity(std::cmp::max(self.d.len(), rhs.d.len()) + 1);

        let (s_pos, s_trail) = if matches!(self.sign, Sign::Pos) { (true, 0) } else { (false, p.m1()) };
        let (r_pos, r_trail) = if matches!(rhs.sign, Sign::Pos) { (true, 0) } else { (false, p.m1()) };
        let s_iter = self.d.iter().map(|d| (false, d)).chain(repeat((true, &s_trail)));
        let r_iter = rhs.d.iter().map(|d| (false, d)).chain(repeat((true, &r_trail)));
        let mut carry = false;
        // Add each pair of digits and manage carry
        for ((s_trailing, sd), (r_trailing, rd)) in s_iter.zip(r_iter) {

            // If finished with both sets digits, check final carry and possibly wrap up with a last digit
            if s_trailing && r_trailing {
                if (!s_pos && !r_pos) {
                    out_pos = false;
                    if !carry {
                        new_digits.push(u32::from(p)-2);
                    }
                } else if (s_pos && r_pos) {
                    out_pos = true;
                    if carry {
                        new_digits.push(1);
                    }
                } else {
                    out_pos = carry;
                }
                break;
            }

            // Add digits together with carry and update carry
            let new_d = sd + rd + if carry { 1 } else { 0 };
            if new_d >= p.into() {
                carry = true;
                new_digits.push(new_d - u32::from(p));
            } else {
                carry = false;
                new_digits.push(new_d);
            }

        }

        // Output positive or negative
        if out_pos {
            IAdic::new_pos(p, new_digits)
        } else {
            IAdic::new_neg(p, new_digits)
        }

    }
}


impl ops::Neg for IAdic {
    type Output = Self;
    fn neg(self) -> Self::Output {

        let p = self.p();

        if self.is_local_zero() {
            self.clone()
        } else {

            let mut new_digits = Vec::with_capacity(self.d.len() + 1);
            let smp = self.sign.mod_p(p);
            let mut old_iter = self.d.iter().chain(once(&smp));
            for d in old_iter.by_ref() {
                new_digits.push(p.mod_neg(*d));
                if *d != 0 {
                    break;
                }
            }
            for d in old_iter.by_ref() {
                new_digits.push(p.mod_neg(*d + 1));
            }

            match self.sign {
                Sign::Pos => IAdic::new_neg(p, new_digits),
                Sign::Neg => IAdic::new_pos(p, new_digits),
            }

        }

    }
}


impl ops::Mul for IAdic {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {

        validate_matching_p([self.p, rhs.p]);
        let p = self.p();

        if self.is_local_zero() || rhs.is_local_zero() {
            IAdic::zero(p)
        } else {

            let new_sign = self.sign * rhs.sign;
            let su = self.abs();
            let ru = rhs.abs();
            match new_sign {
                Sign::Pos => IAdic::new_pos(p, su.mul(ru).digits_vec()),
                Sign::Neg => -IAdic::new_pos(p, su.mul(ru).digits_vec()),
            }

        }

    }
}

impl_add_assign_from_add!(IAdic);
impl_sub_from_neg!(IAdic);
impl_sub_assign_from_sub!(IAdic);
impl_mul_assign_from_mul!(IAdic);
impl_pow_by_squaring!(IAdic);
impl_mul_u32!(IAdic);



// RAdic

impl ops::Add for RAdic {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {

        // Two steps:
        // 1 - add finite integers
        // 2 - add repeating digits until it repeats
        //
        // 1
        // Find which adic has finite integer with more digits
        // Fill out the other finite integer with repeats
        // Add those together to get a new finite integer
        // Strip off digits that were carried past and use as a carry for next step
        //
        // 2
        // Add repeating digits one-by-one keeping track of carry
        // Store a vector of length lcm(self.rep_d.len(), rhs.rep_d.len())
        // Look for the digits AND carry to start repeating
        // Once they repeat, add non-repeating digits to finite digits and keep the rest as repeats

        validate_matching_p([self.p, rhs.p]);
        let p = self.p;

        // Get new finite_int with long_finite_int + short_finite_int + short_repeats
        let longer_len = std::cmp::max(self.fix_d.len(), rhs.fix_d.len());
        let a_digits = self.fix_d
            .iter()
            .copied()
            .chain(self.rep_d.iter().copied().cycle())
            .take(longer_len)
            .collect();
        let b_digits = rhs.fix_d
            .iter()
            .copied()
            .chain(rhs.rep_d.iter().copied().cycle())
            .take(longer_len)
            .collect();
        let mut finite_integer_digits = (
            UAdic::new(p, a_digits) + UAdic::new(p, b_digits)
        ).digits_vec();

        // Fill with zeros
        let finite_len = finite_integer_digits.len();
        if finite_len < longer_len {
            finite_integer_digits.extend(repeat_n(0, longer_len - finite_len));
        }

        // Adding may have overshot longer_len, so change the last digits into a carry
        let overshoot = if finite_len > longer_len {
            finite_integer_digits.split_off(longer_len)
        } else {
            vec![]
        };
        let mut carry = overshoot.into_iter().zip(0..).map(|(d, idx)| d * u32::from(p.pow(idx))).sum();

        // Appropriately advance the shorter iterator to match the repeats that have been used
        let a_replen = max(self.rep_d.len(), 1);
        let mut a_repeat_iter = self.rep_d
            .iter()
            .copied()
            .cycle()
            .skip(longer_len - self.fix_d.len())
            .chain(repeat(0));
        let b_replen = max(rhs.rep_d.len(), 1);
        let mut b_repeat_iter = rhs.rep_d
            .iter()
            .copied()
            .cycle()
            .skip(longer_len - rhs.fix_d.len())
            .chain(repeat(0));

        // Calculate last digits of finite_integer and the new repeats, looking for it to stabilize
        let max_cycle_len = lcm(a_replen, b_replen);
        let mut add_buffer: Vec<(u32, u32)> = vec![];
        loop {
            let longer_rep = a_repeat_iter.next().unwrap();
            let shorter_rep = b_repeat_iter.next().unwrap();
            let added = longer_rep + shorter_rep + carry;
            carry = added / p;
            let new_add = (carry, added % p);
            if (
                add_buffer.len() >= max_cycle_len &&
                *add_buffer.get(add_buffer.len() - max_cycle_len).unwrap() == new_add
            ) {
                break;
            }
            add_buffer.push(new_add);
        }

        // Add last digits to finite_integer and make max_cycle_len new repeats
        let leftover_finite = add_buffer.len() - max_cycle_len;
        let mut added_iter = add_buffer.into_iter().map(|(_, d)| d);
        for _ in 0..leftover_finite {
            finite_integer_digits.push(added_iter.next().unwrap());
        }
        let new_repeats = added_iter.collect::<Vec<_>>();

        RAdic::new(p, finite_integer_digits, new_repeats)

    }
}


impl ops::Neg for RAdic {
    type Output = Self;
    fn neg(self) -> Self::Output {

        let p = self.p();

        if self.fix_d.iter().chain(self.rep_d.iter()).all(|d| *d == 0) {

            // If finite_integer is zero and repeats are zero, return zero
            self.clone()

        } else if self.fix_d.iter().all(|d| *d == 0) {

            // If finite_integer is zero, find the first nonzero repeat and turn into finite_integer
            let repeats_len = self.rep_d.len();
            let zeros_len = self.rep_d.iter().take_while(|d| **d == 0).count();
            let new_repeat_order = self.rep_d
                .iter().copied()
                // cycle through repeats
                .cycle()
                // skip until nonzero
                .skip_while(|d| *d == 0)
                // skip one more since that will be pushed to the finite_integer
                .skip(1)
                // take the same number of repeats as before
                .take(repeats_len)
                .collect::<Vec<_>>();
            let first_nonzero = new_repeat_order.last().unwrap();
            let mut new_digits = self.fix_d.clone();
            new_digits.extend(repeat_n(0, zeros_len));
            new_digits.push(p.mod_neg(*first_nonzero));
            let new_repeats = new_repeat_order.into_iter().map(|d| p.mod_neg(d + 1)).collect::<Vec<_>>();
            RAdic::new(p, new_digits, new_repeats)

        } else {

            let mut new_digits = Vec::with_capacity(self.fix_d.len());
            let mut finite_iter = self.fix_d.iter().copied();
            for d in finite_iter.by_ref() {
                new_digits.push(p.mod_neg(d));
                if d != 0 {
                    break;
                }
            }
            for d in finite_iter.by_ref() {
                new_digits.push(p.mod_neg(d + 1));
            }

            let new_repeats = if self.rep_d.is_empty() {
                vec![p.m1()]
            } else {
                self.rep_d.iter().copied().map(|d| p.mod_neg(d + 1)).collect::<Vec<_>>()
            };

            RAdic::new(p, new_digits, new_repeats)

        }

    }
}


impl ops::Mul for RAdic {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {

        // Input
        // a = [d0, d1, ... d(i-1), r0, ... r(m-1), r0, ... r(m-1), r0, ...]
        // b = [e0, e1, ... e(j-1), s0, ... s(n-1), s0, ... s(n-1), s0, ...]
        //
        // Output
        // a * b = o = o_finite + o_ds + o_er + o_rs
        //
        // Pseudocode:
        //
        // Finite part is easy
        //
        // Calculate o_finite = UAdic(d0, ... d(i-1)) * UAdic(e0, ... e(j-1))
        //
        // Finite times repeating is mostly easy
        //
        // Calculate x_ds = UAdic(d0, ... d(i-1)) * UAdic(s0, ... s(n-1))
        // o_ds = p^j * (- x_ds / (p^n-1))
        //      = p^j * (- (x_ds % p^n-1) / (p^n-1) - (x_ds // (p^n-1)))
        //      = p^j * ( (remainder repeating digits) - (integer) )
        //
        // Calculate x_re = UAdic(r0, ... r(m-1)) * UAdic(e0, ... e(j-1))
        // o_re = p^i * (- x_re / (p^m-1))
        //      = p^i * (- (x_re % p^m-1) / (p^m-1) - (x_re // (p^m-1)))
        //      = p^i * ( (remainder repeating digits) - (integer) )
        //
        // All that's left is multiplying the repeating digits, o_rs
        //
        // For this, we need the Euler totient function
        // Or rather, the REDUCED totient function, the Carmichael function
        // See https://en.wikipedia.org/wiki/Carmichael_function
        // Calculate carmichael for the repeat size:
        // h = new_repeat_size = carm((p^m - 1) (p^n - 1))
        // Note that m | h and n | h, each input repeat matches with the output
        //
        // We will be multiplying separately the digits and the geometric series:
        // o_rs = p^(i + j) * UAdic(r0, ... r(m-1)) * (-1 / (p^m-1)) * UAdic(s0, ... s(n-1)) * (-1 / (p^n-1))
        // y = UAdic(r0, ... r(m-1)) * UAdic(s0, ... s(n-1))
        // g = (-1 / (p^m - 1)) * (-1 / (p^n - 1)) - 1
        //   = [1, 0, 0, 1, 0, 0, 1, ...] * [1, 0, 1, 0, 1, 0, 1, ...] - 1
        // o_rs = p^(i+j) * (y + y*g) = p^(i+j) * (y + y * (-f / (p^h-1)))
        // The extra "-1" is so that -1 < g < 0, which guarantees repeating digits
        // Truncate a UAdic multiplication to length h to get f
        // f represents the repeating digits, numerator for the power-h geometric series
        // Then calculate y * f and split into quotient and remainder (by (p^h-1) using the method above)
        //
        // o_rs = p^(i+j) * (y - (y*f // (p^h-1)) + (y*f % (p^h-1)) * (-1 / (p^h-1)))
        //
        // Finally
        // c = o_finite + o_ds + o_re + o_rs

        validate_matching_p([self.p, rhs.p]);
        let p = self.p;

        // Initialize various lengths and digits
        let i = self.fix_d.len();
        let d = UAdic::new(p, self.fix_d.clone());
        let j = rhs.fix_d.len();
        let e = UAdic::new(p, rhs.fix_d.clone());
        let m = self.rep_d.len();
        let r = UAdic::new(p, self.rep_d.clone());
        let n = rhs.rep_d.len();
        let s = UAdic::new(p, rhs.rep_d.clone());

        // Finite digit multiplication
        let o_finite = if i > 0 && j > 0 {
            let int_finite = d.clone() * e.clone();
            RAdic::new(p, int_finite.digits_vec(), vec![])
        } else {
            RAdic::zero(p)
        };

        // self.finite * rhs.repeats
        let o_ds = if i > 0 && n > 0 {
            let x_ds = d.clone() * s.clone();
            // Calculate quotient and remainder w.r.t. (p^n-1)
            let (r_ds, q_ds) = x_ds.pseudo_pn_minus_1_rem_quot(n);
            let int_ds = RAdic::new(p, [vec![0; j], q_ds.digits_vec()].concat(), vec![]);
            let pad_ds = r_ds.digits().chain(repeat(0)).take(n).collect();
            let frac_ds = RAdic::new(p, vec![0; j], pad_ds);
            -int_ds + frac_ds
        } else {
            RAdic::zero(p)
        };

        // self.repeats * rhs.finite
        let o_re = if j > 0 && m > 0 {
            let x_re = r.clone() * e.clone();
            // Calculate quotient and remainder w.r.t. (p^m-1)
            let (r_re, q_re) = x_re.pseudo_pn_minus_1_rem_quot(m);
            let int_re = RAdic::new(p, [vec![0; i], q_re.digits_vec()].concat(), vec![]);
            let pad_re = r_re.digits().chain(repeat(0)).take(m).collect();
            let frac_re = RAdic::new(p, vec![0; i], pad_re);
            -int_re + frac_re
        } else {
            RAdic::zero(p)
        };

        // self.repeats * rhs.repeats
        let o_rs = if m > 0 && n > 0 {

            let m32 = u32::try_from(m).expect("mul usize -> u32 conversion");
            let n32 = u32::try_from(n).expect("mul usize -> u32 conversion");
            let h = carmichael(
                (u64::from(p).pow(m32) - 1) * (u64::from(p).pow(n32) - 1)
            );
            let h = usize::try_from(h).expect("mul u64 -> usize conversion");

            if h > 1000 {
                println!("WARNING: the computation of {self} x {rhs} will take {h} digits...");
            }

            let y_r = UAdic::new(p, r.digits_vec());
            let y_s = UAdic::new(p, s.digits_vec());
            let y_rs = y_r * y_s;
            let g_r = UAdic::new(p, once(1).chain(repeat_n(0, m-1)).cycle().take(h).collect());
            let g_s = UAdic::new(p, once(1).chain(repeat_n(0, n-1)).cycle().take(h).collect());
            let g_rs = g_r * g_s;
            let f_rs = UAdic::new(p, once(0).chain(
                g_rs.digits_vec().into_iter().skip(1).take(h-1)
            ).collect());
            // o_rs = p^(i + j) * (y + (y*f // (p^h-1)) - (y*f % (p^h-1)) * (-1 / (p^h-1)))
            let yf_rs = y_rs.clone() * f_rs;
            // Calculate quotient and remainder w.r.t. (p^h-1)
            let (r_rs, q_rs) = yf_rs.pseudo_pn_minus_1_rem_quot(h);
            let inty_rs = RAdic::new(p, [vec![0; i+j], y_rs.digits_vec()].concat(), vec![]);
            let intq_rs = -RAdic::new(p, [vec![0; i+j], q_rs.digits_vec()].concat(), vec![]);
            let pad_rs = r_rs.digits().chain(repeat(0)).take(h).collect();
            let frac_rs = RAdic::new(p, vec![0; i+j], pad_rs);
            inty_rs + intq_rs + frac_rs

        } else {
            RAdic::zero(p)
        };

        o_finite + o_ds + o_re + o_rs

    }
}


impl_add_assign_from_add!(RAdic);
impl_mul_assign_from_mul!(RAdic);
impl_sub_from_neg!(RAdic);
impl_sub_assign_from_sub!(RAdic);
impl_pow_by_squaring!(RAdic);
impl_mul_u32!(RAdic);
