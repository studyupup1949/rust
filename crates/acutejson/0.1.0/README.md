# acutejson

A streaming JSON extractor for Rust. Register [RFC 6901 JSON Pointer](https://www.rfc-editor.org/rfc/rfc6901) paths ahead of time; feed byte chunks as they arrive; receive matching values via callback — without buffering the whole document.

## Usage

```rust
use acutejson::{Builder, Status};

let mut parser = Builder::new()
    .register("/user/name", |bytes, is_complete| {
        if is_complete {
            println!("name chunk: {}", std::str::from_utf8(bytes).unwrap());
        }
    })?
    .register("/user/age", |bytes, is_complete| {
        if is_complete {
            println!("age: {}", std::str::from_utf8(bytes).unwrap());
        }
    })?
    .build();

// Feed chunks as they arrive.
match parser.feed(chunk)? {
    Status::Done    => { /* all registered paths resolved */ }
    Status::NeedMore => { /* keep feeding */ }
}

// Signal end of stream after the last chunk.
parser.finish()?;
```

## How it works

- Paths are compiled into a trie at build time.
- `feed` runs a zero-allocation state machine over each incoming byte slice.
- String values are streamed directly from the input buffer to the callback (`is_complete = false` per chunk, `is_complete = true` on the closing `"`).
- Numbers and keywords (`true` / `false` / `null`) are buffered internally and delivered in a single call (`is_complete = true`).
- Unregistered subtrees are skipped with depth tracking — no allocations, no copies.
- Returns `Status::Done` as soon as every registered path has been matched; remaining bytes are not processed.

## Pointer syntax

Follows RFC 6901: paths start with `/`, segments separated by `/`. Use `~0` for a literal `~` and `~1` for a literal `/` in a key name.

```
/foo/bar        →  {"foo": {"bar": <value>}}
/items/2/name   →  {"items": [{}, {}, {"name": <value>}, ...]}
/a~1b           →  {"a/b": <value>}
""              →  <top-level value>
```

## Limitations

- Matches the **first occurrence** of each path; subsequent duplicates are ignored.
- Registered paths that point to a **container** (object or array) receive no callback — only scalar values (strings, numbers, booleans, null) are delivered.
- Input is not fully validated (e.g. leading zeros in numbers are accepted).
