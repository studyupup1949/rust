# adbc-driver-flightsql

`adbc-driver-flightsql` ships the native ADBC FlightSQL driver from the conda-forge
distribution and exposes a stable filesystem path for Rust applications. The crate
handles platform detection, download, extraction, and version reporting during
`cargo build`, so consuming projects only need a single constant.

## Highlights
- Automatic retrieval of the `libadbc-driver-flightsql` conda artifact for the active target
- No runtime dependencies: the crate only exports `DRIVER_PATH` and `DRIVER_VERSION`
- Configurable driver version, build string, and channel through environment variables

## Installation
```toml
[dependencies]
adbc-driver-flightsql = "0.1"
```

## Usage
```rust
use adbc_core::options::AdbcVersion;
use adbc_driver_flightsql::{DRIVER_PATH, DRIVER_VERSION};
use adbc_driver_manager::ManagedDriver;

let driver = ManagedDriver::load_dynamic_from_filename(
    DRIVER_PATH,
    None,
    AdbcVersion::default(),
).expect("load FlightSQL driver");

println!("FlightSQL driver version: {DRIVER_VERSION}");
```

## Version control
The build script bundles version `1.8.0` by default. Override the conda artifact at build
time with:
- `ADBC_FLIGHTSQL_VERSION` – desired package version (e.g. `1.8.0`)
- `ADBC_FLIGHTSQL_BUILD` – build string matching the requested platform (e.g. `hbbbe3c2_1`)
- `ADBC_FLIGHTSQL_CHANNEL` – alternate conda channel base URL

The selected version is available to consumers through the `DRIVER_VERSION` constant.

## Supported targets
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## License
Licensed under Apache-2.0
