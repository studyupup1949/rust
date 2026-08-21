# Arbitrary-Distribution Enumerative Sphere Shaping (AD-ESS)

An implementation of arbitrary-distribution enumerative sphere shaping (AD-ESS)[^1].
AD-ESS is am extension of the enumerative sphere shaping (ESS)[^2] algorithm that maps uniformly distributed bits to amplitudes with a distribution tailored to the additive white Gaussian noise (AWGN) channel.
In AD-ESS the output distribution of the amplitudes is adaptable, making the algorithm suitable for a wide range of channels.

A second algorithm named reverse trellis shaping (RTS) is also implemented.
Unlike AD-ESS it uses energy based ordering of the sequences and thus always has minimal rate loss.
Its complexity is the same as Laroias 1st algorithm[^3].

[^1]: https://arxiv.org/pdf/2512.16808.

[^2] F. M. J. Willems and J. J. Wuijts, "A pragmatic approach to shaped coded modulation," in Proc. IEEE Symp. on Commun. and Veh. Technol. in the Benelux, 1993.

[^3]: R. Laroia, N. Farvardin and S. A. Tretter, "On optimal shaping of multidimensional constellations," in IEEE Trans. Inf. Theory, vol. 40, no. 4, pp. 1044-1056, July 1994, doi: 10.1109/18.335969.

This folder contains the Rust code for AD-ESS.
Interesting files:

- `main.rs` provides a small example computing some metrics for AD-ESS
- `ad_ess.rs` provides a `struct AdEss` with methods for AD-ESS encoding, decoding and computing some useful metrics like average energy
- `rts.rs` provides a `struct RTS` similar to `AdEss` which uses a reversed trellis for shaping

## Installation

As AD-ESS relies on the `rug` crate which in turn uses GMP.
Compiling GMP on Windows might not work, failing the build of AD-ESS.

### From crates.io

Run `cargo add ad-ess` to add this crate to your rust project.

## From Source

Clone this git repo.
The Rust code can be compiled and run with `cargo run`.
An optimized build can be created using `cargo build --release`.

The documentation can be compiled with `cargo doc`.

## Testing

Some tests are located in `src/tests.rs`, these can be run with `cargo test`.
