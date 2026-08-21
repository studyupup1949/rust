// ABOUTME: Anomaly detection modules for different tiers of suspicious behavior
// ABOUTME: Contains temporal, signal, identity, behavioral, and ML detection algorithms

pub mod behavior;
pub mod identity;
pub mod ml;
pub mod signal;
pub mod temporal;

#[cfg(test)]
mod integration_test;
