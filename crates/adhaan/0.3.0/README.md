# `adhaan`

<!-- cargo-rdme start -->

An Islamic prayer time calculator based on the [Adhan](https://github.com/batoulapps/Adhan)
library by Batoul Apps.

It is an attempt to make an ergonomic Rust port of the library from Batoul Apps, and attempts
to be as unopinionated and flexible as possible.

## Example

```rust
use adhaan::*;

let new_york_city = Coordinates { latitude: 40.7128, longitude: -74.0059 };
let date = jiff::civil::date(2019, 1, 25);
let params = Parameters::new(&prominent_methods::NorthAmerica);
let prayers = PrayerTimes::calculate(date, new_york_city, params).unwrap();
```

<!-- cargo-rdme end -->

## Acknowledgement

This library was started as a fork of the earlier port [`salah`](https://github.com/insha/salah) by Farhan Ahmed, which is still maintained but structured significantly differently.

All astronomical calculations are high precision equations directly from the book [Astronomical Algorithms](http://www.willbell.com/math/mc1.htm) by Jean Meeus.

## License

`adhaan` is licensed under the [MIT license](./LICENSE).
