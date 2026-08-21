use accounting::Accounting;
use accounting::{unformat, UnformatError};

#[test]
fn test_set_format() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_format("{v} {s}");
	assert_eq!(ac.format_money(123456789.213123), "123,456,789.21 $");
	assert_eq!(ac.format_money(-123456789.213123), "-123,456,789.21 $");
	assert_eq!(ac.format_money(0), "0.00 $");
}

#[test]
fn test_set_format_zero() {
	let mut ac = Accounting::default();
	ac.set_format_zero("{s} --");
	assert_eq!(ac.format_money(0), "$ --");
}

#[test]
fn test_set_format_neg() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_format_negative("{s}({v})");
	assert_eq!(ac.format_money(-123456789.213123), "$(123,456,789.21)");
}

#[test]
fn test_set_thousand_separator() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_thousand_separator("'");
	assert_eq!(ac.format_money(123456789.213123), "$123'456'789.21")
}

#[test]
fn test_set_decimal_separator() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_decimal_separator("'");
	assert_eq!(ac.format_money(123456789.213123), "$123,456,789'21")
}

#[cfg(feature="decimal")]
#[test]
fn test_format_decimal_type() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_format("{s} {v}");
	let x = rust_decimal::Decimal::new(0, 1);
    assert_eq!(ac.format_money(x), "$ 0.00"); 
	let x = rust_decimal::Decimal::new(-123456789213, 3);
    assert_eq!(ac.format_money(x), "-$ 123,456,789.21"); 
}

#[test]
fn test_unformat() {
	assert_eq!(unformat("-$4,500.23", 2, "USD"), Ok("-4500.23".to_string()));
	assert_eq!(unformat("EUR 111.145.000,33", 2, "eur"), Ok("111145000.33".to_string()));
	assert_eq!(unformat("$45,567.10", 2, "zzz"), Err(UnformatError::NoLocaleFound));
}
