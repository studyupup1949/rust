<a href="https://gitlab.com/jjwiseman/adsbx_json/-/pipelines">
<img alt="pipeline status"
     src="https://gitlab.com/jjwiseman/adsbx_json/badges/master/pipeline.svg">
</a>

# adsbx_json

This is a Rust library for parsing the JSON returned by the ADS-B
Exchange API. It currently supports [version 2 of the
API](https://www.adsbexchange.com/version-2-api-wip/).


## Goals

The goal of this library is to be a fast, type-safe interface to all
the data returned by both v1 and v2 of the ADS-B Exchange API.


## Status

Alpha, unstable. Handles all of v2 of the ADS-B Exchange API. Does not
handle v1 at all.

Currently parses about 250,000-300,000 aircraft objects per second.


## Development

Build:

```bash
cargo build
```

Build examples in release mode:

```bash
cargo build --release --examples fetch
```

Run unit and integration tests:

```bash
cargo test
```

Run example code:

```bash
ADSBX_API_KEY=xxx cargo run --example fetch https://adsbexchange.com/api/aircraft/v2/all
```

Output:

```
Got 3734 aircraft.
Aircraft {
    adsb_version: None,
    aircraft_type: Some(
        "A320",
    ),
    baro_rate: Some(
        -2170,
    ),
    barometric_altitude: Some(
        Altitude(
            12675,
        ),
    ),
    calc_track: None,
    call_sign: Some(
        "NKS907  ",
    ),
[...]
```

Run benchmarks:

```
cargo bench
```
