use acts_tag::{Tags, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct TestStruct {
    pub value: f32,
}

#[derive(Tags)]
pub struct MyTags {
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
        tag1: "a".to_string(),
        tag2: 5,
        others: "others".to_string(),
    };

    let tag1 = t.tag_value("tag1");
    assert_eq!(tag1.unwrap().to::<String>().unwrap(), t.tag1);

    let tag2 = t.tag_value("tag2");
    assert_eq!(tag2.unwrap().to::<i32>().unwrap(), t.tag2);

    let others = t.tag_value("others");
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

    let tag3 = t.tag_value("tag3");
    assert_eq!(tag3.unwrap().to::<Vec<i32>>().unwrap(), t.tag3);

    let tag4 = t
        .tag_value("tag4")
        .unwrap()
        .to::<HashMap<String, TestStruct>>()
        .unwrap();

    for (k, v) in &tag4 {
        let value = map.get(k).unwrap();
        assert_eq!(v.value, value.value);
    }
}

#[test]
fn test_tag_tags() {
    let t = MyTags {
        tag1: "a".to_string(),
        tag2: 5,
        others: "others".to_string(),
    };

    let tags = t.tag_keys();
    assert_eq!(tags, vec!["tag1", "tag2"]);
}
