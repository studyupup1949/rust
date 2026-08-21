# ActionIDMap

A HashMap that can be updated from a URL. Intended to be used to keep reverse-engineered API's in use even as private referenced ID's get updated, so end users don't need to update.

Generate example code below.

```rust

let mut actionmap = ActionIDMap::new("https://gist.githubusercontent.com/[..]".into(), 86400).unwrap();
let bar = actionmap.get("foo")?;

if actionmap.needs_refresh() {
    actionmap.refresh();
}
```

The URL can be, for example, a GitHub Gist or Raw file that contains a serialized HashMap (`{"key":"value"}`).