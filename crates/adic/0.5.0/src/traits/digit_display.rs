/// Helper trait for display
///
/// Has digits that be represented by a string, without dressings like `.` or `_p`.
pub (crate) trait HasDigitDisplay {

    type DigitDisplay;

    /// A string representing just the digits of this adic number; used to Display the integer.
    ///
    /// ```
    /// # use adic::{EAdic, ZAdic};
    /// assert_eq!("(4)31._5", EAdic::new_neg(5, vec![1, 3]).to_string());
    /// assert_eq!("(24)31._5", EAdic::new_repeating(5, vec![1, 3], vec![4, 2]).to_string());
    /// assert_eq!("...3412._5", ZAdic::new_approx(5, 4, vec![2, 1, 4, 3]).to_string());
    /// ```
    fn digit_display(&self) -> Self::DigitDisplay;

}
