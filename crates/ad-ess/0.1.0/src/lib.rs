//! This crate contains the arbitrary-distribution enumerative sphere shaping (AD-ESS) algorithm[^1].
//! The algorithm is accessible via the `struct` [ad_ess::AdEss].
//!
//! A second algorithm named reverse trellis shaping [rts::RTS] is also located in this repository.
//! Unlike AD-ESS it uses energy based ordering of the sequences and thus always has minimal rate loss.
//! Its complexity is the same as Laroias 1st algorithm[^2].
//!
//! [^1]: https://arxiv.org/pdf/2512.16808.
//!
//! [^2]: R. Laroia, N. Farvardin and S. A. Tretter, "On optimal shaping of multidimensional constellations," in IEEE Transactions on Information Theory, vol. 40, no. 4, pp. 1044-1056, July 1994, doi: 10.1109/18.335969.

/// Arbitrary-Distribution ESS
pub mod ad_ess;
/// Implementation of a trellis used in [ad_ess::AdEss] and [rts::RTS]
pub mod trellis;
pub mod trellis_utils;
pub mod utils;

/// Reverse Trellis Shaping
pub mod rts;

#[cfg(test)]
mod tests;
