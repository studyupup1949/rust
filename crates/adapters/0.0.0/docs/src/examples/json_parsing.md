# JSON Parsing

This example demonstrates how to use the built-in, native, zero-dependency recursive-descent JSON tokenization parser and serializer engine.

### Compilable Example

```rust
//! json_parsing example — native JSON parse and stringify.

use adapters::Value;
use adapters::json::{parse, stringify, stringify_pretty};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== json_parsing example ===\n");

    // All value types
    let samples = [
        "null",
        "true",
        "false",
        "42",
        "-7",
        "3.14",
        r#""hello world""#,
        "[]",
        "{}",
        r#"[1, "two", null, false, 3.14]"#,
        r#"{"name":"alice","age":30,"active":true}"#,
    ];

    for s in &samples {
        let v = parse(s)?;
        let out = stringify(&v)?;
        println!("Input:  {s}");
        println!("Parsed: {:?}", v);
        println!("Output: {out}\n");
    }

    // Unicode strings
    let unicode = r#""\u0048\u0065\u006C\u006C\u006F \u4e16\u754c""#;
    let v = parse(unicode)?;
    println!("Unicode: {} → {:?}\n", unicode, v);

    // Empty containers
    let empty_obj = parse("{}")?;
    let empty_arr = parse("[]")?;
    println!("Empty object: {:?}", empty_obj);
    println!("Empty array:  {:?}\n", empty_arr);

    // Pretty print
    let mut m = BTreeMap::new();
    m.insert("name".to_string(), Value::String("Bob".into()));
    m.insert(
        "scores".to_string(),
        Value::Array(vec![Value::Int(10), Value::Int(20)]),
    );
    let obj = Value::Object(m);
    println!("Pretty printed:\n{}", stringify_pretty(&obj)?);

    // Error case
    match parse("{invalid}") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Parse error (expected): {}", e),
    }

    Ok(())
}
```
