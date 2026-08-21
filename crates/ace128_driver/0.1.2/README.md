# Bourns ACE128 Absolute Contacting Encoder Driver

GPIO based driver for the ACE128 Absolute Contacting Encoder.

The driver converts the 128 bit output into an absolute angle in radians as a 64 bit floating point value, with a discrete interval of 2*pi/128.
An example based on the Raspberry Pi is shown below.

## Example

```rust

use rppal::gpio::{Gpio, OutputPin, InputPin};
use ace128_driver::Ace128;

fn main() {

    let encoder = Ace128::new(
        Gpio::new().unwrap().get(1).unwrap().into_input(),
        Gpio::new().unwrap().get(2).unwrap().into_input(),
        Gpio::new().unwrap().get(3).unwrap().into_input(),
        Gpio::new().unwrap().get(4).unwrap().into_input(),
        Gpio::new().unwrap().get(5).unwrap().into_input(),
        Gpio::new().unwrap().get(6).unwrap().into_input(),
        Gpio::new().unwrap().get(7).unwrap().into_input(),
        Gpio::new().unwrap().get(8).unwrap().into_input()
    );

    loop {
        let angle = encoder.read_angle().unwrap();
    }
}
```
