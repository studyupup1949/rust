//! Example demonstrating number theory and cryptography algorithms

use advanced_algorithms::number_theory::{miller_rabin, extended_euclidean, mersenne_twister};

fn main() {
    println!("=== Number Theory & Cryptography Demo ===\n");
    
    // Miller-Rabin Primality Testing
    println!("=== Miller-Rabin Primality Test ===");
    
    let test_numbers = vec![
        (17, true),
        (561, false), // Carmichael number
        (1_000_000_007, true),
        (1_000_000_009, true),
        (1_000_000_000, false),
    ];
    
    println!("Testing primality:");
    for (n, expected) in test_numbers {
        let is_prime = miller_rabin::is_prime(n, 20);
        let symbol = if is_prime { "✓" } else { "✗" };
        println!("  {} {} - {}", symbol, n, if is_prime { "prime" } else { "composite" });
        assert_eq!(is_prime, expected);
    }
    
    // Generate random primes
    println!("\nGenerating random primes:");
    for bits in [16, 20, 24] {
        let prime = miller_rabin::generate_prime(bits, 20);
        let verified = miller_rabin::is_prime_deterministic(prime);
        println!("  {}-bit prime: {} (verified: {})", bits, prime, verified);
    }
    
    // RSA key generation simulation (simplified)
    println!("\n=== Simulated RSA Key Generation ===");
    
    let p = miller_rabin::generate_prime(16, 20);
    let q = miller_rabin::generate_prime(16, 20);
    let n = p * q;
    let phi = (p - 1) * (q - 1);
    
    println!("Prime p: {}", p);
    println!("Prime q: {}", q);
    println!("Modulus n = p*q: {}", n);
    println!("φ(n) = (p-1)(q-1): {}", phi);
    
    // Choose public exponent e (commonly 65537)
    let e = 65537i64;
    
    // Compute private exponent d = e^(-1) mod φ(n)
    match extended_euclidean::mod_inverse(e, phi as i64) {
        Ok(d) => {
            println!("\nPublic key: (e={}, n={})", e, n);
            println!("Private key: (d={}, n={})", d, n);
            
            // Verify: e * d ≡ 1 (mod φ(n))
            let verify = (e * d) % phi as i64;
            println!("Verification: e*d mod φ(n) = {} (should be 1)", verify);
        }
        Err(e) => println!("Error computing private key: {}", e),
    }
    
    // Extended Euclidean Algorithm
    println!("\n=== Extended Euclidean Algorithm ===");
    
    let a = 240;
    let b = 46;
    
    let result = extended_euclidean::extended_gcd(a, b);
    
    println!("GCD({}, {}):", a, b);
    println!("  gcd = {}", result.gcd);
    println!("  Bézout coefficients: x = {}, y = {}", result.x, result.y);
    println!("  Verification: {}*{} + {}*{} = {}", 
             a, result.x, b, result.y, a * result.x + b * result.y);
    
    // Modular arithmetic examples
    println!("\nModular Inverses:");
    let examples = vec![(3, 11), (7, 26), (17, 43)];
    
    for (a, m) in examples {
        match extended_euclidean::mod_inverse(a, m) {
            Ok(inv) => {
                println!("  {}^(-1) mod {} = {} (verify: {}*{} mod {} = {})",
                         a, m, inv, a, inv, m, (a * inv) % m);
            }
            Err(_) => {
                println!("  {}^(-1) mod {} doesn't exist (not coprime)", a, m);
            }
        }
    }
    
    // Linear congruences
    println!("\nSolving Linear Congruences:");
    
    // Solve 3x ≡ 6 (mod 9)
    let solutions = extended_euclidean::solve_linear_congruence(3, 6, 9);
    if !solutions.is_empty() {
        println!("  3x ≡ 6 (mod 9):");
        println!("    Solutions: {:?}", solutions);
        for &x in &solutions {
            println!("    Verify: 3*{} mod 9 = {}", x, (3 * x) % 9);
        }
    }
    
    // GCD and LCM
    println!("\nGCD and LCM:");
    let numbers = vec![48, 18, 30];
    let gcd_result = extended_euclidean::gcd_multiple(&numbers);
    let lcm_result = extended_euclidean::lcm_multiple(&numbers);
    
    println!("  Numbers: {:?}", numbers);
    println!("  GCD: {}", gcd_result);
    println!("  LCM: {}", lcm_result);
    
    // Mersenne Twister PRNG
    println!("\n=== Mersenne Twister PRNG ===");
    
    let mut rng = mersenne_twister::MersenneTwister::new(12345);
    
    println!("Random integers:");
    for i in 0..5 {
        println!("  {}: {}", i + 1, rng.next_u32());
    }
    
    println!("\nRandom floats [0, 1):");
    for i in 0..5 {
        println!("  {}: {:.6}", i + 1, rng.next_f64());
    }
    
    println!("\nRandom range [0, 100):");
    for i in 0..5 {
        println!("  {}: {}", i + 1, rng.next_range(100));
    }
    
    println!("\nGaussian (normal) distribution:");
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let n = 10000;
    
    for _ in 0..n {
        let x = rng.next_gaussian();
        sum += x;
        sum_sq += x * x;
    }
    
    let mean = sum / n as f64;
    let variance = (sum_sq / n as f64) - (mean * mean);
    let stddev = variance.sqrt();
    
    println!("  Generated {} samples", n);
    println!("  Mean: {:.4} (expected: 0.0)", mean);
    println!("  Std dev: {:.4} (expected: 1.0)", stddev);
    
    // Test reproducibility
    println!("\nReproducibility test:");
    let mut rng1 = mersenne_twister::MersenneTwister::new(42);
    let mut rng2 = mersenne_twister::MersenneTwister::new(42);
    
    let same = (0..10).all(|_| rng1.next_u32() == rng2.next_u32());
    println!("  Same seed produces same sequence: {}", same);
    
    // Statistical test: uniform distribution
    println!("\nUniformity test:");
    let mut rng = mersenne_twister::MersenneTwister::new(999);
    let mut buckets = vec![0; 10];
    let total = 100000;
    
    for _ in 0..total {
        let val = rng.next_f64();
        let bucket = (val * 10.0) as usize;
        if bucket < 10 {
            buckets[bucket] += 1;
        }
    }
    
    println!("  Distribution across 10 buckets ({} samples):", total);
    for (i, &count) in buckets.iter().enumerate() {
        let expected = total / 10;
        let deviation = ((count as f64 - expected as f64) / expected as f64 * 100.0).abs();
        println!("    Bucket {}: {} ({:.1}% deviation from expected)", 
                 i, count, deviation);
    }
}
