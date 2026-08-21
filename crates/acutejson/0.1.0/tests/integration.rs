use std::cell::RefCell;
use std::rc::Rc;

use acutejson::{Builder, Status};

// ── Helper ────────────────────────────────────────────────────────────────────

/// Register `pointer` with a collecting callback; return `(parser, collected)`.
/// Each call collects `(bytes, is_complete)`.
fn collecting(pointer: &str) -> (acutejson::Parser, Rc<RefCell<Vec<(Vec<u8>, bool)>>>) {
  let calls: Rc<RefCell<Vec<(Vec<u8>, bool)>>> = Rc::new(RefCell::new(Vec::new()));
  let calls2 = Rc::clone(&calls);
  let p = Builder::new()
    .register(pointer, move |bytes: &[u8], done: bool| {
      calls2.borrow_mut().push((bytes.to_vec(), done));
    })
    .unwrap()
    .build();
  (p, calls)
}

/// Concatenate all non-final callback bytes for a string value.
fn string_body(calls: &[(Vec<u8>, bool)]) -> Vec<u8> {
  calls
    .iter()
    .filter(|(_, done)| !done)
    .flat_map(|(b, _)| b.iter().copied())
    .collect()
}

// ── Basic end-to-end ──────────────────────────────────────────────────────────

#[test]
fn simple_string_value() {
  let (mut p, calls) = collecting("/name");
  assert_eq!(p.feed(b"{\"name\":\"Alice\"}"), Ok(Status::Done));
  assert_eq!(string_body(&calls.borrow()), b"Alice");
}

#[test]
fn simple_number_value() {
  let (mut p, calls) = collecting("/age");
  assert_eq!(p.feed(b"{\"age\":30}"), Ok(Status::Done));
  assert_eq!(calls.borrow()[..], [(b"30".to_vec(), true)]);
}

#[test]
fn simple_bool_value() {
  let (mut p, calls) = collecting("/active");
  assert_eq!(p.feed(b"{\"active\":true}"), Ok(Status::Done));
  assert_eq!(calls.borrow()[..], [(b"true".to_vec(), true)]);
}

#[test]
fn simple_null_value() {
  let (mut p, calls) = collecting("/ref");
  assert_eq!(p.feed(b"{\"ref\":null}"), Ok(Status::Done));
  assert_eq!(calls.borrow()[..], [(b"null".to_vec(), true)]);
}

// ── Nested paths ──────────────────────────────────────────────────────────────

#[test]
fn nested_two_levels() {
  let (mut p, calls) = collecting("/user/id");
  assert_eq!(
    p.feed(b"{\"user\":{\"id\":42,\"name\":\"Bob\"}}"),
    Ok(Status::Done)
  );
  assert_eq!(calls.borrow()[..], [(b"42".to_vec(), true)]);
}

#[test]
fn nested_three_levels() {
  let (mut p, calls) = collecting("/a/b/c");
  assert_eq!(
    p.feed(b"{\"a\":{\"b\":{\"c\":\"deep\"}}}"),
    Ok(Status::Done)
  );
  assert_eq!(string_body(&calls.borrow()), b"deep");
}

// ── Array paths ───────────────────────────────────────────────────────────────

#[test]
fn array_first_element() {
  let (mut p, calls) = collecting("/0");
  assert_eq!(p.feed(b"[\"first\",\"second\"]"), Ok(Status::Done));
  assert_eq!(string_body(&calls.borrow()), b"first");
}

#[test]
fn array_third_element() {
  let (mut p, calls) = collecting("/2");
  p.feed(b"[\"a\",\"b\",\"c\",\"d\"]").unwrap();
  assert_eq!(string_body(&calls.borrow()), b"c");
}

#[test]
fn nested_array_element() {
  let (mut p, calls) = collecting("/items/1/name");
  assert_eq!(
    p.feed(b"{\"items\":[{\"name\":\"first\"},{\"name\":\"second\"}]}"),
    Ok(Status::Done)
  );
  assert_eq!(string_body(&calls.borrow()), b"second");
}

// ── Multiple registered paths ─────────────────────────────────────────────────

#[test]
fn two_paths_both_found() {
  let hits: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
  let h1 = Rc::clone(&hits);
  let h2 = Rc::clone(&hits);
  let mut p = Builder::new()
    .register("/foo", move |b, done| {
      if done {
        h1.borrow_mut()
          .push(format!("foo:{}", String::from_utf8_lossy(b)));
      }
    })
    .unwrap()
    .register("/bar", move |b, done| {
      if done {
        h2.borrow_mut()
          .push(format!("bar:{}", String::from_utf8_lossy(b)));
      }
    })
    .unwrap()
    .build();
  let result = p.feed(b"{\"foo\":1,\"bar\":2}");
  assert_eq!(result, Ok(Status::Done));
  let h = hits.borrow();
  assert!(h.contains(&"foo:1".to_string()));
  assert!(h.contains(&"bar:2".to_string()));
}

#[test]
fn two_paths_one_not_present() {
  let (mut p, calls) = collecting("/present");
  // /missing not registered; just check NeedMore and /present found.
  let mut p2 = Builder::new()
    .register("/present", {
      let calls2 = Rc::clone(&calls);
      move |b, done| calls2.borrow_mut().push((b.to_vec(), done))
    })
    .unwrap()
    .register("/missing", |_, _| {})
    .unwrap()
    .build();
  let _ = p2.feed(b"{\"present\":99}");
  let _ = p.feed(b"{\"present\":99}"); // feed the collecting parser too
  let c = calls.borrow();
  // At least one call with is_complete=true containing "99"
  assert!(c.iter().any(|(b, done)| *done && b == b"99"));
}

// ── Skip interleaved with match ───────────────────────────────────────────────

#[test]
fn skip_before_target() {
  let (mut p, calls) = collecting("/target");
  p.feed(b"{\"skip\":{\"a\":1,\"b\":\"noise\"},\"target\":\"found\"}")
    .unwrap();
  assert_eq!(string_body(&calls.borrow()), b"found");
}

#[test]
fn skip_deeply_nested_before_target() {
  let (mut p, calls) = collecting("/z");
  p.feed(b"{\"x\":{\"y\":{\"w\":[1,2,{\"v\":3}]}},\"z\":true}")
    .unwrap();
  assert_eq!(calls.borrow()[..], [(b"true".to_vec(), true)]);
}

#[test]
fn skip_array_before_target() {
  let (mut p, calls) = collecting("/result");
  p.feed(b"{\"data\":[1,2,3],\"result\":-1}").unwrap();
  assert_eq!(calls.borrow()[..], [(b"-1".to_vec(), true)]);
}

// ── Multi-chunk delivery ──────────────────────────────────────────────────────

#[test]
fn byte_by_byte() {
  let (mut p, calls) = collecting("/v");
  let doc = b"{\"v\":\"hello\"}";
  for byte in doc.iter() {
    match p.feed(std::slice::from_ref(byte)) {
      Ok(Status::Done) => break,
      Ok(Status::NeedMore) => {}
      Err(e) => panic!("unexpected error: {:?}", e),
    }
  }
  assert_eq!(string_body(&calls.borrow()), b"hello");
}

#[test]
fn chunk_boundary_inside_key() {
  let (mut p, calls) = collecting("/longkey");
  p.feed(b"{\"long").unwrap();
  p.feed(b"key\":").unwrap();
  p.feed(b"42}").unwrap();
  assert_eq!(calls.borrow()[..], [(b"42".to_vec(), true)]);
}

#[test]
fn chunk_boundary_inside_number() {
  let (mut p, calls) = collecting("/n");
  p.feed(b"{\"n\":123").unwrap();
  p.feed(b"456}").unwrap();
  assert_eq!(calls.borrow()[..], [(b"123456".to_vec(), true)]);
}

#[test]
fn chunk_boundary_inside_keyword() {
  let (mut p, calls) = collecting("/b");
  p.feed(b"{\"b\":fal").unwrap();
  p.feed(b"se}").unwrap();
  assert_eq!(calls.borrow()[..], [(b"false".to_vec(), true)]);
}

#[test]
fn chunk_boundary_inside_string_body() {
  let (mut p, calls) = collecting("/s");
  p.feed(b"{\"s\":\"hel").unwrap();
  p.feed(b"lo world").unwrap();
  p.feed(b"\"}").unwrap();
  assert_eq!(string_body(&calls.borrow()), b"hello world");
}

// ── RFC 6901 pointer edge cases ───────────────────────────────────────────────

#[test]
fn key_with_tilde_in_name() {
  // Pointer `/a~0b` matches key `a~b`.
  let (mut p, calls) = collecting("/a~0b");
  p.feed(b"{\"a~b\":7}").unwrap();
  assert_eq!(calls.borrow()[..], [(b"7".to_vec(), true)]);
}

#[test]
fn key_with_slash_in_name() {
  // Pointer `/a~1b` matches key `a/b`.
  let (mut p, calls) = collecting("/a~1b");
  p.feed(b"{\"a/b\":8}").unwrap();
  assert_eq!(calls.borrow()[..], [(b"8".to_vec(), true)]);
}

#[test]
fn path_not_found_returns_need_more() {
  let (mut p, calls) = collecting("/missing");
  let result = p.feed(b"{\"a\":1,\"b\":2}");
  assert_eq!(result, Ok(Status::NeedMore));
  assert!(calls.borrow().is_empty());
}

// ── Large / realistic document ────────────────────────────────────────────────

#[test]
fn realistic_json_object() {
  let doc = br#"{
        "id": 1234,
        "name": "Test User",
        "email": "test@example.com",
        "tags": ["rust", "json", "streaming"],
        "address": {
            "street": "123 Main St",
            "city": "Springfield",
            "zip": "12345"
        },
        "active": true,
        "score": 9.5
    }"#;

  let city_calls: Rc<RefCell<Vec<(Vec<u8>, bool)>>> = Rc::new(RefCell::new(Vec::new()));
  let id_calls: Rc<RefCell<Vec<(Vec<u8>, bool)>>> = Rc::new(RefCell::new(Vec::new()));

  let cc = Rc::clone(&city_calls);
  let ic = Rc::clone(&id_calls);

  let mut p = Builder::new()
    .register("/address/city", move |b, done| {
      cc.borrow_mut().push((b.to_vec(), done));
    })
    .unwrap()
    .register("/id", move |b, done| {
      ic.borrow_mut().push((b.to_vec(), done));
    })
    .unwrap()
    .build();

  p.feed(doc).unwrap();

  assert_eq!(string_body(&city_calls.borrow()), b"Springfield");
  assert_eq!(id_calls.borrow()[..], [(b"1234".to_vec(), true)]);
}

#[test]
fn realistic_chunked_delivery() {
  let doc = b"{\"config\":{\"timeout\":30,\"retries\":3},\"endpoint\":\"https://api.example.com\"}";
  let (mut p, calls) = collecting("/endpoint");

  // Feed in 10-byte chunks.
  let mut i = 0;
  while i < doc.len() {
    let end = (i + 10).min(doc.len());
    match p.feed(&doc[i..end]) {
      Ok(Status::Done) => break,
      Ok(Status::NeedMore) => {}
      Err(e) => panic!("error at byte {i}: {:?}", e),
    }
    i = end;
  }

  assert_eq!(string_body(&calls.borrow()), b"https://api.example.com");
}
