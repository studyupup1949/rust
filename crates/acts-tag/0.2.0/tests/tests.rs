use acts_tag::{Tags, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct TestStruct {
    pub value: f32,
}

#[derive(Tags)]
pub struct MyTags {
    #[tag(id)]
    pub tag_id: String,

    #[tag]
    pub tag1: String,

    #[tag]
    pub tag2: i32,

    pub others: String,
}

#[derive(Tags)]
pub struct MyTags2 {
    #[tag]
    pub tag3: Vec<i32>,

    #[tag]
    pub tag4: HashMap<String, TestStruct>,
}

#[test]
fn test_tag_attr_simple_value() {
    let t = MyTags {
        tag_id: "tag_id".to_string(),
        tag1: "a".to_string(),
        tag2: 5,
        others: "others".to_string(),
    };

    let tag1 = t.value("tag1");
    assert_eq!(tag1.unwrap().real::<String>().unwrap(), t.tag1);

    let tag2 = t.value("tag2");
    assert_eq!(tag2.unwrap().real::<i32>().unwrap(), t.tag2);

    let others = t.value("others");
    assert_eq!(others, None);
}

#[test]
fn test_tag_attr_complex_value() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), TestStruct { value: 5_f32 });
    map.insert("b".to_string(), TestStruct { value: 100.20_f32 });

    let t = MyTags2 {
        tag3: vec![5, 100],
        tag4: map.clone(),
    };

    let tag3 = t.value("tag3");
    assert_eq!(tag3.unwrap().real::<Vec<i32>>().unwrap(), t.tag3);

    let tag4 = t
        .value("tag4")
        .unwrap()
        .real::<HashMap<String, TestStruct>>()
        .unwrap();

    for (k, v) in &tag4 {
        let value = map.get(k).unwrap();
        assert_eq!(v.value, value.value);
    }
}

#[test]
fn test_tag_tags() {
    let tags = MyTags::keys();
    assert_eq!(tags, vec!["id", "tag1", "tag2"]);
}

#[test]
fn test_is_tag() {
    let t = MyTags {
        tag_id: "tag_id".to_string(),
        tag1: "a".to_string(),
        tag2: 5,
        others: "others".to_string(),
    };

    let is_tag1 = t.is_tag("tag1");
    assert_eq!(is_tag1, true);

    let is_others = t.is_tag("others");
    assert_eq!(is_others, false);
}

#[test]
fn test_tag_id() {
    let t = MyTags {
        tag_id: "tag_id".to_string(),
        tag1: "a".to_string(),
        tag2: 5,
        others: "others".to_string(),
    };

    let tag_id = t.value("id");
    assert_eq!(tag_id.unwrap().real::<String>().unwrap(), t.tag_id);
}
