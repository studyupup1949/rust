extern crate serde;
#[macro_use]
extern crate serde_big_array;

/// Global Constants
pub mod constants;
/// Traits for Digital Signatures
pub mod traits;


pub mod prelude;

pub mod encoding;

/// CSPRNG
pub mod os_random;

pub mod seed;

/// Falcon1024, a Lattice-Based crytography 
pub mod falcon;
/// BLS Signatures that can be aggregated
pub mod bls;
pub mod schnorr;
pub mod ed25519;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
