

use super::Accounting;
#[test]
fn accounting_test() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_format("{v} {s}");
	assert_eq!(ac.format_money(123456789.213123), "123,456,789.21 $");
	assert_eq!(ac.format_money(-123456789.213123), "-123,456,789.21 $");
	assert_eq!(ac.format_money(0), "0.00 $");
}

#[test]
fn accounting_zero_test() {
	let mut ac = Accounting::default();
	ac.set_format_zero("{s} --");
	assert_eq!(ac.format_money(0), "$ --");
}

#[test]
fn accounting_neg_test() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_format_negative("{s}({v})");
	assert_eq!(ac.format_money(-123456789.213123), "$(123,456,789.21)");
}

#[test]
fn accounting_set_thousand_separator_test() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_thousand_separator("'");
	assert_eq!(ac.format_money(123456789.213123), "$123'456'789.21")
}

#[test]
fn accounting_set_decimal_separator_test() {
	let mut ac = Accounting::new_from("$", 2);
	ac.set_decimal_separator("'");
	assert_eq!(ac.format_money(123456789.213123), "$123,456,789'21")
}

#[test]
fn account_format_money_type_test() {
	let ac = Accounting::new_from("$", 2);
	assert_eq!(ac.format_money(-1i8), "-$1.00");
	assert_eq!(ac.format_money(1u8), "$1.00");

	assert_eq!(ac.format_money(-1i16), "-$1.00");
	assert_eq!(ac.format_money(1u16), "$1.00");

	assert_eq!(ac.format_money(-1i32), "-$1.00");
	assert_eq!(ac.format_money(1u32), "$1.00");

	assert_eq!(ac.format_money(-1i64), "-$1.00");
	assert_eq!(ac.format_money(1u64), "$1.00");

	assert_eq!(ac.format_money(-1i128), "-$1.00");
	assert_eq!(ac.format_money(1u128), "$1.00");

	assert_eq!(ac.format_money(-1isize), "-$1.00");
	assert_eq!(ac.format_money(1usize), "$1.00");

	assert_eq!(ac.format_money(-1f32), "-$1.00");
	assert_eq!(ac.format_money(1f64), "$1.00");
}

#[test]
fn account_format_money_test() {
	let ac = Accounting::new_from("$", 2);
	assert_eq!(ac.format_money(123456789.213123), "$123,456,789.21");
	assert_eq!(ac.format_money(12345678), "$12,345,678.00");
	assert_eq!(ac.format_money(-12345678), "-$12,345,678.00");
	assert_eq!(ac.format_money(0), "$0.00");

	let ac = Accounting::new("$", 0, ",", ".",
	 "{s} {v}", "-{s} {v}", "{s} {v}");
	assert_eq!(ac.format_money(123456789.213123), "$ 123,456,789");
	assert_eq!(ac.format_money(12345678), "$ 12,345,678");
	assert_eq!(ac.format_money(-12345678), "-$ 12,345,678");
	assert_eq!(ac.format_money(0), "$ 0");

	let ac = Accounting::new_from_seperator("€", 2, ".", ",");
	assert_eq!(ac.format_money(4999.99), "€4.999,99");

	let ac = Accounting::new_from("£ ", 0);
	assert_eq!(ac.format_money(500000), "£ 500,000");

	let mut ac = Accounting::new_from("GBP", 0);
	ac.set_format_positive("{s} {v}");
	ac.set_format_negative("{s} ({v})");
	ac.set_format_zero("{s} --");
	assert_eq!(ac.format_money(1000000), "GBP 1,000,000");
	assert_eq!(ac.format_money(-5000), "GBP (5,000)");
	assert_eq!(ac.format_money(0), "GBP --");

	let mut ac = Accounting::new_from("GBP", 2);
	ac.set_format("{s} {v}");
	ac.set_format_negative("{s} ({v})");
	ac.set_format_zero( "{s} --");
	assert_eq!(ac.format_money(1000000), "GBP 1,000,000.00");
	assert_eq!(ac.format_money(-5000), "GBP (5,000.00)");
	assert_eq!(ac.format_money(0), "GBP --");

	let mut ac = Accounting::new_from("€", 2);
	ac.set_format_zero("0.-");
	assert_eq!(ac.format_money(0), "0.-");
}


