// Filename: Cargo.toml

// [profile.dev]
// opt-level = 0

// [profile.release]
// opt-level = 3

// The opt-level setting controls the number of optimizations Rust will apply to your code, with a range of 0 to 3. 
// Applying more optimizations extends compiling time, so if you’re in development and compiling your code often, 
// you’ll want fewer optimizations to compile faster even if the resulting code runs slower. 
// The default opt-level for dev is therefore 0. When you’re ready to release your code, it’s best to spend more time compiling. 
// You’ll only compile in release mode once, but you’ll run the compiled program many times, 
// so release mode trades longer compile time for code that runs faster. 
// That is why the default opt-level for the release profile is 3.