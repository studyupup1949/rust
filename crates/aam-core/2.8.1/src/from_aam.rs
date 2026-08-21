use crate::aam::AAM;
use crate::aaml::parsing;
use crate::error::{AamlError, ErrorDiagnostics};
use std::collections::HashMap;

/// Trait for deserializing Rust types from AAM string values.
///
/// `from_aam_str` uses the **same AAM pipeline engine** as `AAM::parse`.
/// Two formats are supported:
///
/// 1. **Inline object** — `{ field = value, ... }`
/// 2. **Full AAM document** — `key = value\n...`
///
/// # Provided implementations
///
/// Built-in: `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`,
/// `usize`, `f32`, `f64`, `bool`, `String`.
///
/// Generic: `Option<T>`, `Vec<T>`, `HashMap<String, T>`.
///
/// # Manual implementation
///
/// ```
/// use aam_core::from_aam::FromAam;
/// use aam_core::from_aam::parse_fields;
/// use aam_core::error::AamlError;
///
/// struct Point { x: f64, y: f64 }
///
/// impl FromAam for Point {
///     fn from_aam_str(value: &str) -> Result<Self, AamlError> {
///         let fields = parse_fields(value)?;
///         Ok(Point {
///             x: f64::from_aam_str(fields.get("x").ok_or_else(|| {
///                 AamlError::InvalidValue { details: "missing x".into(), expected: "number".into(), diagnostics: None }
///             })?)?,
///             y: f64::from_aam_str(fields.get("y").ok_or_else(|| {
///                 AamlError::InvalidValue { details: "missing y".into(), expected: "number".into(), diagnostics: None }
///             })?)?,
///         })
///     }
/// }
///
/// let p = Point::from_aam_str("x = 1.5\ny = 2.5\n").unwrap();
/// assert_eq!(p.x, 1.5);
/// ```
///
/// # Derive macro (requires `aam-derive` / `aam-rs`)
///
/// In `Cargo.toml`: `aam-rs = "2"`\
/// In code: `use aam_rs::FromAam; use aam_rs::from_aam::FromAam as _;`
///
/// ```ignore
/// use aam_rs::FromAam;
/// use aam_rs::from_aam::FromAam as _;
///
/// #[derive(Debug, Clone, FromAam, PartialEq)]
/// struct Server { host: String, port: i32, debug: Option<bool>, tags: Vec<String> }
///
/// let s = Server::from_aam_str("host = localhost\nport = 8080\ntags = [api, worker]\n").unwrap();
///
/// #[derive(Debug, Clone, FromAam, PartialEq)]
/// struct Address { host: String, port: i32 }
///
/// #[derive(Debug, Clone, FromAam, PartialEq)]
/// struct App { name: String, address: Address }
///
/// let app = App::from_aam_str("name = proxy\naddress = { host = 10.0.0.1, port = 3128 }\n").unwrap();
/// ```
pub trait FromAam: Sized {
    /// Deserializes a value from an AAM string.
    ///
    /// # Errors
    ///
    /// Returns `AamlError` if the string cannot be parsed into `Self`.
    fn from_aam_str(value: &str) -> Result<Self, AamlError>;
}

impl FromAam for String {
    fn from_aam_str(value: &str) -> Result<Self, AamlError> {
        Ok(value.to_string())
    }
}

macro_rules! impl_from_aam_parse {
    ($($t:ty),* $(,)?) => {
        $(
            impl FromAam for $t {
                fn from_aam_str(value: &str) -> Result<Self, AamlError> {
                    value.trim().parse().map_err(|e| AamlError::InvalidValue {
                        details: format!("cannot parse '{}' as {}: {}", value, stringify!($t), e),
                        expected: format!("valid {}", stringify!($t)),
                        diagnostics: None,
                    })
                }
            }
        )*
    };
}

impl_from_aam_parse!(
    i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, bool
);

impl<T: FromAam> FromAam for Option<T> {
    fn from_aam_str(value: &str) -> Result<Self, AamlError> {
        let v = value.trim();
        if v.is_empty() {
            return Ok(None);
        }
        T::from_aam_str(value).map(Some)
    }
}

impl<T: FromAam> FromAam for Vec<T> {
    fn from_aam_str(value: &str) -> Result<Self, AamlError> {
        let v = value.trim();
        if v.starts_with('[') && v.ends_with(']') {
            let items = parse_list_items(v)?;
            return items.iter().map(|s| T::from_aam_str(s)).collect();
        }
        T::from_aam_str(v).map(|item| vec![item])
    }
}

impl<T: FromAam, S: ::std::hash::BuildHasher + Default> FromAam for HashMap<String, T, S> {
    fn from_aam_str(value: &str) -> Result<Self, AamlError> {
        let fields = parse_object_fields(value)?;
        fields
            .into_iter()
            .map(|(k, v)| T::from_aam_str(&v).map(|tv| (k, tv)))
            .collect()
    }
}

/// Parses the fields of an inline AAM object `{ k = v, ... }` into a map.
///
/// # Errors
///
/// Returns an error if the value is not a valid inline object.
pub fn parse_object_fields(value: &str) -> Result<HashMap<String, String>, AamlError> {
    Ok(parsing::parse_inline_object(value)?.into_iter().collect())
}

/// Parses key-value pairs using the **AAM pipeline** (`AAM::parse`).
///
/// - Inline objects `{ k = v, ... }` are wrapped as `_ = { ... }` before
///   entering the pipeline, then unwrapped.
/// - Full documents `k = v\n...` go through the pipeline directly.
///
/// # Errors
///
/// Returns an error if the value cannot be parsed into key-value pairs.
pub fn parse_fields(value: &str) -> Result<HashMap<String, String>, AamlError> {
    let v = value.trim();

    // Inline objects are parsed directly so that whitespace inside values is
    // preserved. Rounding them through the AAM pipeline strips syntactic
    // spaces and can collapse values like `export CC=gcc` into `exportCC=gcc`.
    if v.starts_with('{') && v.ends_with('}') {
        return parse_object_fields(v);
    }

    let aam = AAM::parse(v).map_err(|mut errs| errs.remove(0))?;
    let map: HashMap<String, String> = aam
        .iter()
        .map(|(k, val)| (k.to_string(), val.to_string()))
        .collect();
    Ok(map)
}

/// Parses a list literal `[a, b, c]` through the **AAM pipeline**.
///
/// # Errors
///
/// Returns an error if the value is not a valid list literal.
pub fn parse_list_items(value: &str) -> Result<Vec<String>, AamlError> {
    let v = value.trim();
    let input = format!("_ = {v}");
    let aam = AAM::parse(&input).map_err(|mut errs| errs.remove(0))?;
    let raw = aam.get("_").unwrap_or("");
    crate::types_aam::list::ListType::parse_items(raw).ok_or_else(|| AamlError::InvalidValue {
        details: format!("'{v}' is not a valid list literal"),
        expected: "[item, item, ...] format".to_string(),
        diagnostics: None,
    })
}

/// Retrieves and deserializes a value from an [`AAM`] document by key.
///
/// # Errors
///
/// Returns an error if the key is not found or the value cannot be parsed into `T`.
pub fn get_aam<T: FromAam>(aam: &AAM, key: &str) -> Result<T, AamlError> {
    let value = aam.get(key).ok_or_else(|| AamlError::NotFound {
        key: key.to_string(),
        context: "AAM map".to_string(),
        diagnostics: Some(Box::new(ErrorDiagnostics::new(
            "Key not found",
            format!("Key '{key}' not found in AAM document"),
            "Check that the key exists and the document was parsed correctly",
        ))),
    })?;
    T::from_aam_str(value)
}

/// Retrieves and deserializes an optional value from an [`AAM`] document by key.
///
/// # Errors
///
/// Returns an error if the key is found but the value cannot be parsed into `T`.
pub fn get_opt_aam<T: FromAam>(aam: &AAM, key: &str) -> Result<Option<T>, AamlError> {
    aam.get(key)
        .map_or_else(|| Ok(None), |value| T::from_aam_str(value).map(Some))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert_eq!(i32::from_aam_str("42").unwrap(), 42);
        assert!((f64::from_aam_str("3.14").unwrap() - std::f64::consts::PI).abs() < 0.01);
        assert!(bool::from_aam_str("true").unwrap());
        assert_eq!(String::from_aam_str("hello").unwrap(), "hello");
    }

    #[test]
    fn test_option() {
        assert_eq!(Option::<i32>::from_aam_str("42").unwrap(), Some(42));
        assert_eq!(Option::<i32>::from_aam_str("").unwrap(), None);
    }

    #[test]
    fn test_vec() {
        assert_eq!(
            Vec::<i32>::from_aam_str("[1, 2, 3]").unwrap(),
            vec![1, 2, 3]
        );
        assert_eq!(
            Vec::<String>::from_aam_str("[a, b, c]").unwrap(),
            vec!["a", "b", "c"]
        );
        assert_eq!(Vec::<i32>::from_aam_str("42").unwrap(), vec![42]);
    }

    #[test]
    fn test_parse_object_fields() {
        let fields = parse_object_fields("{ x = 1.0, y = 2.0, name = hello }").unwrap();
        assert_eq!(fields.get("x").unwrap(), "1.0");
        assert_eq!(fields.get("y").unwrap(), "2.0");
        assert_eq!(fields.get("name").unwrap(), "hello");
    }

    #[test]
    fn test_parse_list_items() {
        let items = parse_list_items("[rust, aam, config]").unwrap();
        assert_eq!(items, vec!["rust", "aam", "config"]);
    }

    #[test]
    fn test_parse_fields_inline() {
        let fields = parse_fields("{ x = 1.0, y = 2.0 }").unwrap();
        assert_eq!(fields.get("x").unwrap(), "1.0");
        assert_eq!(fields.get("y").unwrap(), "2.0");
    }

    #[test]
    fn test_parse_fields_full_aam() {
        let fields = parse_fields("x = 1.0\ny = 2.0\n").unwrap();
        assert_eq!(fields.get("x").unwrap(), "1.0");
        assert_eq!(fields.get("y").unwrap(), "2.0");
    }

    #[test]
    fn test_parse_fields_skips_directives_and_comments() {
        let fields = parse_fields(
            "@schema Point { x: f64, y: f64 }\n\n# this is a comment\nx = 1.0\n\ny = 2.0\n",
        )
        .unwrap();
        assert_eq!(fields.get("x").unwrap(), "1.0");
        assert_eq!(fields.get("y").unwrap(), "2.0");
    }

    #[test]
    fn test_get_aam() {
        let aam = AAM::parse("x = 1.5\ny = 2.5\n").unwrap();
        let x: f64 = get_aam(&aam, "x").unwrap();
        assert_eq!(x, 1.5);
    }

    #[test]
    fn test_get_opt_aam() {
        let aam = AAM::parse("name = hello\n").unwrap();
        let name: Option<String> = get_opt_aam(&aam, "name").unwrap();
        assert_eq!(name, Some("hello".to_string()));
        let missing: Option<String> = get_opt_aam(&aam, "missing").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_manual_full_aam_format() {
        struct Point {
            x: f64,
            y: f64,
        }

        impl FromAam for Point {
            fn from_aam_str(value: &str) -> Result<Self, AamlError> {
                let fields = parse_fields(value)?;
                Ok(Self {
                    x: fields
                        .get("x")
                        .map(|s| f64::from_aam_str(s))
                        .ok_or_else(|| AamlError::NotFound {
                            key: "x".to_string(),
                            context: "Point".to_string(),
                            diagnostics: None,
                        })??,
                    y: fields
                        .get("y")
                        .map(|s| f64::from_aam_str(s))
                        .ok_or_else(|| AamlError::NotFound {
                            key: "y".to_string(),
                            context: "Point".to_string(),
                            diagnostics: None,
                        })??,
                })
            }
        }

        let p = Point::from_aam_str("x = 1.5\ny = 2.5\n").unwrap();
        assert_eq!(p.x, 1.5);
        assert_eq!(p.y, 2.5);

        let p2 = Point::from_aam_str("{ x = 3.0, y = 4.0 }").unwrap();
        assert_eq!(p2.x, 3.0);
        assert_eq!(p2.y, 4.0);
    }
}
