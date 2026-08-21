# Ad Hoc Figment

An ad hoc provider for the [figment] configuration library.

```rust
use figment::Figment;
use adhoc_figment::AdHocProvider;

fn main() {
    let ad_hoc = AdHocProvider::new("key", "value");
    let figment = Figment::from(ad_hoc);
    let value: String = figment.extract_inner("key").unwrap();
    assert_eq!(value, "value");
}
```

[figment]: https://docs.rs/figment/
