//! Mersenne Twister PRNG (MT19937)
//!
//! A widely-used pseudorandom number generator with a very long period (2^19937 - 1)
//! and excellent statistical properties. Named after Mersenne prime numbers.
//!
//! This implementation follows the MT19937 (32-bit) specification.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::number_theory::mersenne_twister::MersenneTwister;
//!
//! let mut rng = MersenneTwister::new(12345);
//!
//! // Generate random u32
//! let random_int = rng.next_u32();
//!
//! // Generate random float in [0, 1)
//! let random_float = rng.next_f64();
//! ```

/// Mersenne Twister RNG state
pub struct MersenneTwister {
    state: [u32; 624],
    index: usize,
}

impl MersenneTwister {
    const N: usize = 624;
    const M: usize = 397;
    const MATRIX_A: u32 = 0x9908b0df;
    const UPPER_MASK: u32 = 0x80000000;
    const LOWER_MASK: u32 = 0x7fffffff;
    
    /// Creates a new Mersenne Twister with the given seed
    ///
    /// # Arguments
    ///
    /// * `seed` - Seed value for initialization
    pub fn new(seed: u32) -> Self {
        let mut mt = MersenneTwister {
            state: [0; 624],
            index: 624,
        };
        
        mt.seed(seed);
        mt
    }
    
    /// Seeds the generator
    ///
    /// # Arguments
    ///
    /// * `seed` - New seed value
    pub fn seed(&mut self, seed: u32) {
        self.state[0] = seed;
        
        for i in 1..Self::N {
            self.state[i] = 1812433253u32
                .wrapping_mul(self.state[i - 1] ^ (self.state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        
        self.index = Self::N;
    }
    
    /// Seeds the generator from an array
    ///
    /// # Arguments
    ///
    /// * `key` - Array of seed values
    pub fn seed_from_array(&mut self, key: &[u32]) {
        self.seed(19650218);
        
        let mut i = 1usize;
        let mut j = 0usize;
        let k = Self::N.max(key.len());
        
        for _ in 0..k {
            self.state[i] = (self.state[i]
                ^ ((self.state[i - 1] ^ (self.state[i - 1] >> 30))
                    .wrapping_mul(1664525)))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            
            i += 1;
            j += 1;
            
            if i >= Self::N {
                self.state[0] = self.state[Self::N - 1];
                i = 1;
            }
            
            if j >= key.len() {
                j = 0;
            }
        }
        
        for _ in 0..Self::N - 1 {
            self.state[i] = (self.state[i]
                ^ ((self.state[i - 1] ^ (self.state[i - 1] >> 30))
                    .wrapping_mul(1566083941)))
                .wrapping_sub(i as u32);
            
            i += 1;
            
            if i >= Self::N {
                self.state[0] = self.state[Self::N - 1];
                i = 1;
            }
        }
        
        self.state[0] = 0x80000000;
        self.index = Self::N;
    }
    
    /// Generates the next 32-bit random number
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= Self::N {
            self.twist();
        }
        
        let mut y = self.state[self.index];
        self.index += 1;
        
        // Tempering
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        
        y
    }
    
    /// Generates a random f64 in [0, 1)
    pub fn next_f64(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }
    
    /// Generates a random f64 in [0, 1]
    pub fn next_f64_inclusive(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        
        (a * 67108864.0 + b) * (1.0 / 9007199254740991.0)
    }
    
    /// Generates a random i32
    pub fn next_i32(&mut self) -> i32 {
        self.next_u32() as i32
    }
    
    /// Generates a random u64
    pub fn next_u64(&mut self) -> u64 {
        let high = self.next_u32() as u64;
        let low = self.next_u32() as u64;
        
        (high << 32) | low
    }
    
    /// Generates a random number in the range [0, n)
    pub fn next_range(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        
        // Use rejection sampling to avoid bias
        let threshold = (u32::MAX - n + 1) % n;
        
        loop {
            let value = self.next_u32();
            if value >= threshold {
                return (value - threshold) % n;
            }
        }
    }
    
    /// Generates a random f64 with normal distribution (mean=0, stddev=1)
    /// Uses the Box-Muller transform
    pub fn next_gaussian(&mut self) -> f64 {
        let u1 = self.next_f64();
        let u2 = self.next_f64();
        
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
    
    /// The twist transformation
    fn twist(&mut self) {
        for i in 0..Self::N {
            let x = (self.state[i] & Self::UPPER_MASK)
                | (self.state[(i + 1) % Self::N] & Self::LOWER_MASK);
            
            let mut x_a = x >> 1;
            
            if !x.is_multiple_of(2) {
                x_a ^= Self::MATRIX_A;
            }
            
            self.state[i] = self.state[(i + Self::M) % Self::N] ^ x_a;
        }
        
        self.index = 0;
    }
}

impl Default for MersenneTwister {
    fn default() -> Self {
        Self::new(5489)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reproducibility() {
        let mut rng1 = MersenneTwister::new(12345);
        let mut rng2 = MersenneTwister::new(12345);
        
        for _ in 0..100 {
            assert_eq!(rng1.next_u32(), rng2.next_u32());
        }
    }
    
    #[test]
    fn test_range() {
        let mut rng = MersenneTwister::new(42);
        
        for _ in 0..1000 {
            let value = rng.next_f64();
            assert!(value >= 0.0 && value < 1.0);
        }
    }
    
    #[test]
    fn test_next_range() {
        let mut rng = MersenneTwister::new(123);
        
        for _ in 0..1000 {
            let value = rng.next_range(100);
            assert!(value < 100);
        }
    }
    
    #[test]
    fn test_distribution() {
        let mut rng = MersenneTwister::new(999);
        let mut count_low = 0;
        let n = 10000;
        
        for _ in 0..n {
            if rng.next_f64() < 0.5 {
                count_low += 1;
            }
        }
        
        // Should be roughly 50/50
        let ratio = count_low as f64 / n as f64;
        assert!(ratio > 0.45 && ratio < 0.55);
    }
    
    #[test]
    fn test_gaussian() {
        let mut rng = MersenneTwister::new(777);
        let n = 10000;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        
        for _ in 0..n {
            let x = rng.next_gaussian();
            sum += x;
            sum_sq += x * x;
        }
        
        let mean = sum / n as f64;
        let variance = (sum_sq / n as f64) - (mean * mean);
        
        // Mean should be close to 0, variance close to 1
        assert!(mean.abs() < 0.1);
        assert!((variance - 1.0).abs() < 0.1);
    }
}
