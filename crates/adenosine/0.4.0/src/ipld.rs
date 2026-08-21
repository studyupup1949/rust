use libipld::{Cid, Ipld};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::str::FromStr;

/// Intentionally serializing with this instead of DAG-JSON, because ATP schemas don't encode CID
/// links in any special way, they just pass the CID as a string.
pub fn ipld_into_json_value(val: Ipld) -> Value {
    match val {
        Ipld::Null => Value::Null,
        Ipld::Bool(b) => Value::Bool(b),
        Ipld::Integer(v) => json!(v),
        Ipld::Float(v) => json!(v),
        Ipld::String(s) => Value::String(s),
        Ipld::Bytes(b) => json!({
            "$bytes": Value::String(data_encoding::BASE64_NOPAD.encode(&b)),
        }),
        Ipld::List(l) => Value::Array(l.into_iter().map(ipld_into_json_value).collect()),
        Ipld::Map(m) => Value::Object(serde_json::Map::from_iter(
            m.into_iter().map(|(k, v)| (k, ipld_into_json_value(v))),
        )),
        Ipld::Link(c) => json!({
            "$link": Value::String(c.to_string()),
        }),
    }
}

/// Crude reverse generation
///
/// Does not handle base64 to bytes, and the link generation is pretty simple (object elements with
/// key "cid"). Numbers always come through as f64 (float).
pub fn json_value_into_ipld(val: Value) -> Ipld {
    match val {
        Value::Null => Ipld::Null,
        Value::Bool(b) => Ipld::Bool(b),
        Value::String(s) => Ipld::String(s),
        // TODO: handle numbers better?
        Value::Number(v) => Ipld::Float(v.as_f64().unwrap()),
        Value::Array(l) => Ipld::List(l.into_iter().map(json_value_into_ipld).collect()),
        Value::Object(m) => {
            if m.len() == 1 && m.contains_key("$link") {
                let cid_str = m["$link"].as_str().expect("$link as CID string");
                Ipld::Link(Cid::from_str(cid_str).unwrap())
            } else if m.len() == 1 && m.contains_key("$bytes") {
                let bytes_b64 = m["$bytes"].as_str().expect("$bytes as base64 string");
                Ipld::Bytes(
                    data_encoding::BASE64_NOPAD
                        .decode(bytes_b64.as_bytes())
                        .expect("valid base64 in $bytes"),
                )
            } else {
                let map: BTreeMap<String, Ipld> =
                    BTreeMap::from_iter(m.into_iter().map(|(k, v)| (k, json_value_into_ipld(v))));
                Ipld::Map(map)
            }
        }
    }
}
