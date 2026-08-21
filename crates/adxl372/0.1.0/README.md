# ADXL372

`no_std` driver for the Analog Devices ADXL372 high-g 3-axis MEMS accelerometer.

The driver targets microcontrollers and exposes a safe, typed API built on
[`embedded-hal`](https://docs.rs/embedded-hal/) traits. It follows the datasheet's
register settings and timing requirements and keeps core memory usage explicit by
avoiding heap allocation.

## Features

Optional Cargo features:

- `defmt`: enable `defmt` logging for internal debug traces

## Usage

Import the relevant HAL crate for your platform. For this example I'm using esp-hal on ESP32C3.

```rust
use adxl372::device::Adxl372;
use adxl372::config::Config;
use adxl372::interface::spi::SpiInterface;
use adxl372::params::{Bandwidth, OutputDataRate, PowerMode};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::delay::Delay;
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::gpio::{Level, Output, OutputConfig};

use panic_rtt_target as _;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
	let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
	let peripherals = esp_hal::init(config);

	let sclk = peripherals.GPIO6;
	let miso = peripherals.GPIO2;
	let mosi = peripherals.GPIO7;
	let cs = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
	let spi_delay = Delay::new();

	let spi = Spi::new(
		peripherals.SPI2,
		SpiConfig::default()
			.with_frequency(Rate::from_khz(400))
			.with_mode(Mode::_0),
	)
	.expect("SPI init")
	.with_sck(sclk)
	.with_miso(miso)
	.with_mosi(mosi);

	let spi_device = ExclusiveDevice::new(spi, cs, spi_delay).unwrap();
	let iface = SpiInterface::new(spi_device);
	let config = Config::new()
		.odr(OutputDataRate::Od6400Hz)
		.bandwidth(Bandwidth::Bw1600Hz)
		.power_mode(PowerMode::Measure)
		.build();

	let mut accel = Adxl372::new(iface, config);
	let mut delay = Delay::new();
	accel.init(&mut delay).unwrap();

	loop {
		let [x, y, z] = accel.read_xyz_raw().unwrap();
		let _ = (x, y, z);
		delay.delay_ms(500);
	}
}
```

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.