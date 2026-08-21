// adminx-mongo/src/convert.rs
//
// BSON <-> JSON conversion tuned for the admin UI. The key concern is that ids
// and dates render as plain strings (so template links and views work), rather
// than the `{"$oid": ...}` / `{"$date": ...}` shapes of canonical extended JSON.

use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use serde_json::{Map, Value};

/// Convert a BSON value into UI-friendly JSON: `ObjectId` -> hex string,
/// `DateTime` -> RFC 3339 string, nested docs/arrays recursively.
pub fn bson_to_json(b: Bson) -> Value {
    match b {
        Bson::ObjectId(oid) => Value::String(oid.to_hex()),
        Bson::DateTime(dt) => Value::String(
            dt.try_to_rfc3339_string()
                .unwrap_or_else(|_| dt.to_string()),
        ),
        Bson::String(s) => Value::String(s),
        Bson::Boolean(b) => Value::Bool(b),
        Bson::Int32(i) => Value::from(i),
        Bson::Int64(i) => Value::from(i),
        Bson::Double(f) => Value::from(f),
        Bson::Null => Value::Null,
        Bson::Array(a) => Value::Array(a.into_iter().map(bson_to_json).collect()),
        Bson::Document(d) => doc_to_json(d),
        // Decimal128, Timestamp, Binary, etc. — fall back to relaxed extended JSON.
        other => other.into_relaxed_extjson(),
    }
}

pub fn doc_to_json(d: Document) -> Value {
    let mut map = Map::new();
    for (k, v) in d {
        map.insert(k, bson_to_json(v));
    }
    // Expose `_id` also as `id`, so a resource using the default
    // `primary_key() = "id"` renders correct links and ids on Mongo without any
    // Mongo-specific code — upholding adminx's "same resource code over SQL or
    // Mongo" promise. `id_filter` already maps the query side (`id` -> `_id`);
    // this mirrors it on the read side. A resource that explicitly uses `_id`
    // still works — the extra `id` key is harmless.
    if !map.contains_key("id") {
        if let Some(oid) = map.get("_id").cloned() {
            map.insert("id".to_string(), oid);
        }
    }
    Value::Object(map)
}

/// Convert a writable JSON object (from a form or JSON body) into a BSON
/// document for insert/update.
pub fn json_map_to_doc(map: Map<String, Value>) -> Document {
    match mongodb::bson::to_bson(&Value::Object(map)) {
        Ok(Bson::Document(d)) => d,
        _ => Document::new(),
    }
}

/// Build a primary-key filter. The resource's `primary_key()` ("id" by default)
/// is mapped onto Mongo's `_id`; the id string is parsed as an `ObjectId` when
/// possible, otherwise matched as a literal (string/numeric `_id`).
pub fn id_filter(pk: &str, id: &str) -> Document {
    let field = if pk == "id" { "_id" } else { pk };
    if field == "_id" {
        match ObjectId::parse_str(id) {
            Ok(oid) => doc! { "_id": oid },
            Err(_) => doc! { "_id": id },
        }
    } else {
        doc! { field: id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objectid_renders_as_hex() {
        let oid = ObjectId::parse_str("507f1f77bcf86cd799439011").unwrap();
        let json = bson_to_json(Bson::ObjectId(oid));
        assert_eq!(json, Value::String("507f1f77bcf86cd799439011".into()));
    }

    #[test]
    fn nested_document_and_id_flatten() {
        let d = doc! { "_id": ObjectId::parse_str("507f1f77bcf86cd799439011").unwrap(),
                       "name": "Ada", "meta": { "age": 36i32 } };
        let json = doc_to_json(d);
        assert_eq!(json["_id"], Value::String("507f1f77bcf86cd799439011".into()));
        assert_eq!(json["name"], Value::String("Ada".into()));
        assert_eq!(json["meta"]["age"], Value::from(36));
    }

    #[test]
    fn id_is_aliased_from_underscore_id_for_default_pk_resources() {
        // The bug this guards: a resource using the default `primary_key() = "id"`
        // must find its id under `"id"` on Mongo, or every view/edit/attach link
        // renders empty. `doc_to_json` mirrors `_id` to `id`.
        let d = doc! { "_id": ObjectId::parse_str("507f1f77bcf86cd799439011").unwrap(),
                       "title": "Post" };
        let json = doc_to_json(d);
        assert_eq!(json["id"], Value::String("507f1f77bcf86cd799439011".into()));
        // And the id round-trips: the aliased value parses back to the same _id.
        let filter = id_filter("id", json["id"].as_str().unwrap());
        assert!(matches!(filter.get("_id"), Some(Bson::ObjectId(_))));
    }

    #[test]
    fn an_explicit_id_field_is_not_clobbered_by_the_alias() {
        // If a document already carries its own `id` (e.g. a legacy column), the
        // alias must not overwrite it with `_id`.
        let d = doc! { "_id": ObjectId::parse_str("507f1f77bcf86cd799439011").unwrap(),
                       "id": "legacy-42" };
        let json = doc_to_json(d);
        assert_eq!(json["id"], Value::String("legacy-42".into()));
    }

    #[test]
    fn id_filter_parses_objectid_else_literal() {
        let f = id_filter("id", "507f1f77bcf86cd799439011");
        assert!(matches!(f.get("_id"), Some(Bson::ObjectId(_))));

        let f2 = id_filter("id", "not-an-oid");
        assert_eq!(f2.get_str("_id").unwrap(), "not-an-oid");

        let f3 = id_filter("slug", "abc");
        assert_eq!(f3.get_str("slug").unwrap(), "abc");
    }
}
