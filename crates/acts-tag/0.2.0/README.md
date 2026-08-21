# acts-tag
A simple trait to mark up the struct and help recognize the struct's attributes.
 A simple trait to mark up the struct and help recognize the struct's attributes.

 # Example

```rust
use acts_tag::{Tags, Value};
use std::any::Any;
#[derive(Tags)]
pub struct MyTags {
    // tag(id) change the key to id
    #[tag(id)]
    pub my_id: String,
    #[tag]
    pub tag1: String,
    #[tag]
    pub tag2: i32,
    pub others: String,
}
fn main() {
    let t = MyTags {
        my_id: "my id".to_string(),
        tag1: "a".to_string(),
        tag2: 5,
        others: "others".to_string(),
    };
    assert_eq!(MyTags::keys(), vec![ "id", "tag1", "tag2" ]);
    
    for tag_key in &MyTags::keys() {
        let value = t.value(tag_key).unwrap();
        println!("value {} = {:?}", tag_key, value);
    }
    let value = t.value("id").unwrap();
    assert_eq!(value.real::<String>().unwrap(), t.my_id);

    let value = t.value("tag1").unwrap();
    assert_eq!(value.real::<String>().unwrap(), t.tag1);

    let value = t.value("tag2").unwrap();
    assert_eq!(value.real::<i32>().unwrap(), t.tag2);

    let value = t.value("others");
    assert_eq!(value, None);

    let is_tag = t.is_tag("others");
    assert_eq!(is_tag, false);

    let is_tag = t.is_tag("tag2");
    assert_eq!(is_tag, true);
}
```