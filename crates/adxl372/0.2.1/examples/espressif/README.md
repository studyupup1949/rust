# ADXL372 Espressif Examples

The purpose of these examples is to demonstrate how to use the ADXL372 driver in different scenarios 

If you need more information on how to work with Rust on Espressif chips, please refer to the [Official Espressif Rust documentation](https://docs.espressif.com/projects/rust/book/)

All the examples are based on the [Rust ESP Board](https://github.com/esp-rs/esp-rust-board)

## Hardware Setup

### Wiring Diagram

```text
+----------------+              +------------------+
| Rust ESP Board |              | ADXL372 Breakout |
|                |              |                  |
|    GPIO6 (SCK) |------------->| SCLK             |
|   GPIO2 (MISO) |<-------------| MISO             |
|   GPIO7 (MOSI) |------------->| MOSI             |
|    GPIO10 (CS) |------------->| CS               |
|                |              |                  |
|            3V3 |------------->| VS / VDDIO       |
|            GND |------------->| GND              |
+----------------+              +------------------+
```

## Available Examples

| Example | Description |
|---------|-------------|
| [`basic`](basic/) | Simple periodic read of XYZ acceleration data from the sensor using polling. | 

## Run Instructions

All example projects in this folder were generated with [esp-generate](https://github.com/esp-rs/esp-generate)

Each example defines its Cargo runner inside its own `.cargo/config.toml`. This means you can run an example from its directory with plain Cargo commands, and Cargo will use the correct runner configuration for flashing/monitoring

Typical workflow:

```sh
cd basic
cargo run
```

Repeat the same pattern for other examples by changing the directory (`basic`, `interrupt-read`, etc.)