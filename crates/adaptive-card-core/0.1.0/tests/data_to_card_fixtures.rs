//! `data_to_card` fixtures: detect shape and render appropriate card body.

use adaptive_card_core::{DataToCardOpts, Host, Presentation, data_to_card};
use serde_json::json;

#[test]
fn factset_from_flat_object() {
    let data = json!({ "name": "Alice", "dept": "Eng", "salary": 100_000 });
    let card = data_to_card(
        &data,
        &DataToCardOpts {
            title: Some("Employee".to_string()),
            presentation: None,
            host: Host::Teams,
        },
    )
    .unwrap();
    assert_eq!(card["body"][1]["type"], "FactSet");
}

#[test]
fn table_from_array_of_objects() {
    let data = json!([
        { "col1": "A", "col2": "B" },
        { "col1": "C", "col2": "D" }
    ]);
    let card = data_to_card(
        &data,
        &DataToCardOpts {
            title: None,
            presentation: None,
            host: Host::Teams,
        },
    )
    .unwrap();
    assert_eq!(card["body"][0]["type"], "Table");
}

#[test]
fn list_from_primitive_array() {
    let data = json!(["apple", "banana", "cherry"]);
    let card = data_to_card(
        &data,
        &DataToCardOpts {
            title: None,
            presentation: None,
            host: Host::Teams,
        },
    )
    .unwrap();
    assert_eq!(card["body"][0]["type"], "Container");
}

#[test]
fn explicit_presentation_overrides_detection() {
    let data = json!({ "k": "v" });
    let card = data_to_card(
        &data,
        &DataToCardOpts {
            title: None,
            presentation: Some(Presentation::List),
            host: Host::Teams,
        },
    )
    .unwrap();
    assert_eq!(card["body"][0]["type"], "Container");
}
