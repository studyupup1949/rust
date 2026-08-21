# ab-testing-rs

<<<<<<< HEAD
Rust port of [ab-testing](https://github.com/SuperInstance/ab-testing) — A/B testing with chi-squared and Welch's t-test.

## Features

- **Chi-squared test** for comparing two proportions
- **Welch's t-test** for comparing two continuous samples
- **Confidence intervals** for sample means
- **Experiment** builder with variant management and reporting
=======
Rust port of [ab-testing](https://github.com/SuperInstance/ab-testing) — statistical A/B testing from scratch.

## Features

- **Chi-squared test** for 2×2 contingency tables
- **Welch's t-test** for unequal-variance independent samples
- **Confidence intervals** for proportions and means
- All math from scratch: Lanczos gamma, A&S erf, normal/t CDF approximations
>>>>>>> 4c151e3 (Initial Rust port: statistical A/B testing)

## Usage

```rust
<<<<<<< HEAD
use ab_testing::{Experiment, Variant};

let mut exp = Experiment::new("landing_page");
exp.add_variant(Variant::new("control").with_conversion(100, 200));
exp.add_variant(Variant::new("treatment").with_conversion(150, 200));

let report = exp.report();
println!("Winner: {:?}", report.winner);

// Continuous metrics
let mut exp2 = Experiment::new("revenue");
exp2.add_variant(Variant::new("A").with_values(vec![10.0, 12.0, 15.0, 11.0]));
exp2.add_variant(Variant::new("B").with_values(vec![14.0, 16.0, 18.0, 15.0]));
let welch = exp2.run_welch_t().unwrap();
println!("t={:.3}, p={:.3}", welch.t_statistic, welch.p_value);
=======
use ab_testing::{chi_squared_test, welch_t_test, proportion_ci, mean_ci};

// Chi-squared: compare conversion rates
let result = chi_squared_test(150, 80, 1000, 1000);
assert!(result.significant);

// Welch's t-test: compare continuous metrics
let control = vec![1.0, 2.0, 3.0, 4.0, 5.0];
let treatment = vec![10.0, 11.0, 12.0, 13.0, 14.0];
let result = welch_t_test(&control, &treatment);

// Confidence interval
let ci = proportion_ci(500, 1000, 0.95);
println!("Conversion rate: {:.1}% ± {:.1}%", ci.mean * 100.0, (ci.upper - ci.lower) / 2.0 * 100.0);
>>>>>>> 4c151e3 (Initial Rust port: statistical A/B testing)
```

## License

MIT
