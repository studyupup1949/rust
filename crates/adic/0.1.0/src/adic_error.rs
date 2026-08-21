#[derive(Debug, Clone, Copy, PartialEq)]
/// Error from Adic operations
pub enum AdicError {
    /// Error that involves mixing adic's with different primes
    MixedCharacteristic,
    /// Error that results in an invalid adic number, e.g. sqrt(2) in 5-adics
    NotAnAdic,
    /// Error that results when an operation is requested that is not yet implemented
    NotImplemented,
}
