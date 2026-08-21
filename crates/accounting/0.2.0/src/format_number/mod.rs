//! Define and implement FormatNumber trait. 
//! 
//! This trait be used for formatting numberic types to string with custom precision and separators.
//! Implemented types include:
//! * primitive type: i8, u8, i16, u16 i32, u32 i64, u64, i128, u128, isize, usize, f32, f64.
//! * decimal type: `rust_decimal::Decimal`
//!
//! # Examples
//! 
//! ```
//! # use accounting::FormatNumber;
//! let x = 123456789.213123f64;
//! assert_eq!(x.format_number(2, ",", "."), "123,456,789.21");
//! 
//! #[cfg(feature="decimal")]
//! {
//!     use rust_decimal::Decimal;
//!     let x = Decimal::new(12345678921, 2);
//!     assert_eq!( x.format_number(2, ",", "."), "123,456,789.21"); 
//! }
//! ```

mod primitive;
#[cfg(feature = "decimal")]
mod decimal;

/// Trait for formatting numbers with custom precision and separators. 
pub trait FormatNumber {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String;
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn format_number_int_test() {
        let x = -220300isize;
        assert_eq!(x.format_number(2, ",", "."), "-220,300.00");

        let x = 320300usize;
        assert_eq!( x.format_number(2, ",", "."), "320,300.00");
	}

    #[test]
    fn format_number_float_test() {
        let x = 123456789.213123f64;
        assert_eq!(x.format_number(2, ",", "."), "123,456,789.21");
	}
    #[cfg(feature = "decimal")]
    #[test]
    fn format_number_decimal_test() {
        let x = rust_decimal::Decimal::new(12345678921, 2);
        assert_eq!( x.format_number(2, ",", "."), "123,456,789.21");     
        
        let x = rust_decimal::Decimal::new(-123456789213, 3);
        assert_eq!( x.format_number(2, ",", "."), "-123,456,789.21"); 
	}
}