# ADLine  
```
use ta_common::traits::Indicator;
use ad_rs::ADLine;

let mut ad_line = ADLine::new();
let ad = ad_line.next([200f64, 100f64, 170f64, 10f64]);
assert_eq!(4_f64, ad);
let ad = ad_line.next([200f64, 100f64, 170f64, 10f64]);
assert_eq!(8_f64, ad);
ad_line.reset();
let ad = ad_line.next([200f64, 100f64, 170f64, 10f64]);
assert_eq!(4_f64, ad);
```