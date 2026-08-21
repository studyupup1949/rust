//! Transformation utilities — `Adapt` trait, `Pipeline`, and `FieldMapper`.
//!
//! This module provides dynamic structures and traits to map and remodel data.

use crate::error::Error;
use crate::value::Value;
use std::collections::BTreeMap;

/// Convert a value of type `T` into `Self`.
///
/// Implement this trait to construct mapping layers between different data representations
/// (e.g. mapping database record types directly into presentation models).
pub trait Adapt<T>: Sized {
    /// Transforms the input type into the target type, returning an error on failure.
    fn adapt(value: T) -> Result<Self, Error>;
}

type PipelineStep = Box<dyn Fn(Value) -> Result<Value, Error>>;

/// A sequential pipeline of dynamic [`Value`] → [`Value`] transformation steps.
///
/// Execution stops and bubbles up the error immediately if any intermediate mapping step fails.
#[derive(Default)]
pub struct Pipeline {
    steps: Vec<PipelineStep>,
}

impl Pipeline {
    /// Create a new, empty transformation pipeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a mapping function step onto the end of this pipeline.
    pub fn step(mut self, f: impl Fn(Value) -> Result<Value, Error> + 'static) -> Self {
        self.steps.push(Box::new(f));
        self
    }

    /// Evaluates the input through each pipeline step in sequence, returning the final transformed [`Value`].
    pub fn run(&self, input: Value) -> Result<Value, Error> {
        let mut current = input;
        for step in &self.steps {
            current = step(current)?;
        }
        Ok(current)
    }
}

/// Remaps field keys in a [`Value::Object`] based on explicit registry entries.
///
/// Keys not explicitly present in the mapper bypass mapping and are copied untouched.
#[derive(Default)]
pub struct FieldMapper {
    mappings: BTreeMap<String, String>,
}

impl FieldMapper {
    /// Creates a new, empty `FieldMapper` registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a field rename operation from `from` key name to `to` key name.
    pub fn map(mut self, from: &str, to: &str) -> Self {
        self.mappings.insert(from.to_string(), to.to_string());
        self
    }

    /// Applies the renaming transformations to the given [`Value::Object`].
    ///
    /// # Errors
    ///
    /// Returns an error if the input value is not an object.
    pub fn apply(&self, value: &Value) -> Result<Value, Error> {
        match value.as_object() {
            Some(obj) => {
                let mut out = BTreeMap::new();
                for (k, v) in obj {
                    let new_key = self.mappings.get(k).cloned().unwrap_or_else(|| k.clone());
                    out.insert(new_key, v.clone());
                }
                Ok(Value::Object(out))
            }
            None => Err(crate::error::SerializationError::new(format!(
                "FieldMapper expects an object, got {}",
                value.type_name()
            ))
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Celsius(f64);
    struct Fahrenheit(f64);

    impl Adapt<Celsius> for Fahrenheit {
        fn adapt(c: Celsius) -> Result<Self, Error> {
            Ok(Fahrenheit(c.0 * 9.0 / 5.0 + 32.0))
        }
    }

    #[test]
    fn test_adapt_basic() {
        let f = Fahrenheit::adapt(Celsius(0.0)).unwrap();
        assert_eq!(f.0, 32.0);
        let f2 = Fahrenheit::adapt(Celsius(100.0)).unwrap();
        assert_eq!(f2.0, 212.0);
    }

    #[test]
    fn test_pipeline_chain() {
        let p = Pipeline::new()
            .step(|v| match v {
                Value::Int(n) => Ok(Value::Int(n * 2)),
                other => Ok(other),
            })
            .step(|v| match v {
                Value::Int(n) => Ok(Value::Int(n + 1)),
                other => Ok(other),
            });
        assert_eq!(p.run(Value::Int(5)).unwrap(), Value::Int(11));
    }

    #[test]
    fn test_pipeline_short_circuits_on_error() {
        let p = Pipeline::new()
            .step(|_| Err(crate::error::SerializationError::new("step1 failed").into()))
            .step(Ok);
        assert!(p.run(Value::Null).is_err());
    }

    #[test]
    fn test_pipeline_empty() {
        let p = Pipeline::new();
        assert_eq!(p.run(Value::Int(7)).unwrap(), Value::Int(7));
    }

    fn make_obj(fields: &[(&str, Value)]) -> Value {
        let mut m = BTreeMap::new();
        for (k, v) in fields {
            m.insert(k.to_string(), v.clone());
        }
        Value::Object(m)
    }

    #[test]
    fn test_field_mapper_rename() {
        let mapper = FieldMapper::new()
            .map("first_name", "firstName")
            .map("last_name", "lastName");
        let input = make_obj(&[
            ("first_name", Value::String("Alice".into())),
            ("last_name", Value::String("Smith".into())),
            ("age", Value::Int(30)),
        ]);
        let out = mapper.apply(&input).unwrap();
        assert!(out.get("firstName").is_some());
        assert!(out.get("lastName").is_some());
        assert!(out.get("age").is_some());
        assert!(out.get("first_name").is_none());
    }

    #[test]
    fn test_field_mapper_non_object_fails() {
        let mapper = FieldMapper::new();
        assert!(mapper.apply(&Value::Int(1)).is_err());
    }

    #[test]
    fn test_field_mapper_unmapped_keys_pass_through() {
        let mapper = FieldMapper::new().map("a", "A");
        let input = make_obj(&[("a", Value::Int(1)), ("b", Value::Int(2))]);
        let out = mapper.apply(&input).unwrap();
        assert!(out.get("A").is_some());
        assert!(out.get("b").is_some());
    }
}
