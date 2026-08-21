//! Define and implement FormatNumber trait. 
//! 
//! This trait be used for formatting numberic types to string with custom precision and separators.
//! Implemented types include： i8, u8, i16, u16 i32, u32 i64, u64, i128, u128, isize, usize, f32, f64。
//!
//! # Examples
//! 
//! ```
//! # use accounting::FormatNumber;
//! let x = 123456789.213123f64;
//! assert_eq!(x.format_number(2, ",", "."), "123,456,789.21");
//! ```

mod format_macro;
use format_macro::*;

/// Trait for formatting numbers with custom precision and separators. 
pub trait FormatNumber {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String;
}

impl FormatNumber for i8 {
    fn format_number(&self, precision: usize, _: &str, decimal: &str) -> String {
        if precision > 0 {
            format!("{}{}{}", *self, decimal, "0".repeat(precision))
        } else {
            self.to_string()
        }
    }
}

impl FormatNumber for u8 {
    fn format_number(&self, precision: usize, _: &str, decimal: &str) -> String {
        if precision > 0 {
            format!("{}{}{}", *self, decimal, "0".repeat(precision))
        } else {
            self.to_string()
        }
    }
}

impl FormatNumber for i16 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for i32 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for i64 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for i128 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for isize {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u16 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_uint!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u32 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_uint!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u64 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_uint!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u128 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_uint!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for usize {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_uint!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for f32 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_float!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for f64 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_float!(*self, precision, thousand, decimal)
    }
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn format_number_int_test() {
        let x = -220300isize;
        assert_eq!(x.format_number(2, ",", "."), "-220,300.00");
	}

    #[test]
    fn format_number_uint_test() {
        let x = 320300usize;
        assert_eq!( x.format_number(2, ",", "."), "320,300.00");
	}

    #[test]
    fn format_number_float_test() {
        let x = 123456789.213123f64;
        assert_eq!(x.format_number(2, ",", "."), "123,456,789.21");
	}
}