use activities::object::{TypedObjectType, Video};
use std::convert::TryInto;

fn main() {
    let v = Video::new(
        "https://dbhattarai.info.np".try_into().unwrap(),
        "hello".to_string(),
    );
    let _json = serde_json::to_string_pretty(&v).unwrap();
    let json = to_json(&v).unwrap();
    println!("{}", json);
}

fn to_json(
    v: &dyn TypedObjectType,
) -> std::result::Result<std::string::String, serde_json::error::Error> {
    serde_json::to_string_pretty(&v)
}
