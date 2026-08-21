extern crate serde;

/// Global Constants
pub mod constants;
/// Traits for Digital Signatures
pub mod traits;


pub mod prelude;

pub mod encoding;

/// CSPRNG
pub mod os_random;

pub mod seed;

pub mod falcon;
pub mod bls;
pub mod schnorr;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
