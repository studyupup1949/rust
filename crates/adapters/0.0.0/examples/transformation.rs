//! Transformation example showcasing the Adapt trait, Pipeline, and FieldMapper.

use adapters::{Adapt, Error, FieldMapper, Pipeline, Value};
use std::collections::BTreeMap;

struct Celsius(f64);
struct Fahrenheit(f64);

impl Adapt<Celsius> for Fahrenheit {
    fn adapt(c: Celsius) -> Result<Self, Error> {
        Ok(Fahrenheit(c.0 * 9.0 / 5.0 + 32.0))
    }
}

fn make_obj(fields: &[(&str, Value)]) -> Value {
    let mut m = BTreeMap::new();
    for (k, v) in fields {
        m.insert(k.to_string(), v.clone());
    }
    Value::Object(m)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== transformation example ===\n");

    let f = Fahrenheit::adapt(Celsius(100.0))?;
    println!("100°C = {}°F", f.0);

    let pipeline = Pipeline::new()
        .step(|v| match v {
            Value::Int(n) => Ok(Value::Int(n * 2)),
            other => Ok(other),
        })
        .step(|v| match v {
            Value::Int(n) => Ok(Value::Int(n + 10)),
            other => Ok(other),
        });

    let result = pipeline.run(Value::Int(5))?;
    println!("Pipeline 5 → {}", result);

    let mapper = FieldMapper::new()
        .map("first_name", "firstName")
        .map("last_name", "lastName")
        .map("phone_number", "phoneNumber");

    let db_row = make_obj(&[
        ("first_name", Value::String("Alice".into())),
        ("last_name", Value::String("Smith".into())),
        ("phone_number", Value::String("+1-555-0100".into())),
        ("id", Value::Int(42)),
    ]);

    let api_response = mapper.apply(&db_row)?;
    println!("\nDB row:       {}", db_row);
    println!("API response: {}", api_response);

    Ok(())
}
