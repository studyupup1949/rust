//! Strip out all currency formatting and return the numberic string.  
//! 
//! The [unformat] function pulls the currency descripter from the locale_info_map and 
//! uses it to return an unformatted value based on thousands seperator and decimal seperator.
//! 
//! # Examples
//! 
//! ```
//! # use accounting::{unformat, UnformatError};
//! assert_eq!(unformat("$4,500.23", 2, "USD"), Ok("4500.23".to_string()));
//! assert_eq!(unformat("$45,567.10", 2, "zzz"), Err(UnformatError::NoLocaleFound));
//! ```

mod locale;

use std::fmt;
use std::error;
use std::num::ParseFloatError;
use regex::Regex;
use locale::{Locale, locale_info_map};

type Result<T> = std::result::Result<T, UnformatError>;

#[derive(Debug, PartialEq)]
pub enum UnformatError {
    NoLocaleFound,
    Parse(ParseFloatError),
}

impl fmt::Display for UnformatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UnformatError::NoLocaleFound => write!(f, "no locale info found"),
            UnformatError::Parse(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for UnformatError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            UnformatError::NoLocaleFound => None,
            UnformatError::Parse(ref e) => Some(e),
        }
    }
}

impl From<ParseFloatError> for UnformatError {
    fn from(err: ParseFloatError) -> UnformatError {
        UnformatError::Parse(err)
    }
}

/// Takes a string of the number to strip currency info on
/// and precision for decimals.
pub fn unformat(n: &str, precision: usize, currency: &str) -> Result<String> {
	let lc: Locale;
	let currency = currency.to_uppercase();
	
	if let Some(val)=  locale_info_map(&currency) {
		lc = val;
	} else {
		return Err(UnformatError::NoLocaleFound);
	}

	let r = Regex::new(r"[^0-9-., ]").unwrap();
	let mut num = r.replace_all(n, "").into_owned();
	num = num.replace(lc.thousands_seperator, "");

	// Replace decimal seperator with a decimal
	if lc.decimal_seperator != "." {
		num = num.replace(lc.decimal_seperator, ".");
	}

	let v: f64 = num.trim().parse()?;
	num = format!("{0:.1$}", v, precision);
	Ok(num)
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn unformat_test() {
		assert_eq!(unformat("$4,500.23", 2, "USD"), Ok("4500.23".to_string()));
		assert_eq!(unformat("EUR 12.500,3474", 3, "EUR"), Ok("12500.347".to_string()));
		assert_eq!(unformat("EUR 111.145.000,33", 2, "eur"), Ok("111145000.33".to_string()));
	}

	#[test]
	fn unformat_error_test() {
		assert_eq!(unformat("$45,567.10", 2, "zzz"), Err(UnformatError::NoLocaleFound));
	}
}