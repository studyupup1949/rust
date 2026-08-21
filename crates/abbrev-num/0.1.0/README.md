# abbrev-num

Abbreviate numbers into a human-friendly format

# Examples

## Basic

```rust
use abbrev_num::abbrev_num;

assert_eq!(abbrev_num(1_400, None), Some("1.4k".to_string()));
```

## Precision

```rust
use abbrev_num::{abbrev_num, Options};

let options = Options {
    precision: Some(2),
    ..Default::default()
};

assert_eq!(abbrev_num(1_420, Some(options)), Some("1.42k".to_string()));
```

## Custom units

```rust
use abbrev_num::{abbrev_num, Options};

let units: [&str; 7] = ["mm", "cm", "m", "km", "", "", ""];
let options = Options {
    abbreviations: Some(units),
    ..Default::default()
};

assert_eq!(abbrev_num(1_400, Some(options)), Some("1.4cm".to_string()));
```

## Custom rounding strategy

```rust
use abbrev_num::{abbrev_num, Options, RoundingStrategy};

let options = Options {
    rounding_strategy: Some(RoundingStrategy::ToZero),
    precision: Some(0),
    ..Default::default()
};

assert_eq!(abbrev_num(1_566_450, Some(options)), Some("1M".to_string()));
```
