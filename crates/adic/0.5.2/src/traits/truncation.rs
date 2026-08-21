use super::{HasDigits, HasApproximateDigits};



/// Can be truncated into `Truncation`
pub trait CanTruncate: HasDigits {

    /// Output of quotient
    type Quotient: HasDigits;

    /// Output of truncation
    type Truncation: HasDigits;

    /// Split adic into digits [0, n) and [n, ...).
    /// This splits the number into remainder and quotient.
    /// See also: [`into_split`](Self::into_split)
    ///
    /// `a = (a % p^n) + p^n * (a // p^n)`
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, EAdic, QAdic, UAdic, ZAdic};
    /// assert_eq!((UAdic::new(5, vec![1, 2]), UAdic::new(5, vec![3, 4])), UAdic::new(5, vec![1, 2, 3, 4]).split(2));
    /// assert_eq!((UAdic::new(5, vec![1, 2, 4, 4]), EAdic::new_neg(5, vec![])), EAdic::new_neg(5, vec![1, 2]).split(4));
    /// assert_eq!(
    ///     (UAdic::new(7, vec![1, 2, 3, 4, 5, 3]), EAdic::new_repeating(7, vec![], vec![4, 5, 3])),
    ///     EAdic::new_repeating(7, vec![1, 2], vec![3, 4, 5]).split(6)
    /// );
    /// assert_eq!(
    ///     (UAdic::new(5, vec![1, 2]), ZAdic::new_approx(5, 2, vec![3, 4])),
    ///     ZAdic::new_approx(5, 4, vec![1, 2, 3, 4]).split(2)
    /// );
    ///
    /// let r = EAdic::new_repeating(7, vec![1, 2], vec![3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = QAdic::new(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_rem, q_int) = q.split(1);
    /// assert_eq!("4.354321_7", q_rem.to_string());
    /// assert_eq!("(435)._7", q_int.to_string());
    /// let (q_rem, q_int) = q.split(-5);
    /// assert_eq!("0.000001_7", q_rem.to_string());
    /// assert_eq!("(543)2._7", q_int.to_string());
    /// let (q_rem, q_int) = q.split(-7);
    /// assert_eq!("0._7", q_rem.to_string());
    /// assert_eq!("(543)210._7", q_int.to_string());
    /// ```
    ///
    /// Note especially the approximate behavior.
    /// E.g. splitting an integer past certainty gives a `UAdic` and an empty `ZAdic`!
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, UAdic, ZAdic};
    /// assert_eq!(
    ///     (UAdic::new(5, vec![1, 2]), ZAdic::new_approx(5, 0, vec![])),
    ///     ZAdic::new_approx(5, 2, vec![1, 2]).split(4)
    /// );
    /// ```
    fn split(&self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient);

    /// Split adic into digits [0, n) and [n, ...).
    /// This splits the number into remainder and quotient.
    /// See also: [`split`](Self::split)
    ///
    /// `a = (a % p^n) + p^n * (a // p^n)`
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, EAdic, QAdic, UAdic, ZAdic};
    /// assert_eq!((UAdic::new(5, vec![1, 2]), UAdic::new(5, vec![3, 4])), UAdic::new(5, vec![1, 2, 3, 4]).into_split(2));
    /// assert_eq!((UAdic::new(5, vec![1, 2, 4, 4]), EAdic::new_neg(5, vec![])), EAdic::new_neg(5, vec![1, 2]).into_split(4));
    /// assert_eq!(
    ///     (UAdic::new(7, vec![1, 2, 3, 4, 5, 3]), EAdic::new_repeating(7, vec![], vec![4, 5, 3])),
    ///     EAdic::new_repeating(7, vec![1, 2], vec![3, 4, 5]).into_split(6)
    /// );
    /// assert_eq!(
    ///     (UAdic::new(5, vec![1, 2]), ZAdic::new_approx(5, 2, vec![3, 4])),
    ///     ZAdic::new_approx(5, 4, vec![1, 2, 3, 4]).into_split(2)
    /// );
    ///
    /// let r = EAdic::new_repeating(7, vec![1, 2], vec![3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = QAdic::new(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_rem, q_int) = q.clone().into_split(1);
    /// assert_eq!("4.354321_7", q_rem.to_string());
    /// assert_eq!("(435)._7", q_int.to_string());
    /// let (q_rem, q_int) = q.clone().into_split(-5);
    /// assert_eq!("0.000001_7", q_rem.to_string());
    /// assert_eq!("(543)2._7", q_int.to_string());
    /// let (q_rem, q_int) = q.clone().into_split(-7);
    /// assert_eq!("0._7", q_rem.to_string());
    /// assert_eq!("(543)210._7", q_int.to_string());
    /// ```
    ///
    /// Note especially the approximate behavior: splitting past certainty gives a `UAdic` and an empty `ZAdic`!
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, UAdic, ZAdic};
    /// assert_eq!(
    ///     (UAdic::new(5, vec![1, 2]), ZAdic::new_approx(5, 0, vec![])),
    ///     ZAdic::new_approx(5, 2, vec![1, 2]).into_split(4)
    /// );
    /// ```
    fn into_split(self, n: Self::DigitIndex) -> (Self::Truncation, Self::Quotient);

    /// Truncate an adic number's expansion to n.
    /// This can be thought of as the remainder `a % p^n`.
    /// See also: [`into_truncation`](Self::into_truncation)
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, EAdic, QAdic, UAdic};
    /// let u = UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]);
    /// assert_eq!(u, u.truncation(9));
    /// assert_eq!(u, u.truncation(7));
    /// let r = EAdic::new_repeating(5, vec![1, 3], vec![2, 1]);
    /// assert_eq!(u, r.truncation(7));
    ///
    /// let u = QAdic::new(UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]), -2);
    /// assert_eq!(u, u.truncation(7));
    /// assert_eq!(u, u.truncation(5));
    /// let r = QAdic::new(EAdic::new_repeating(5, vec![1, 3], vec![2, 1]), -2);
    /// assert_eq!(u, r.truncation(5));
    /// ```
    fn truncation(&self, n: Self::DigitIndex) -> Self::Truncation
    where Self: Sized {
        self.split(n).0
    }

    /// Truncate an adic number's expansion to n.
    /// This can be thought of as the remainder `a % p^n`.
    /// See also: [`truncation`](Self::truncation)
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, EAdic, QAdic, UAdic};
    /// let u = UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]);
    /// assert_eq!(u, u.clone().into_truncation(9));
    /// assert_eq!(u, u.clone().into_truncation(7));
    /// let r = EAdic::new_repeating(5, vec![1, 3], vec![2, 1]);
    /// assert_eq!(u, r.into_truncation(7));
    ///
    /// let u = QAdic::new(UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]), -2);
    /// assert_eq!(u, u.clone().into_truncation(7));
    /// assert_eq!(u, u.clone().into_truncation(5));
    /// let r = QAdic::new(EAdic::new_repeating(5, vec![1, 3], vec![2, 1]), -2);
    /// assert_eq!(u, r.into_truncation(5));
    /// ```
    fn into_truncation(self, n: Self::DigitIndex) -> Self::Truncation
    where Self: Sized {
        self.into_split(n).0
    }

    #[must_use]
    /// Divide an adic number by p^n.
    /// This can be thought of as the quotient a // p^n
    /// See also: [`into_quotient`](Self::into_quotient)
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, QAdic, UAdic};
    /// let u = UAdic::new(5, vec![1, 2, 3, 4]);
    /// let q = u.quotient(2);
    /// assert_eq!(UAdic::new(5, vec![3, 4]), q);
    ///
    /// let q = QAdic::new(UAdic::new(5, vec![1, 2, 3, 4]), -1);
    /// assert_eq!("432.1_5", q.to_string());
    /// let quot = q.quotient(2);
    /// assert_eq!(UAdic::new(5, vec![4]), quot);
    /// ```
    fn quotient(&self, n: Self::DigitIndex) -> Self::Quotient
    where Self: Sized {
        self.split(n).1
    }

    #[must_use]
    /// Divide an adic number by p^n.
    /// This can be thought of as the quotient a // p^n
    /// See also: [`quotient`](Self::quotient)
    ///
    /// ```
    /// # use adic::{traits::CanTruncate, QAdic, UAdic};
    /// let u = UAdic::new(5, vec![1, 2, 3, 4]);
    /// let q = u.into_quotient(2);
    /// assert_eq!(UAdic::new(5, vec![3, 4]), q);
    ///
    /// let q = QAdic::new(UAdic::new(5, vec![1, 2, 3, 4]), -1);
    /// assert_eq!("432.1_5", q.to_string());
    /// let quot = q.quotient(2);
    /// assert_eq!(UAdic::new(5, vec![4]), quot);
    /// ```
    fn into_quotient(self, n: Self::DigitIndex) -> Self::Quotient
    where Self: Sized {
        self.into_split(n).1
    }

}


/// Can be approximated into `Approximate`
pub trait CanApproximate: HasDigits {

    /// Output of the approximation
    type Approximation: HasApproximateDigits;

    /// Approximate an number expansion to digit index `n`.
    /// See also: [`into_approximation`](Self::into_approximation)
    ///
    /// ```
    /// # use adic::{traits::CanApproximate, EAdic, QAdic, UAdic, ZAdic};
    /// let u = UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]);
    /// let z = ZAdic::new_approx(5, 9, vec![1, 3, 2, 1, 2, 1, 2, 0, 0]);
    /// let zs = ZAdic::new_approx(5, 5, vec![1, 3, 2, 1, 2]);
    /// assert_eq!(z, u.approximation(9));
    /// assert_eq!(zs, u.approximation(5));
    /// assert_eq!(zs, z.approximation(5));
    /// assert_eq!(zs, zs.approximation(9));
    /// let r = EAdic::new_repeating(5, vec![1, 3], vec![2, 1]);
    /// assert_eq!(zs, r.approximation(5));
    ///
    /// let q = QAdic::new(UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]), -5);
    /// let z = QAdic::new(ZAdic::new_approx(5, 9, vec![1, 3, 2, 1, 2, 1, 2, 0, 0]), -5);
    /// let zs = QAdic::new(ZAdic::new_approx(5, 6, vec![1, 3, 2, 1, 2, 1]), -5);
    /// assert_eq!(z, q.approximation(4));
    /// assert_eq!(zs, q.approximation(1));
    /// let qr = QAdic::new(EAdic::new_repeating(5, vec![1, 3], vec![2, 1]), -5);
    /// assert_eq!(zs, qr.approximation(1));
    /// ```
    fn approximation(&self, n: Self::DigitIndex) -> Self::Approximation;

    /// Consume and get the approximation to digit index `n`.
    /// See also: [`approximation`](Self::approximation)
    ///
    /// ```
    /// # use adic::{traits::CanApproximate, EAdic, QAdic, UAdic, ZAdic};
    /// let u = UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]);
    /// let z = ZAdic::new_approx(5, 9, vec![1, 3, 2, 1, 2, 1, 2, 0, 0]);
    /// let zs = ZAdic::new_approx(5, 5, vec![1, 3, 2, 1, 2]);
    /// assert_eq!(z, u.clone().into_approximation(9));
    /// assert_eq!(zs, u.clone().into_approximation(5));
    /// assert_eq!(zs, z.clone().into_approximation(5));
    /// assert_eq!(zs, zs.clone().into_approximation(9));
    /// let r = EAdic::new_repeating(5, vec![1, 3], vec![2, 1]);
    /// assert_eq!(zs, r.into_approximation(5));
    ///
    /// let q = QAdic::new(UAdic::new(5, vec![1, 3, 2, 1, 2, 1, 2]), -5);
    /// let z = QAdic::new(ZAdic::new_approx(5, 9, vec![1, 3, 2, 1, 2, 1, 2, 0, 0]), -5);
    /// let zs = QAdic::new(ZAdic::new_approx(5, 6, vec![1, 3, 2, 1, 2, 1]), -5);
    /// assert_eq!(z, q.clone().into_approximation(4));
    /// assert_eq!(zs, q.clone().into_approximation(1));
    /// let qr = QAdic::new(EAdic::new_repeating(5, vec![1, 3], vec![2, 1]), -5);
    /// assert_eq!(zs, qr.into_approximation(1));
    /// ```
    fn into_approximation(self, n: Self::DigitIndex) -> Self::Approximation;

}
