//! Functions for doing Hensel lift


use num::{BigInt, Rational32};
use num_prime::nt_funcs::is_prime;
use crate::adic_error::AdicError;

/// Newton method for deriving fraction that approximates a variety
/// <div class="warning">
///
/// This method does not scale well; you probably want [variety_to_digits]
///
/// </div>
pub fn variety_to_fraction (p: u32, a: i32, n: u32, precision: u32) -> Vec<Vec<Rational32>> {
    // Calculate term r_k+1
    //
    // r_k+1 = r_k - f(r_k)/f'(r_k)
    // f(r) = r^2 - a
    //
    // Iterate "precision" number of times to ensure we match "precision" digits
    //
    // We also need to do an initial step to determine whether there is a solution. So we need to
    // check whether r_0 is a root in the finite field F_p
    let p = p as i32;
    let mut varieties = Vec::new();

    for num in 0..p {
        let square = num.pow(n);
        if (square - a) % p == 0 {
            varieties.push(num);
        }
    }


    let mut full_varieties = varieties.iter().map(|num| vec![Rational32::new(*num, 1)]).collect::<Vec<_>>();
    for (index, _) in varieties.iter().enumerate() {
        let full_variety = &mut full_varieties[index];
        for _ in 1..precision {
            // r_k+1 = r_k - f(r_k)/f'(r_k)
            // f(r) = r^2 - a
            let term = *full_variety.last().unwrap();
            let n = n as i32;
            full_variety.push(term - (term.pow(n) - a)/(Rational32::new(n,1) * term.pow(n - 1)));
        }
    }

    full_varieties
}


/// Newton method for deriving digits that approximates a variety
///
/// 7-adic sqrt(2) has two solutions, starting with 3 and with 4
/// ```
/// use adic::variety_to_digits;
/// let digits = variety_to_digits(7, 2, 2, 6).unwrap();
/// let expected = vec![vec![3, 1, 2, 6, 1, 2], vec![4, 5, 4, 0, 5, 4]];
/// assert_eq!(expected, digits);
/// ```
///
/// 5-adic sqrt(2) has no solutions, as seen since no element of F_5 has x^2 = 2 mod 5
/// ```
/// use adic::variety_to_digits;
/// let digits = variety_to_digits(5, 2, 2, 6).unwrap();
/// let expected: Vec<Vec<u32>> = vec![];
/// assert_eq!(expected, digits);
/// ```
///
/// Every (p > 2) p-adic has (p-1) roots of unity
/// ```
/// use adic::variety_to_digits;
/// let digits = variety_to_digits(5, 1, 4, 6).unwrap();
/// let num_fourth_roots = 4;
/// let expected = vec![
///     vec![1, 0, 0, 0, 0, 0],
///     vec![2, 1, 2, 1, 3, 4],
///     vec![3, 3, 2, 3, 1, 0],
///     vec![4, 4, 4, 4, 4, 4]
/// ];
/// assert_eq!(num_fourth_roots, digits.len());
/// assert_eq!(expected, digits);
/// ```
///
/// <div class="warning">
///
/// Currently, we handle neither 2-adic numbers nor p-th roots.
/// Both of these cases require special attention.
/// 2-adic numbers are exceptional in many ways, while p-th roots create a factor
///  of p in the Hensel lift that needs processing we do not currently perform.
///
/// </div>
pub fn variety_to_digits (p: u32, a: i32, n: u32, precision: u32) -> Result<Vec<Vec<u32>>, AdicError> {
    // Not implemented for p = 2
    if p == 2 { return Err(AdicError::NotImplemented); }
    // Not implemented for pth roots
    if n % p == 0 { return Err(AdicError::NotImplemented); }
    // Not implemented if p is not prime
    if !is_prime(&p, None).probably() { return Err(AdicError::NotImplemented); }

    // The root of 0 is always 0, but return no solution instead
    if a == 0 { return Ok(vec![]); }

    // If a has factors of p, we need to divide them out before rootfinding
    // Find b and m such that a = b * p^m
    let m = {
        let (mut num, mut count) = (a as u32, 0);
        while num % p == 0 {
            num /= p;
            count += 1;
        }

        count
    };

    // If m is not proportional to n then there are no n-th roots
    if m % n != 0 { return Ok(vec![]); }

    // Otherwise, divide out the powers of p and multiply them back at the end
    // I.e. if a = b * p^m then nth_root(a) = p^(m/n) * nth_root(b)
    let p = p as i32;
    let b = a / p.pow(m);
    let mut varieties = Vec::new();

    // println!("p is {p}, a is {a}, n is {n}, m is {m}, b is {b}");

    // Find the number of varieties in Z_p by looking for the number in F_p
    // TODO: This is only really correct for small n; we need to use the generalized
    //  Hensel lemma to more generally find all varieties
    for num in 0..p {
        let square = num.pow(n);
        if (square - b) % p == 0 {
            varieties.push(num);
        }
    }

    // We will use arbitrary precision integers for the calculation
    // We need to be careful not to do too much with these integers; very expensive
    let mut full_varieties = varieties.iter().map(|num| vec![*num as u32]).collect::<Vec<_>>();
    for (index, _) in varieties.iter().enumerate() {
        let full_variety = &mut full_varieties[index];
        for k in 1..precision {

            // r_k+1 = r_k - f(r_k)/f'(r_k)
            // r_k - b === 0 mod p^(k); r_k - b === c * p^(k) mod p^(k+1)
            // Calculate c = (r_k^2 - b) / p^(k); a number between 0 and p-1 inclusive
            // Calculate next digit d_(k+1) using c + n * r_(k) * d_(k+1) === 0 mod p

            let term = full_variety.iter().enumerate().map(|(index, num)| *num * BigInt::from(p).pow(index as u32)).sum::<BigInt>();
            let c = (term.pow(n) - b) / BigInt::from(p).pow(k);

            for d in 0..p {
                if (&c + (n as i32) * &term.pow((n - 1) as u32) * d) % BigInt::from(p) == BigInt::ZERO {
                    full_variety.push(d as u32);
                }
            }

        }
    }

    let k = m as usize / n as usize;
    Ok(full_varieties.into_iter().map(|variety| {
        let mut new_variety = vec![0; k];
        let len = variety.len();
        new_variety.extend(variety.into_iter().take(len - k));

        new_variety
    }).collect())
}

#[cfg(test)]
mod tests {
    use num::Rational32;
    use crate::adic_error::AdicError;

    use super::variety_to_fraction;
    use super::variety_to_digits;

    #[test]
    fn test_7_adic_sqrt_2_fraction() {
        let expected = vec![
            vec![Rational32::new(3, 1), Rational32::new(11, 6), Rational32::new(193, 132)], 
            vec![Rational32::new(4, 1), Rational32::new(9, 4), Rational32::new(113, 72)],
        ];
        let actual= variety_to_fraction(7, 2, 2, 3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_5_adic_sqrt_2_fraction() {
        let expected: Vec<Vec<Rational32>> = vec![];
        let actual= variety_to_fraction(5, 2, 2, 3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_7_adic_sqrt_2_digits() {
        let expected = vec![
            vec![3, 1, 2],
            vec![4, 5, 4],
        ];
        let actual = variety_to_digits(7, 2, 2, 3);
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_5_adic_sqrt_2_digits() {
        let expected: Vec<Vec<u32>> = vec![];
        let actual= variety_to_digits(5, 2, 2, 3);
        assert_eq!(expected, actual.unwrap());
    }

    #[ignore = "until we add special handling for powers of p in a"]
    #[test]
    fn test_sqrt_p() {

        // sqrt(5^m)
        let expected: Vec<Vec<u32>> = vec![];
        let actual = variety_to_digits(5, 5, 2, 3);
        assert_eq!(expected, actual.unwrap());
        let expected = vec![
            vec![0, 1, 0],
            vec![0, 4, 4],
        ];
        let actual = variety_to_digits(5, 25, 2, 3);
        assert_eq!(expected, actual.unwrap());
        let expected: Vec<Vec<u32>> = vec![];
        let actual = variety_to_digits(5, 125, 2, 3);
        assert_eq!(expected, actual.unwrap());

        // cubert(5^m)
        let expected: Vec<Vec<u32>> = vec![];
        let actual = variety_to_digits(5, 5, 3, 3);
        assert_eq!(expected, actual.unwrap());
        let expected: Vec<Vec<u32>> = vec![];
        let actual = variety_to_digits(5, 25, 3, 3);
        assert_eq!(expected, actual.unwrap());
        let expected = vec![
            vec![0, 1, 0],
            vec![0, 4, 4],
        ];
        let actual = variety_to_digits(5, 125, 3, 3);
        assert_eq!(expected, actual.unwrap());

        let expected = vec![
            vec![0, 3, 1, 2, 6, 1],
            vec![0, 4, 5, 4, 0, 5],
        ];
        let actual = variety_to_digits(7, 98, 2, 6);
        assert_eq!(expected, actual.unwrap());

    }

    #[ignore = "until we add special handling for p-th power roots"]
    #[test]
    fn test_pth_root() {

        // cubert(10) in 3-adics
        let expected: Vec<Vec<_>> = vec![vec![1, 1, 1, 0, 0, 0]];
        let actual = variety_to_digits(3, 10, 3, 6);
        assert_eq!(expected, actual.unwrap());

    }

    #[test]
    fn test_not_implemented() {
        let expected = Err(AdicError::NotImplemented);
        let actual = variety_to_digits(2, 2, 2, 3);
        assert_eq!(expected, actual);

        let actual = variety_to_digits(5, 2, 5, 3);
        assert_eq!(expected, actual);

        let actual = variety_to_digits(4, 2, 2, 3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_a_equals_zero() {
        let expected: Vec<Vec<_>> = vec![];
        let actual = variety_to_digits(5, 0, 2, 3);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_5_adic_sqrt_dne_digits() {
        let expected: Vec<Vec<_>> = vec![];

        let actual = variety_to_digits(5, 5, 2, 3);
        assert_eq!(actual.unwrap(), expected);

        let actual = variety_to_digits(5, 125, 2, 3);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_5_adic_sqrt_exists_digits() {
        let expected: Vec<Vec<_>> = vec![
            vec![0, 1, 0, 0],
            vec![0, 4, 4, 4],
        ];
        let actual = variety_to_digits(5, 25, 2, 4);
        assert_eq!(actual.unwrap(), expected);

        let expected: Vec<Vec<_>> = vec![
            vec![0, 0, 1, 0],
            vec![0, 0, 4, 4],
        ];
        let actual = variety_to_digits(5, 625, 2, 4);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_7_adic_sqrt_exist_digits() {
        let expected: Vec<Vec<_>> = vec![
            vec![0, 3, 1, 2, 6],
            vec![0, 4, 5, 4, 0],
        ];
        let actual = variety_to_digits(7, 98, 2, 5);
        assert_eq!(actual.unwrap(), expected);
    }

}
