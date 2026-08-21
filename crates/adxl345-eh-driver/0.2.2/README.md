# ADXL345 Accelerometer
A Rust device driver for the [ADXL345 Analog Devices Digital Accelerometer](https://www.analog.com/media/en/technical-documentation/data-sheets/adxl345.pdf).

## Usage
```rust
    let mut accel = adxl345_eh_driver::Driver::new(i2c, None).unwrap();
    let (x, y, z) = accel.get_accel_raw().unwrap();
```

## License
[![license](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://gitlab.com/scrobotics/embedded-rs/adxl234-rs/-/blob/master/LICENSE)

This tool is released under the MIT license, hence allowing commercial use of the library. Please refer to the [LICENSE](https://gitlab.com/scrobotics/embedded-rs/adxl234-rs/-/blob/master/LICENSE) file.
