<a id="readme-top"></a>

<div align="center">
  <h1 align="center">acn-protocol</h3>
  <h3 align="center">
    Architecture for Control Networks (ACN) protocol written in Rust
  </h3>
  <div align="center">

[![Crates.io](https://img.shields.io/crates/v/acn-protocol.svg)](https://crates.io/crates/acn-protocol)
[![Docs](https://img.shields.io/badge/docs-latest-blue)](https://docs.rs/acn-protocol/latest/acn_protocol/)
[![Docs](https://img.shields.io/badge/msrv-1.86.0-red)](https://docs.rs/acn-protocol/latest/acn_protocol/)

  </div>
</div>

## About the project

Architecture for Control Networks (ACN) consists of a suite of protocols and languages which may be
configured and combined with other standard protocols in a number of ways to form flexible networked control
systems.

### Included

- Data-types and traits for encoding and decoding ACN protocols.

### Not included

- Specific ACN protocol implementations

### Implemented specifications / supported parameters

- ANSI E1.17 (2015): Architecture for Control Networks – ACN Architecture

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Installation

```sh
cargo add acn-protocol
```

or add to Cargo.toml dependencies, [crates.io](https://crates.io/crates/acn-protocol) for latest version.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Usage

### Implement PduCodec for the protocols specific PDUs

See RootLayerCodec and PduCodec tests for examples.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Contributing

This project is open to contributions, create a new issue and let's discuss.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Acknowledgments

- The ANSI E1.17 (2015) specification used to create this library is copyright and published by [ESTA](https://www.esta.org/)

<p align="right">(<a href="#readme-top">back to top</a>)</p>
