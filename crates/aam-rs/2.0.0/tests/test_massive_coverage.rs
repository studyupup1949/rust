use aam_rs::aam::AAM;

#[test]
fn matrix_parse_and_lookup_cases() {
    for i in 0..200 {
        let key = format!("key_{i}");
        let value = format!("value_{i}");
        let content = format!("{key} = {value}");
        let doc = AAM::parse(&content).expect("matrix parse must succeed");
        assert_eq!(doc.get(&key), Some(value.as_str()));
    }
}

#[test]
fn matrix_reverse_and_deep_search_cases() {
    let content = "service.a = x\nservice.b = x\nother = y";
    let doc = AAM::parse(content).expect("parse must succeed");

    let reverse = doc.reverse_search("x");
    assert!(reverse.contains(&"service.a"));
    assert!(reverse.contains(&"service.b"));

    let deep = doc.deep_search("service");
    assert_eq!(deep.len(), 2);
}

#[test]
fn schema_and_type_metadata_presence() {
    let content =
        "@schema Device { id: i32, name: string }\n@type port = i32\nid = 1\nname = sensor";
    let doc = AAM::parse(content).expect("schema parse must succeed");

    assert!(doc.get_schema("Device").is_some());
    assert!(doc.get_type("port").is_some());
}
