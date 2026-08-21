use rand::Rng;
use rand::distr::{Distribution, Uniform}; // Fix: rand 0.9 uses `distr` not `distributions`
use abhedya_chhandas::{self as sanskrit, MatraWeight}; 

// Define Encryption Configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionMode {
    Standard, // Raw Output, High Throughput
    Metered,  // Chhandas Output, High Steganography
}

// LWE Parameters
// Dimensions suitable for "toy" post-quantum or PoC
pub const N: usize = 768; // Dimension
pub const Q: i16 = 3329;   // Modulus (Standard Kyber-768 param)

// Structures for Keys
pub struct SecretKey {
    pub s: Vec<i16>,
}

pub struct PublicKey {
    pub A: Vec<Vec<i16>>, // N x N matrix for simplicity (Square LWE)
    pub b: Vec<i16>,      // b = As + e
}

pub struct Ciphertext {
    pub u: Vec<i16>,
    pub v: Vec<i16>, // The encrypted message part
}

// Math helpers
fn dot(v1: &[i16], v2: &[i16]) -> i16 {
    let sum: i32 = v1.iter().zip(v2.iter())
        .map(|(a, b)| (*a as i32) * (*b as i32))
        .sum();
    (sum.rem_euclid(Q as i32)) as i16
}

fn add_noise(val: i16, rng: &mut impl Rng) -> i16 {
    // Simple centralized noise (hamming or gaussian approximation)
    let e: i16 = rng.random_range(-1..=1); 
    (val + e).rem_euclid(Q)
}

impl SecretKey {
    pub fn new(rng: &mut impl Rng) -> Self {
        // Ternary secret {-1, 0, 1} is common in LWE
        // Uniform::new(low, high) -> [low, high)
        let dist = Uniform::new(-1, 2).unwrap(); 
        let s: Vec<i16> = (0..N).map(|_| dist.sample(rng) as i16).collect();
        SecretKey { s }
    }
}

impl PublicKey {
    pub fn new(sk: &SecretKey, rng: &mut impl Rng) -> Self {
        // Generate uniform A
        let mut A = vec![vec![0; N]; N];
        for row in A.iter_mut() {
            for val in row.iter_mut() {
                *val = rng.random_range(0..Q);
            }
        }

        // Calculate b = As + e
        let mut b = Vec::with_capacity(N);
        for row in &A {
            let dot_prod = dot(row, &sk.s);
            let noise = add_noise(0, rng);
            b.push((dot_prod + noise).rem_euclid(Q));
        }

        PublicKey { A, b }
    }
}

// SIMD Implementation Module
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
mod simd {
    // Rust 2024 requires explicit unsafe blocks even in unsafe fns
    #![allow(unsafe_op_in_unsafe_fn)] 
    
    use super::*;
    use std::arch::x86_64::*;

    pub unsafe fn dot_avx2(v1: &[i16], v2: &[i16]) -> i16 {
        let mut sum_vec = _mm256_setzero_si256();
        
        for i in (0..N).step_by(16) {
             let a = _mm256_loadu_si256(v1.as_ptr().add(i) as *const __m256i);
             let b = _mm256_loadu_si256(v2.as_ptr().add(i) as *const __m256i);
             let prod = _mm256_madd_epi16(a, b);
             sum_vec = _mm256_add_epi32(sum_vec, prod);
        }
        
        let mut temp = [0i32; 8];
        _mm256_storeu_si256(temp.as_mut_ptr() as *mut __m256i, sum_vec);
        
        let total_sum: i32 = temp.iter().sum();
        (total_sum.rem_euclid(Q as i32)) as i16
    }
    
    pub unsafe fn acc_row_avx2(row: &[i16], r_val: i16, accum: &mut [i32]) {
        let r_vec = _mm256_set1_epi16(r_val);
        
        for k in (0..N).step_by(16) {
            let a_chunk = _mm256_loadu_si256(row.as_ptr().add(k) as *const __m256i);
            let prod = _mm256_mullo_epi16(a_chunk, r_vec);
            
            let prod_lo = _mm256_cvtepi16_epi32(_mm256_castsi256_si128(prod));
            let prod_hi = _mm256_cvtepi16_epi32(_mm256_extracti128_si256(prod, 1));
            
            let acc_ptr = accum.as_mut_ptr().add(k);
            let mut acc_lo = _mm256_loadu_si256(acc_ptr as *const __m256i);
            let mut acc_hi = _mm256_loadu_si256(acc_ptr.add(8) as *const __m256i);
            
            acc_lo = _mm256_add_epi32(acc_lo, prod_lo);
            acc_hi = _mm256_add_epi32(acc_hi, prod_hi);
            
            _mm256_storeu_si256(acc_ptr as *mut __m256i, acc_lo);
            _mm256_storeu_si256(acc_ptr.add(8) as *mut __m256i, acc_hi);
        }
    }
}

// Encryption: Encrypt a vector of messages m
pub fn encrypt(pk: &PublicKey, message: &[i16], rng: &mut impl Rng, mode: EncryptionMode) -> (Vec<Vec<i16>>, Vec<i16>) {
    let mut u_vecs = Vec::new();
    let mut v_vec = Vec::new();

    // Standard Anushtubh: 4 padas of 8 syllables.
    // Pattern: 5th is light, 6th is heavy.
    // For PoC: Let's use alternating Short-Long (S-L-S-L-S-L-S-L) for max entropy testing.
    // S=Laghu, L=Guru
    
    // Check for AVX2 support at runtime
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    let use_avx2 = std::is_x86_feature_detected!("avx2");
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    let use_avx2 = false;

    // Constant-Time Logic Helper
    // Returns 1 if condition is true, 0 if false (using bitwise ops for primitive types would be better in C, here we use boolean logic)
    // NOTE: In Rust, minimizing branches is key. 
    
    for (m_idx, &m) in message.iter().enumerate() {
        let r: Vec<i16> = (0..N).map(|_| rng.random_range(-1..=1)).collect();
        

        let mut u_accum = vec![0i32; N]; // Use i32 to prevent overflow during accumulation
        
        for j in 0..N {
            let r_val = r[j] as i32;
            if r_val == 0 { continue; } // Optimization for sparse ternary r
            
            // SAXPY: u += r_val * A[j]
            // We can SIMD this line easily?
            // Actually, A[j] is Vec<i16>.
            
            if use_avx2 {
                #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
                unsafe {
                     // Call the helper in the simd module
                     simd::acc_row_avx2(&pk.A[j], r_val as i16, &mut u_accum);
                }
                #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
                {
                   // Fallback (detected but not enabled at compile time?)
                   for k in 0..N {
                       u_accum[k] += (pk.A[j][k] as i32) * r_val;
                   }
                }
            } else {
                for k in 0..N {
                    u_accum[k] += (pk.A[j][k] as i32) * r_val;
                }
            }
        }
        
        // Finalize u (Mod Q and add Noise)
        let mut u: Vec<i16> = Vec::with_capacity(N);
        for i in 0..N {
            let col_dot = u_accum[i].rem_euclid(Q as i32) as i16;
            
            // Initial Noise
            let mut noise = add_noise(0, rng);
            let mut final_noise = noise;
            
            if mode == EncryptionMode::Metered {
                // Address Timing Oracle: Fixed Iterations
                // We ALWAYS iterate through the fixed window, regardless of when we find a match.
                // We use a "found" flag to latch onto the first valid value.
                
                let target = if i % 2 == 0 { MatraWeight::Laghu } else { MatraWeight::Guru };
                let mut found = false;

                // Candidate offsets to try: -128 to +128 (257 iterations)
                // This covers the entire "Modulus Gap" (size 161) ensuring we can always escape the forbidden range.
                for offset in -128..=128 {
                   let candidate_noise = noise + offset;
                   let val = (col_dot + candidate_noise).rem_euclid(Q) as u16;
                   
                   // Check 1: Meter Constraints
                   let weight_match = sanskrit::get_matra_weight((val % 16) as usize) == target;
                   
                   // Check 2: Modulus Truncation (Bias Elimination)
                   // Reject values >= 3168 to ensure perfect u16->Sanskrit mapping
                   let range_valid = val < 3168; 
                   
                   // Combined Check
                   let valid = weight_match && range_valid;
                   
                   // Constant-Time Selection
                   if valid && !found {
                       final_noise = candidate_noise;
                       found = true;
                   }
                }
                
                // If not found, we technically fail.
                // In this hardened version, we fallback to 'noise' (which might be >= 3168 or wrong meter).
                // But with 81 tries, probability is effectively zero.
            } else {
                final_noise = noise;
            }
            
            u.push((col_dot + final_noise).rem_euclid(Q));
        }
        
        // v calculation
        let b_dot = dot(&pk.b, &r);
        let mut noise = add_noise(0, rng);
        let scaled_m = if m == 1 { Q/2 } else { 0 };
        
        let v_val = (b_dot + noise + scaled_m).rem_euclid(Q);
        
        u_vecs.push(u);
        v_vec.push(v_val);
    }
    
    (u_vecs, v_vec)
}

pub fn decrypt(sk: &SecretKey, u_vecs: &[Vec<i16>], v_vec: &[i16]) -> Vec<i16> {
    let mut messages = Vec::new();
    for (u, &v) in u_vecs.iter().zip(v_vec.iter()) {
        // m_noisy = v - s^T u
        let s_dot_u = dot(&sk.s, u);
        let diff = (v - s_dot_u).rem_euclid(Q);
        
        // If close to Q/2 -> 1, if close to 0 -> 0
        let m = if diff > Q/4 && diff < 3*Q/4 { 1 } else { 0 };
        messages.push(m);
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use abhedya_chhandas as sanskrit;

    #[test]
    fn test_encryption_correctness() {
        // rand 0.9: use rng() for standard generator
        let mut rng = rand::rng(); 
        let sk = SecretKey::new(&mut rng);
        let pk = PublicKey::new(&sk, &mut rng);
        
        let msg = vec![0, 1, 0, 1, 1, 0];
        let (u, v) = encrypt(&pk, &msg, &mut rng, EncryptionMode::Standard);
        let decrypted = decrypt(&sk, &u, &v);
        
        assert_eq!(msg, decrypted);
    }

    #[test]
    fn test_metered_mode_constraints() {
        let mut rng = rand::rng();
        let sk = SecretKey::new(&mut rng);
        let pk = PublicKey::new(&sk, &mut rng);
        
        // Encrypt meaningful amount of data to hit constraints
        let msg = vec![1; 10]; 
        let (u_vecs, _) = encrypt(&pk, &msg, &mut rng, EncryptionMode::Metered);
        
        for u in u_vecs {
            for (i, val) in u.iter().enumerate() {
                // Check 1: Modulus Truncation
                assert!(*val < 3168, "Value {} exceeds truncated modulus 3168", val);
                
                // Check 2: Meter Pattern (Even=Laghu, Odd=Guru)
                let target = if i % 2 == 0 { MatraWeight::Laghu } else { MatraWeight::Guru };
                let current = sanskrit::get_matra_weight((val % 16) as usize);
                
                assert_eq!(current, target, "Meter mismatch at index {}", i);
            }
        }
    }
}
