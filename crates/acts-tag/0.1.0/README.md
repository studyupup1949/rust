# acts-tag
A simple trait to mark up the struct and help recognize the struct's attributes.

# Example

```rust
use acts_tag::{Tags, Value};
use std::any::Any;
#[derive(Tags)]
pub struct MyTags {
    #[tag]
    pub tag1: String,
    #[tag]
    pub tag2: i32,
    pub others: String,
}
fn main() {
    let t = MyTags {
        tag1: "a".to_string(),
        tag2: 5,
        others: "others".to_string(),
    };
    assert_eq!(t.tag_keys(), vec![ "tag1", "tag2" ]);
    for tag_key in t.tag_keys() {
        let value = t.tag_value(tag_key).unwrap();
        println!("value {} = {:?}", tag_key, value);
    }
    let value = t.tag_value("tag1").unwrap();
    assert_eq!(value.to::<String>().unwrap(), t.tag1);
    let value = t.tag_value("tag2").unwrap();
    assert_eq!(value.to::<i32>().unwrap(), t.tag2);
    let value = t.tag_value("others");
    assert_eq!(value, None);
    let is_tag = t.is_tag("others");
    assert_eq!(is_tag, false);
    let is_tag = t.is_tag("tag2");
    assert_eq!(is_tag, true);
}
```