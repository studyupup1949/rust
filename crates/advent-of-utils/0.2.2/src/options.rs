use std::fmt::Debug;
use std::fmt::Display;
use std::num::TryFromIntError;

/// Represents different types of option values that can be returned from your Advent of Code solutions.
/// This enum provides a type-safe way to handle different value types that might be
/// encountered as results.
///
/// # Variants
/// - `Str(String)` - Contains a String value for text-based results
/// - `Int(i64)` - Contains a 64-bit signed integer value for numeric results
/// - `None` - Represents the absence of a value or an unset option
///
/// # Automatic Conversions
/// This type implements `From` for multiple common types to make working with Advent of Code inputs easier:
/// - `String` and `&str` -> `AocOption::Str`
/// - Small numeric types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `isize`, `usize`) -> `AocOption::Int`
/// - `Option<T>` where T implements `Into<AocOption>`
///
/// For larger numeric types, `TryFrom` is implemented to handle potential conversion failures:
/// - `u64`, `i128`, `u128` -> `Result<AocOption::Int, TryFromIntError>`
///
/// # Example
/// ```rust
/// let string_opt: AocOption = "puzzle input".to_string().into();   // Creates AocOption::Str
/// let num_opt: AocOption = 42_i32.into();                          // Creates AocOption::Int
/// let some_opt: AocOption = Some(42_i32).into();                   // Creates AocOption::Int
/// let none_opt: AocOption = None::<String>.into();                 // Creates AocOption::None
///
/// // Using TryFrom for larger numbers
/// let big_num = u64::MAX;
/// let result = AocOption::try_from(big_num);                       // Returns Result
/// ```
#[derive(Clone, Hash, Eq, PartialEq, Default)]
pub enum AocOption {
    /// Contains a String value for text-based results
    Str(String),
    /// Contains a 128-bit signed integer value for numeric results
    Int(i64),
    /// Represents the absence of a value or an unset option
    #[default]
    None,
}

impl From<String> for AocOption {
    fn from(value: String) -> Self {
        AocOption::Str(value)
    }
}

impl From<&str> for AocOption {
    fn from(value: &str) -> Self {
        AocOption::Str(value.to_string())
    }
}

macro_rules! impl_from_number {
    ($($t:ty),*) => {
        $(
            impl From<$t> for AocOption {
                fn from(value: $t) -> Self {
                    AocOption::Int(value as i64)
                }
            }
        )*
    }
}

impl_from_number!(i8, i16, i32, i64, u8, u16, u32, isize, usize);

impl<T: Into<AocOption>> From<Option<T>> for AocOption {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => AocOption::None,
        }
    }
}

macro_rules! impl_try_from_number {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for AocOption {
                fn try_from(value: $t) -> Result<Self, Self::Error> {
                    Ok(AocOption::Int(i64::try_from(value)?))
                }

                type Error = TryFromIntError;
            }
        )*
    }
}

impl_try_from_number!(u64, i128, u128);

impl Debug for AocOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AocOption::Str(s) => write!(f, "Str: {}", s),
            AocOption::Int(i) => write!(f, "Int: {}", i),
            AocOption::None => write!(f, "None"),
        }
    }
}

impl Display for AocOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AocOption::Str(s) => write!(f, "{}", s),
            AocOption::Int(i) => write!(f, "{}", i),
            AocOption::None => write!(f, "None"),
        }
    }
}
