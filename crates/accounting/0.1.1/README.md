# accounting

[![Crate](https://img.shields.io/crates/v/accounting.svg)](https://crates.io/crates/accounting)
[![API](https://docs.rs/rand/badge.svg)](https://docs.rs/accounting)


accounting is a library for money and currency formatting. (inspired by [accounting for golang](https://github.com/leekchan/accounting))


## Examples: 
```Rust
use accounting::Accounting;
fn main() {
    let mut ac = Accounting::new_from("$", 2);
    ac.set_format("{s} {v}");
    assert_eq!(ac.format_money(1000000), "$ 1,000,000.00");
    assert_eq!(ac.format_money(-5000), "-$ 5,000.00");
}
```
Set the format string of `Accounting` variable，then format numbers as money values. In the format string:
- {v} is placehoder of value, will be replaced by number.  
- {s} is placehoder of symbol, will be replaced by currency symbol like $、￥ and so on.


## Accounting struct

```rust
pub struct Accounting {
	symbol: String,
	precision: usize,  
	thousand: String,
	decimal: String, 
	format_positive: String,
	format_negative: String,
	format_zero: String
}
```

 
| Field | Type | Description | Default | Example |
| ------------- | ------------- | ------------- | ------------- | ------------- |
| symbol          | String | currency symbol |  $ | $ |
| precision       | usize  | currency precision (decimal places) | 0 | 2 |
| thousand        | String | thousand separator | , | . |
| decimal         | String | decimal separator | . | , |
| format_positive | String | format string for positive values ({v} = value, {s} = symbol) | {s}{v} | {s} {v} |
| format_negative | String | format string for negative values | -{s}{v} | {s} ({v}) |
| format_zero     | String | format string for zero values | {s}{v} | {s} -- |


## Examples: 
- Set format string.
```rust
let mut ac = Accounting::new_from("$", 2);
ac.set_format_positive("{s} {v}");
ac.set_format_negative("{s} ({v})");
ac.set_format_zero( "{s} --");
assert_eq!(ac.format_money(1000000), "$ 1,000,000.00");
assert_eq!(ac.format_money(-5000), "$ (5,000.00)");
assert_eq!(ac.format_money(0), "$ --");
```
- Set thousand separator.
```rust
let mut ac = Accounting::new_from("$", 2);
ac.set_thousand_separator("'");
assert_eq!(ac.format_money(123456789.213123), "$123'456'789.21")
```

- Set decimal separator.
```rust
let mut ac = Accounting::new_from("$", 2);
ac.set_decimal_separator("'");
assert_eq!(ac.format_money(123456789.213123), "$123,456,789'21")
```
 
`format_money` function parameter need to implement `FormatNumber` trait.


## FormatNumber trait
`FormatNumber` is a trait of the library.
The type which implement this trait can be format to string with custom precision and separators. Implemented types include:   
i8, u8, i16, u16 i32, u32 i64, u64, i128, u128, isize, usize, f32, f64

Trait define:
```rust
pub trait FormatNumber {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String;
}
```

Examples:

```rust
use accounting::FormatNumber;
let x = 123456789.213123f64;
assert_eq!(x.format_number(2, ",", "."), "123,456,789.21");
```



## unformat function
`unformat` function strips out all currency formatting and returns the numberic string.

Examples:

```rust
use accounting::unformat;
assert_eq!(unformat("$4,500.23", 2, "USD"), Ok("4500.23".to_string()));
assert_eq!(unformat("EUR 12.500,3474", 3, "EUR"), Ok("12500.347".to_string()));
```

