
//! accounting is a library for money and currency formatting. 
//! 
//! # Examples
//! 
//! ```
//! # use accounting::Accounting;
//! let mut ac = Accounting::new_from("$", 2);
//!	ac.set_format("{s} {v}");
//!	assert_eq!(ac.format_money(1000000), "$ 1,000,000.00");
//!	assert_eq!(ac.format_money(-5000), "-$ 5,000.00");
//! ```
//! 
//! Set the format string of [Accounting] variable，then format numbers as money values.  In the format string:  
//! - {v} is placehoder of value, will be replaced by number.  
//! - {s} is placehoder of symbol, will be replaced by currency symbol like $、￥ and so on.
//! 

#[cfg(test)]
mod tests;

pub mod unformat_number;
pub mod format_number;
pub use format_number::FormatNumber;
pub use unformat_number::{unformat, UnformatError};

/// Format numbers as money values according to settings.   
/// 
/// Accounting struct fields and default value. 
///  
/// | Field | Type | Description | Default | Example |
/// | --------------- | ------------- | ------------- | ------------- | ------------- |
/// | symbol          | String | currency symbol |  $ | $ |
/// | precision       | usize  | currency precision (decimal places) | 0 | 2 |
/// | thousand        | String | thousand separator | , | . |
/// | decimal         | String | decimal separator | . | , |
/// | format_positive | String | format string for positive values ({v} = value, {s} = symbol) | {s}{v} | {s} {v} |
/// | format_negative | String | format string for negative values | -{s}{v} | {s} ({v}) |
/// | format_zero     | String | format string for zero values | {s}{v} | {s} -- |
///
pub struct Accounting {
	symbol: String,
	precision: usize,  
	thousand: String,
	decimal: String, 
	format_positive: String,
	format_negative: String,
	format_zero: String
}

impl Default for Accounting {
    /// Returns a “default" Accounting.  
    fn default() -> Self {
        let format = "{s}{v}";
        Accounting {
            symbol: "$".to_string(), 
            precision: 0, 
            thousand: ",".to_string(),
            decimal: ".".to_string(), 
            format_positive: format.to_string(), 
            format_negative: "-".to_string() + format, 
            format_zero: format.to_string()
        }
    }
}

impl Accounting {

    /// Create Accounting from symbol、 precision and default settings.
    pub fn new_from(symbol: &str, precision: usize) -> Self {
        let mut ac = Self::default();
        ac.symbol = symbol.to_string();
        ac.precision = precision;
        return ac;
    }
    
    /// Create Accounting from symbol、 precision、thousand separator、 decimal separator and default settings.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use accounting::Accounting;
    /// let ac = Accounting::new_from_seperator("€", 2, ".", ",");
    /// assert_eq!(ac.format_money(4999.99), "€4.999,99");
    /// ```
    pub fn new_from_seperator(symbol: &str, precision: usize, thousand: &str, decimal: &str) -> Self {
        let mut ac = Self::default();
        ac.symbol = symbol.to_string();
        ac.precision = precision;
        ac.thousand = thousand.to_string();
        ac.decimal = decimal.to_string();
        return ac;
    }

    /// Create Accounting 
    pub fn new(
        symbol: &str, 
        precision: usize, 
        thousand: &str, 
        decimal: &str, 
        format: &str, 
        format_negative: &str, 
        format_zero: &str
    ) -> Self {
        Accounting {
            symbol: symbol.to_string(), 
            precision, 
            thousand: thousand.to_string(), 
            decimal: decimal.to_string(), 
            format_positive: format.to_string(), 
            format_negative: format_negative.to_string(), 
            format_zero: format_zero.to_string()
        }
    }

    /// Sets the separator for the thousands separation.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use accounting::Accounting;
    /// let mut ac = Accounting::new_from("$", 2);
    /// ac.set_thousand_separator("'");
    /// assert_eq!(ac.format_money(123456789.213123), "$123'456'789.21")
    /// ```
    pub fn set_thousand_separator(&mut self, str: &str) {
        self.thousand = str.to_string();
    }

    /// Sets the separator for the decimal separation.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use accounting::Accounting;
    /// let mut ac = Accounting::new_from("$", 2);
	/// ac.set_decimal_separator("'");
	/// assert_eq!(ac.format_money(123456789.213123), "$123,456,789'21")
    /// ```
    pub fn set_decimal_separator(&mut self, str: &str) {
        self.decimal = str.to_string();
    }

    /// Sets the format string for positive and zero value. 
    /// Also Sets format string by adding `-` at begining for negative value.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use accounting::Accounting;
    /// let mut ac = Accounting::new_from("$", 2);
    /// ac.set_format("{v} {s}");
    ///	assert_eq!(ac.format_money(123456789.213123), "123,456,789.21 $");
    ///	assert_eq!(ac.format_money(-123456789.213123), "-123,456,789.21 $");
    ///	assert_eq!(ac.format_money(0), "0.00 $");
    /// ```
    pub fn set_format(&mut self, str: &str) {
        self.set_format_positive(str);
        self.set_format_negative(&format!("-{}", str));
        self.set_format_zero(str);
    }

    /// Sets the format string for positive values.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use accounting::Accounting;
    /// let mut ac = Accounting::new_from("$", 2);
    /// ac.set_format_positive("{s} {v}");
    /// ac.set_format_negative("{s} ({v})");
    ///	ac.set_format_zero( "{s} --");
    /// assert_eq!(ac.format_money(1000000), "$ 1,000,000.00");
    ///	assert_eq!(ac.format_money(-5000), "$ (5,000.00)");
    ///	assert_eq!(ac.format_money(0), "$ --");
    /// ```
    pub fn set_format_positive(&mut self, str: &str) {
        self.format_positive = str.to_string();
    }

    /// Sets the format string for negative values.
    pub fn set_format_negative(&mut self, str: &str) {
        self.format_negative = str.to_string();
    }

    /// Sets the format string for zero values.
    pub fn set_format_zero(&mut self, str: &str) {
        self.format_zero = str.to_string();
    }
 
    /// `format_money` function format numbers as money values, 
    /// using customisable settings of currency symbol, precision, and thousand/decimal separators. 
    /// The value type need to implement [FormatNumber] trait. 
    pub fn format_money<T:FormatNumber>(&self, value: T) -> String {
        let mut number_string = value.format_number(self.precision, &self.thousand, &self.decimal);
        let zero_string = 0.format_number(self.precision, &self.thousand, &self.decimal);

        let format_string;
        if &number_string[0..1] == "-" {
            number_string = number_string[1..number_string.len()].to_string();
            format_string = &self.format_negative;
        } else if number_string == zero_string {
            format_string = &self.format_zero;
        } else {
            format_string = &self.format_positive;
        }

        let mut result = format_string.replace("{s}", &self.symbol);
        result = result.replace("{v}", &number_string);
        return result;
    }
}