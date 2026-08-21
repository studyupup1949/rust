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
adbc-driver-flightsql = "0.1.1"
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
The build script downloads the official PyPI wheel (`adbc-driver-flightsql`) for the active
target. Version `1.9.0` is used by default across all platforms, and developers can
override the behavior with:
- `ADBC_FLIGHTSQL_VERSION` – desired PyPI version (e.g. `1.9.0`)
- `ADBC_FLIGHTSQL_LIB_PATH` – custom filesystem path (directory or full file path) to copy the library to (e.g. `/usr/local/lib/` or `/usr/local/lib/libadbc_driver_flightsql.so`)

The resolved version is exported to consuming crates through the `DRIVER_VERSION` constant.

## Supported targets
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
## License
Licensed under Apache-2.0
