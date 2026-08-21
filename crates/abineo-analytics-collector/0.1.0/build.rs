use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    // create perfect hash table for timezone lookup
    let raw_data = include_str!("timezones.json");
    let tz_map: HashMap<String, String> = serde_json::from_str(raw_data).unwrap();
    let mut map = phf_codegen::Map::new();
    for (key, value) in tz_map.into_iter() {
        map.entry(key, format!("\"{}\"", value).as_str());
    }
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("timezone-codegen.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());
    writeln!(
        &mut file,
        "pub static TIMEZONES: phf::Map<&'static str, &'static str> = {};",
        map.build()
    )
    .unwrap();
}
