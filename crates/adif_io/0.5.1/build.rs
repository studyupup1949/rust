use linked_hash_map::LinkedHashMap;
use serde_json::Value;
use std::{env, fs};

/// Filter for relevant types
const ALLOWED_TYPES: [&str; 9] = [
    "String",
    "Date",
    "Number",
    "PositiveInteger",
    "Integer",
    "MultilineString",
    "Boolean",
    "Time",
    "GridSquare",
];

/// Get just the type mapping for ADI relevant fields form the ADIF definition file for the type collation
fn main() {
    let json = fs::read_to_string("data/fields.json").expect("could not read input file");
    let json: Value = serde_json::from_str(&json).expect("could not interpret JSON");

    let mut map: LinkedHashMap<String, String> = LinkedHashMap::new();

    let fields = json["Adif"].clone()["Fields"].clone()["Records"].clone();
    let fields = if let Some(f) = fields.as_object() {
        f
    } else {
        panic!("could not get 'Records' object")
    };

    fields
        .iter()
        .map(|(k, v)| {
            (
                k,
                if let Some(t) = v["Data Type"].as_str() {
                    t.to_string()
                } else {
                    panic!("could not get 'Data Type' for '{}'", k)
                },
            )
        })
        .filter(|(_, v)| ALLOWED_TYPES.contains(&v.as_str()))
        .filter(|(k, _)| !k.ends_with("_INTL"))
        .for_each(|(k, v)| {
            let _ = map.insert(k.to_string(), v.to_string());
        });

    let mut out_path = env::var_os("OUT_DIR").unwrap();
    out_path.push("/type_map.json");
    fs::write(
        out_path,
        serde_json::to_string(&map).expect("could not generate JSON"),
    )
    .expect("could not write output file");
}
