//! # Math Module
//!
//! Provides math-related functions and utilities.
//!
//! ## Getting Started
//!
//! ```rust
//! use actual::math;
//!
//! let m = math::new();
//! ```
//!
//!
//! ## Common Methods
//!
//! ```rust
//! let m = math::math();
//!
//! m.new();
//! m.tables();
//! ```
//!
//! ## Modules
//!
//! - `add`
//! - `cool_patterns`
//! - `call` - Examples and use cases for math functions.
//! - `indices`
//! - `tables`

pub mod add;
pub mod c_ool_patterns;
pub mod call;
pub mod indices;
pub use c_ool_patterns::c_ool_patterns;
pub mod tables;

/// Returns a new instance of 'math' struct, which u can call METHods on.
/// ```rust
/// let math = actual::math::new();
/// math.tables().generate();
///
/// ```
///
pub fn new() -> Math {
    Math {}
}
pub struct Math;

impl Default for Math {
    /// Default function for math struct
    fn default() -> Self {
        Self
    }
}

/// Returns different structs from different math files.
impl Math {
    pub fn new() -> Self {
        Self
    }

    /// Returns the 'Table' struct from tables.rs.
    /// Table struct is used to generate, print and modify tables, their contents and their rows.
    ///
    /// # Table
    /// Table struct that contains 7 fields one being private
    /// struct is private Becauz we dont want user to manually accidently do v.initialized = false
    ///
    ///```rust
    /// pub struct Tables {
    /// pub number : BD // BD here is short for BigDecimal
    /// pub start :
    /// pub struct Tables {
    ///    pub number: BD,
    ///    pub start: BD,
    ///    pub end: BD,
    ///    pub step: BD,
    ///    pub table_data: Vec<(String, String, String)>,
    ///    pub curr: BD,
    ///    initialized: bool,
    ///}
    /// ```
    /// ### Example usage:
    /// ```rust
    /// let math = actual::math::new();
    /// let tables = math.table();
    /// let t1 = tables.clone();
    /// t1.auto_generate();
    /// t1.print();
    /// t1.reset();
    /// t1.print();
    ///```
    /// num = 1
    /// step  1
    /// start = 0
    /// end = 10
    /// curr = start
    /// ====...== (is always as wide as the amount of characters in max row)
    /// 1 *  1  = 1
    /// 1 *  2  = 2
    /// ...........
    /// till end
    /// #### Basic Idea:
    /// The `auto_generate` method automatically takes input, calculates the rows, and fields and returns a complete
    /// struct. using print method on it prints each rows
    ///  
    pub fn table(&self) -> tables::Tables {
        tables::Tables::new()
    }
}
