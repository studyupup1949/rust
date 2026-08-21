/// Has a [norm](https://en.wikipedia.org/wiki/Norm_(mathematics)) and
///  [unit](https://en.wikipedia.org/wiki/Unit_(ring_theory))
///
/// For our purposes, the units are expected to have norm `1`.
/// However, we do **not** expect `x = |x| unit(x)`.
/// For example, for a normal integer you would have `x = |x| unit(x)` but
///  for a p-adic integer you would have `x = p^(val(x)) unit(x) = (|x|_p)^(-1) unit(x)`.
pub trait Normed: Sized {

    /// Type for the number's norm
    type Norm;

    /// Type for the number's unit
    type Unit;

    /// Norm of the number, the "size"
    fn norm(&self) -> Self::Norm;

    /// Unit component of the number, or None if `Zero`
    fn unit(&self) -> Option<Self::Unit>;

    /// Unit component of the number, or None if `Zero`
    fn into_unit(self) -> Option<Self::Unit>;

    /// Create with given norm and unit
    fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self;

    /// Create from a unit
    fn from_unit(u: Self::Unit) -> Self;

    /// Test if the number is a unit
    ///
    /// Equivalently, if the number is valuation zero or if there are no fractional digits, but the zero-index digit is nonzero.
    fn is_unit(&self) -> bool;

}
