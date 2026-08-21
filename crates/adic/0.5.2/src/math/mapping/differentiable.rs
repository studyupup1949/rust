/// Trait for objects that are differentiable
pub trait Differentiable {

    /// Output Differentiable type
    type Output;

    /// The derivative of this object.
    /// See also: [`into_derivative`](Self::into_derivative)
    ///
    /// ```
    /// # use adic::{mapping::Differentiable, EAdic, Polynomial};
    /// let poly = Polynomial::<EAdic>::new_with_prime(5, vec![3, 7, 3]);
    /// assert_eq!("3._5x^2 + 12._5x^1 + 3._5x^0", poly.to_string());
    /// let deriv = poly.derivative();
    /// assert_eq!("11._5x^1 + 12._5x^0", deriv.to_string());
    /// ```
    fn derivative(&self) -> Self::Output
    where Self: Clone {
        self.clone().into_derivative()
    }

    /// Consume this object and return the derivative
    /// See also: [`derivative`](Self::derivative)
    ///
    /// ```
    /// # use adic::{mapping::Differentiable, EAdic, Polynomial};
    /// let poly = Polynomial::<EAdic>::new_with_prime(5, vec![3, 7, 3]);
    /// assert_eq!("3._5x^2 + 12._5x^1 + 3._5x^0", poly.to_string());
    /// let deriv = poly.into_derivative();
    /// assert_eq!("11._5x^1 + 12._5x^0", deriv.to_string());
    /// ```
    fn into_derivative(self) -> Self::Output;
}
