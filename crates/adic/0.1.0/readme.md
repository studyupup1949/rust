# adic

Adic number math

This crate handles adic numbers, arithmetic, and calculations, including:
- Adic numbers of various types,
 e.g. [UAdic] for natural numbers and [RAdic] for (most) rationals
- Rootfinding, through use of hensel lifting

### Example: calculate the two varieties for 7-adic sqrt(2) to 6 digits:
```rust
use adic::variety_to_digits;
let digits = variety_to_digits(7, 2, 2, 6).unwrap();
assert_eq!(vec![vec![3, 1, 2, 6, 1, 2], vec![4, 5, 4, 0, 5, 4]], digits);
```

### Example: 5-adic arithmetic
```rust
use adic::{UAdic, RAdic};
// 3 is a single digit (3) and no repeating digits
let three = RAdic::new(5, vec![3], vec![]);
// -1/6 consists only of repeating ...040404.
let neg_one_sixth = RAdic::new(5, vec![], vec![4, 0]);
// 3 - 1/6 = 17/6 is two digits 12. and then repeating 04
let seventeen_sixth = three + neg_one_sixth;
assert_eq!(RAdic::new(5, vec![2, 1], vec![4, 0]), seventeen_sixth);
assert_eq!(UAdic::new(5, vec![2, 1, 4, 0, 4, 0]), seventeen_sixth.truncate(6));
```

License: MIT OR Apache-2.0
