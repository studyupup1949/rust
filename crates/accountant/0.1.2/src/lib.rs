//! # Accountant - Security Research Placeholder
//!
//! This crate is a placeholder registered for security research purposes.
//! It was created to prevent dependency confusion attacks.
//!
//! ## Notice
//!
//! This package name was found to be referenced in a project but was not
//! registered on crates.io, making it vulnerable to dependency confusion attacks.
//!
//! If you are the legitimate owner of a project that uses this crate name,
//! please contact the author to transfer ownership.
//!
//! ## What is Dependency Confusion?
//!
//! Dependency confusion is a supply chain attack where an attacker registers
//! a package name on a public registry that matches an internal/private package
//! name used by an organization. When developers or CI systems build the project,
//! they may inadvertently download the attacker's malicious package instead.
//!
//! ## Security Research
//!
//! This placeholder was registered as part of responsible security research.
//! No malicious code is contained in this package.

/// Placeholder struct for the accountant crate
pub struct Accountant {
    /// A placeholder field
    pub placeholder: bool,
}

impl Accountant {
    /// Creates a new Accountant placeholder instance
    pub fn new() -> Self {
        Self { placeholder: true }
    }
    
    /// Returns a message indicating this is a security research placeholder
    pub fn notice(&self) -> &'static str {
        "This is a security research placeholder. See README for details."
    }
}

impl Default for Accountant {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_placeholder() {
        let acc = Accountant::new();
        assert!(acc.placeholder);
    }
}
