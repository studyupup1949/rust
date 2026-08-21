//! # Accountant
//!
//! A simple accounting utilities library for financial calculations.
//!
//! This crate provides basic structures and functions for handling
//! accounting operations in Rust applications.

/// Accountant struct for basic financial operations
pub struct Accountant {
    /// Current balance
    pub balance: f64,
}

impl Accountant {
    /// Creates a new Accountant with zero balance
    pub fn new() -> Self {
        Self { balance: 0.0 }
    }
    
    /// Creates an Accountant with initial balance
    pub fn with_balance(balance: f64) -> Self {
        Self { balance }
    }
    
    /// Adds amount to balance
    pub fn credit(&mut self, amount: f64) {
        self.balance += amount;
    }
    
    /// Subtracts amount from balance
    pub fn debit(&mut self, amount: f64) {
        self.balance -= amount;
    }
    
    /// Returns the current balance
    pub fn get_balance(&self) -> f64 {
        self.balance
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
    fn test_new_accountant() {
        let acc = Accountant::new();
        assert_eq!(acc.balance, 0.0);
    }
    
    #[test]
    fn test_credit() {
        let mut acc = Accountant::new();
        acc.credit(100.0);
        assert_eq!(acc.get_balance(), 100.0);
    }
}
