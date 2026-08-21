use std::collections::HashMap;

/// Errors returned by [`Builder::register`] when a JSON Pointer is invalid.
#[derive(Debug, PartialEq)]
pub enum PointerError {
  /// The pointer does not start with `'/'` (and is not the empty string `""`
  /// that refers to the document root).
  MissingLeadingSlash,
  /// A `~` escape in the pointer is not followed by `0` or `1`.
  InvalidEscape,
}

impl std::fmt::Display for PointerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PointerError::MissingLeadingSlash => {
        write!(f, "JSON Pointer must start with '/'")
      }
      PointerError::InvalidEscape => {
        write!(
          f,
          "invalid escape sequence in JSON Pointer (~ must be followed by 0 or 1)"
        )
      }
    }
  }
}

impl std::error::Error for PointerError {}

pub type Callback = Box<dyn Fn(&[u8], bool)>;

/// A node in the JSON Pointer path trie.
///
/// Each edge is a decoded path segment (after `~0`/`~1` unescaping).
/// Leaf nodes (those with a `callback`) represent fully registered paths.
pub(crate) struct TrieNode {
  /// Children indexed by decoded segment string.
  pub(crate) children: HashMap<String, TrieNode>,
  /// Callback to invoke when this path's value is found.
  /// Called one or more times with `(bytes, is_complete)`.
  pub(crate) callback: Option<Callback>,
}

impl TrieNode {
  pub(crate) fn new() -> Self {
    Self {
      children: HashMap::new(),
      callback: None,
    }
  }

  /// Returns the total number of registered paths in this subtree.
  #[cfg(test)]
  pub(crate) fn path_count(&self) -> usize {
    let self_count = if self.callback.is_some() { 1 } else { 0 };
    self_count
      + self
        .children
        .values()
        .map(|n| n.path_count())
        .sum::<usize>()
  }

  fn insert(&mut self, segments: &[String], callback: Callback) {
    match segments {
      [] => {
        self.callback = Some(callback);
      }
      [head, tail @ ..] => {
        self
          .children
          .entry(head.clone())
          .or_insert_with(TrieNode::new)
          .insert(tail, callback);
      }
    }
  }
}

/// Parses a JSON Pointer string into a list of decoded reference token strings.
///
/// An empty string refers to the whole document (returns an empty vec).
/// All other valid pointers begin with '/'.
pub(crate) fn parse_pointer(pointer: &str) -> Result<Vec<String>, PointerError> {
  if pointer.is_empty() {
    return Ok(vec![]);
  }
  if !pointer.starts_with('/') {
    return Err(PointerError::MissingLeadingSlash);
  }
  pointer[1..].split('/').map(decode_segment).collect()
}

/// Decodes a single JSON Pointer reference token.
///
/// Per RFC 6901: `~1` → `/`, `~0` → `~` (in that order, left-to-right).
/// Any lone `~` not followed by `0` or `1` is an error.
pub(crate) fn decode_segment(segment: &str) -> Result<String, PointerError> {
  let mut result = String::with_capacity(segment.len());
  let mut chars = segment.chars();
  while let Some(c) = chars.next() {
    if c == '~' {
      match chars.next() {
        Some('0') => result.push('~'),
        Some('1') => result.push('/'),
        _ => return Err(PointerError::InvalidEscape),
      }
    } else {
      result.push(c);
    }
  }
  Ok(result)
}

/// Builder for constructing a [`Parser`](crate::Parser) with registered JSON Pointer paths.
///
/// Call [`register`](Self::register) for each path of interest, then
/// [`build`](Self::build) to produce a [`Parser`](crate::Parser).
pub struct Builder {
  pub(crate) root: TrieNode,
}

impl Builder {
  /// Creates a new, empty builder with no registered paths.
  pub fn new() -> Self {
    Self {
      root: TrieNode::new(),
    }
  }

  /// Registers a JSON Pointer path and the callback to invoke when its value is found.
  ///
  /// The callback signature is `fn(bytes: &[u8], is_complete: bool)`:
  /// - For **string** values the callback may fire multiple times as chunks arrive.
  ///   `is_complete` is `false` on intermediate calls and `true` on the final call
  ///   (which delivers an empty slice after the closing `"`). Raw JSON escape
  ///   sequences are forwarded as-is; decoding is the caller's responsibility.
  /// - For **numbers, booleans, and `null`** the callback fires exactly once with
  ///   `is_complete = true` and the full raw value bytes (e.g. `b"42"`, `b"true"`).
  ///
  /// Registering the same `pointer` twice overwrites the earlier callback.
  ///
  /// # Arguments
  ///
  /// * `pointer` — an RFC 6901 JSON Pointer such as `"/foo/bar"`,
  ///   `"/items/0/name"`, or `""` for the document root.
  ///   Use `~0` for a literal `~` and `~1` for a literal `/` in a key name.
  /// * `callback` — closure invoked with each chunk of raw JSON bytes for the
  ///   matched value.
  ///
  /// # Errors
  ///
  /// Returns [`PointerError`] if `pointer` is not a valid RFC 6901 JSON Pointer.
  pub fn register(
    mut self,
    pointer: &str,
    callback: impl Fn(&[u8], bool) + 'static,
  ) -> Result<Self, PointerError> {
    let segments = parse_pointer(pointer)?;
    self.root.insert(&segments, Box::new(callback));
    Ok(self)
  }
}

impl Default for Builder {
  /// Creates a new, empty builder. Equivalent to [`Builder::new`].
  fn default() -> Self {
    Self::new()
  }
}

/// A node in the flattened trie.
pub(crate) struct FlatNode {
  /// Children sorted by segment string for binary-search lookup.
  pub(crate) children: Vec<(String, u32)>,
  /// Callback if this node represents a registered path terminal.
  pub(crate) callback: Option<Callback>,
}

/// Flattened representation of the path trie.
///
/// All nodes live in a single `Vec`; the root is always at index `0`.
/// Children are stored as sorted `(segment, index)` pairs so lookup
/// is a binary search — no heap allocation per query.
pub(crate) struct FlatTrie {
  pub(crate) nodes: Vec<FlatNode>,
  /// Total number of registered paths (used to detect early-exit).
  pub(crate) total_paths: usize,
}

impl FlatTrie {
  /// Returns the index of the child node reached by `segment` from
  /// `node_idx`, or `None` if no such edge exists.
  pub(crate) fn child(&self, node_idx: u32, segment: &str) -> Option<u32> {
    let children = &self.nodes[node_idx as usize].children;
    children
      .binary_search_by(|(s, _)| s.as_str().cmp(segment))
      .ok()
      .map(|i| children[i].1)
  }
}

/// Recursively flattens `node` into `nodes` using pre-order DFS.
/// Returns the index assigned to `node`.
pub(crate) fn flatten(node: TrieNode, nodes: &mut Vec<FlatNode>) -> u32 {
  let idx = nodes.len() as u32;
  // Push a placeholder so the index is reserved before recursing.
  nodes.push(FlatNode {
    children: Vec::new(),
    callback: None,
  });

  let mut children: Vec<(String, u32)> = node
    .children
    .into_iter()
    .map(|(segment, child)| {
      let child_idx = flatten(child, nodes);
      (segment, child_idx)
    })
    .collect();

  // Sort so `child()` can use binary search.
  children.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

  nodes[idx as usize] = FlatNode {
    children,
    callback: node.callback,
  };

  idx
}

pub(crate) fn count_paths(node: &TrieNode) -> usize {
  let own = if node.callback.is_some() { 1 } else { 0 };
  own + node.children.values().map(count_paths).sum::<usize>()
}

#[cfg(test)]
mod tests {
  use super::*;

  // --- decode_segment ---

  #[test]
  fn decode_plain_segment() {
    assert_eq!(decode_segment("foo").unwrap(), "foo");
  }

  #[test]
  fn decode_empty_segment() {
    assert_eq!(decode_segment("").unwrap(), "");
  }

  #[test]
  fn decode_tilde_zero() {
    assert_eq!(decode_segment("~0").unwrap(), "~");
  }

  #[test]
  fn decode_tilde_one() {
    assert_eq!(decode_segment("~1").unwrap(), "/");
  }

  #[test]
  fn decode_tilde_zero_then_one() {
    // "~01" must decode to "~1" (literal tilde + literal 1), not "/"
    // Processing left-to-right: ~0 → ~, then remaining "1" → "1" → "~1"
    assert_eq!(decode_segment("~01").unwrap(), "~1");
  }

  #[test]
  fn decode_tilde_one_then_zero() {
    // "~10" → "/" + "0" → "/0"
    assert_eq!(decode_segment("~10").unwrap(), "/0");
  }

  #[test]
  fn decode_mixed_escapes() {
    assert_eq!(decode_segment("a~1b~0c").unwrap(), "a/b~c");
  }

  #[test]
  fn decode_lone_tilde_is_error() {
    assert_eq!(
      decode_segment("~").unwrap_err(),
      PointerError::InvalidEscape
    );
  }

  #[test]
  fn decode_tilde_invalid_char_is_error() {
    assert_eq!(
      decode_segment("~2").unwrap_err(),
      PointerError::InvalidEscape
    );
    assert_eq!(
      decode_segment("~a").unwrap_err(),
      PointerError::InvalidEscape
    );
  }

  // --- parse_pointer ---

  #[test]
  fn parse_empty_pointer_is_root() {
    assert_eq!(parse_pointer("").unwrap(), Vec::<String>::new());
  }

  #[test]
  fn parse_single_segment() {
    assert_eq!(parse_pointer("/foo").unwrap(), vec!["foo"]);
  }

  #[test]
  fn parse_multiple_segments() {
    assert_eq!(
      parse_pointer("/foo/bar/baz").unwrap(),
      vec!["foo", "bar", "baz"]
    );
  }

  #[test]
  fn parse_array_index_segment() {
    assert_eq!(
      parse_pointer("/foo/0/bar").unwrap(),
      vec!["foo", "0", "bar"]
    );
  }

  #[test]
  fn parse_empty_key_segment() {
    // "/foo//bar" has an empty segment between the two slashes
    assert_eq!(parse_pointer("/foo//bar").unwrap(), vec!["foo", "", "bar"]);
  }

  #[test]
  fn parse_single_slash_is_empty_key() {
    // "/" refers to the key "" in the root object
    assert_eq!(parse_pointer("/").unwrap(), vec![""]);
  }

  #[test]
  fn parse_with_escapes() {
    assert_eq!(parse_pointer("/a~1b/c~0d").unwrap(), vec!["a/b", "c~d"]);
  }

  #[test]
  fn parse_missing_leading_slash() {
    assert_eq!(
      parse_pointer("foo").unwrap_err(),
      PointerError::MissingLeadingSlash
    );
  }

  #[test]
  fn parse_invalid_escape_propagates() {
    assert_eq!(
      parse_pointer("/foo/~2/bar").unwrap_err(),
      PointerError::InvalidEscape
    );
  }

  // --- TrieNode::path_count ---

  #[test]
  fn empty_trie_has_zero_paths() {
    assert_eq!(TrieNode::new().path_count(), 0);
  }

  // --- Builder ---

  #[test]
  fn builder_single_path() {
    let builder = Builder::new().register("/foo/bar", |_, _| {}).unwrap();

    let foo = builder.root.children.get("foo").expect("foo child");
    assert!(foo.callback.is_none());
    let bar = foo.children.get("bar").expect("bar child");
    assert!(bar.callback.is_some());
    assert_eq!(builder.root.path_count(), 1);
  }

  #[test]
  fn builder_multiple_disjoint_paths() {
    let builder = Builder::new()
      .register("/foo", |_, _| {})
      .unwrap()
      .register("/baz", |_, _| {})
      .unwrap();

    assert!(builder.root.children.get("foo").unwrap().callback.is_some());
    assert!(builder.root.children.get("baz").unwrap().callback.is_some());
    assert_eq!(builder.root.path_count(), 2);
  }

  #[test]
  fn builder_overlapping_paths() {
    // /foo and /foo/bar: /foo is a registered leaf, /foo/bar is deeper
    let builder = Builder::new()
      .register("/foo", |_, _| {})
      .unwrap()
      .register("/foo/bar", |_, _| {})
      .unwrap();

    let foo = builder.root.children.get("foo").unwrap();
    assert!(foo.callback.is_some());
    let bar = foo.children.get("bar").unwrap();
    assert!(bar.callback.is_some());
    assert_eq!(builder.root.path_count(), 2);
  }

  #[test]
  fn builder_array_index_path() {
    let builder = Builder::new().register("/items/0/name", |_, _| {}).unwrap();

    let items = builder.root.children.get("items").unwrap();
    let zero = items.children.get("0").unwrap();
    let name = zero.children.get("name").unwrap();
    assert!(name.callback.is_some());
  }

  #[test]
  fn builder_invalid_pointer_returns_error() {
    let result = Builder::new().register("no-leading-slash", |_, _| {});
    assert_eq!(result.err(), Some(PointerError::MissingLeadingSlash));
  }

  #[test]
  fn builder_invalid_escape_returns_error() {
    let result = Builder::new().register("/foo/~9/bar", |_, _| {});
    assert_eq!(result.err(), Some(PointerError::InvalidEscape));
  }

  #[test]
  fn builder_root_pointer() {
    // Empty pointer "" refers to the whole document — registered at root node
    let builder = Builder::new().register("", |_, _| {}).unwrap();
    assert!(builder.root.callback.is_some());
    assert_eq!(builder.root.path_count(), 1);
  }

  #[test]
  fn builder_escaped_key_path() {
    // /a~1b registers a child with decoded key "a/b"
    let builder = Builder::new().register("/a~1b", |_, _| {}).unwrap();
    assert!(builder.root.children.contains_key("a/b"));
    assert!(!builder.root.children.contains_key("a~1b"));
  }

  // --- FlatTrie / flatten / count_paths ---

  fn make_flat_trie(builder: Builder) -> FlatTrie {
    let total_paths = count_paths(&builder.root);
    let mut nodes = Vec::new();
    flatten(builder.root, &mut nodes);
    FlatTrie { nodes, total_paths }
  }

  #[test]
  fn build_single_path() {
    // /foo/bar → root(0) → foo(1) → bar(2)
    let trie = make_flat_trie(Builder::new().register("/foo/bar", |_, _| {}).unwrap());

    assert_eq!(trie.nodes.len(), 3);
    assert_eq!(trie.total_paths, 1);

    // root has one child "foo" at index 1
    assert_eq!(trie.nodes[0].children, vec![("foo".to_string(), 1)]);
    assert!(trie.nodes[0].callback.is_none());

    // foo has one child "bar" at index 2
    assert_eq!(trie.nodes[1].children, vec![("bar".to_string(), 2)]);
    assert!(trie.nodes[1].callback.is_none());

    // bar is a leaf with a callback
    assert!(trie.nodes[2].children.is_empty());
    assert!(trie.nodes[2].callback.is_some());
  }

  #[test]
  fn build_disjoint_paths_children_are_sorted() {
    // /foo and /baz → root should have children sorted: ["baz", "foo"]
    let trie = make_flat_trie(
      Builder::new()
        .register("/foo", |_, _| {})
        .unwrap()
        .register("/baz", |_, _| {})
        .unwrap(),
    );

    assert_eq!(trie.total_paths, 2);
    let root_keys: Vec<&str> = trie.nodes[0]
      .children
      .iter()
      .map(|(s, _)| s.as_str())
      .collect();
    assert_eq!(root_keys, vec!["baz", "foo"]);

    // Both children have callbacks
    for (_, idx) in &trie.nodes[0].children {
      assert!(trie.nodes[*idx as usize].callback.is_some());
    }
  }

  #[test]
  fn build_overlapping_paths() {
    // /foo (leaf) and /foo/bar (deeper leaf)
    let trie = make_flat_trie(
      Builder::new()
        .register("/foo", |_, _| {})
        .unwrap()
        .register("/foo/bar", |_, _| {})
        .unwrap(),
    );

    assert_eq!(trie.total_paths, 2);

    let foo_idx = trie.child(0, "foo").expect("foo child");
    assert!(trie.nodes[foo_idx as usize].callback.is_some());

    let bar_idx = trie.child(foo_idx, "bar").expect("bar child");
    assert!(trie.nodes[bar_idx as usize].callback.is_some());
  }

  #[test]
  fn build_root_pointer() {
    // "" registers callback at root node (index 0)
    let trie = make_flat_trie(Builder::new().register("", |_, _| {}).unwrap());

    assert_eq!(trie.nodes.len(), 1);
    assert_eq!(trie.total_paths, 1);
    assert!(trie.nodes[0].callback.is_some());
  }

  #[test]
  fn build_empty_no_paths() {
    let trie = make_flat_trie(Builder::new());
    assert_eq!(trie.nodes.len(), 1); // root always exists
    assert_eq!(trie.total_paths, 0);
    assert!(trie.nodes[0].children.is_empty());
    assert!(trie.nodes[0].callback.is_none());
  }

  #[test]
  fn child_lookup_hit() {
    let trie = make_flat_trie(Builder::new().register("/foo/bar", |_, _| {}).unwrap());

    assert!(trie.child(0, "foo").is_some());
    let foo_idx = trie.child(0, "foo").unwrap();
    assert!(trie.child(foo_idx, "bar").is_some());
  }

  #[test]
  fn child_lookup_miss() {
    let trie = make_flat_trie(Builder::new().register("/foo", |_, _| {}).unwrap());
    assert!(trie.child(0, "missing").is_none());
  }

  #[test]
  fn build_array_index_path() {
    let trie = make_flat_trie(Builder::new().register("/items/0/name", |_, _| {}).unwrap());

    let items = trie.child(0, "items").unwrap();
    let zero = trie.child(items, "0").unwrap();
    let name = trie.child(zero, "name").unwrap();
    assert!(trie.nodes[name as usize].callback.is_some());
  }

  #[test]
  fn build_total_paths_counts_all() {
    let trie = make_flat_trie(
      Builder::new()
        .register("/a", |_, _| {})
        .unwrap()
        .register("/b", |_, _| {})
        .unwrap()
        .register("/c/d", |_, _| {})
        .unwrap(),
    );
    assert_eq!(trie.total_paths, 3);
  }

  #[test]
  fn builder_same_path_twice_overwrites() {
    use std::sync::{Arc, Mutex};

    let first_called = Arc::new(Mutex::new(false));
    let second_called = Arc::new(Mutex::new(false));

    let first = Arc::clone(&first_called);
    let second = Arc::clone(&second_called);

    let builder = Builder::new()
      .register("/foo", move |_, _| {
        *first.lock().unwrap() = true;
      })
      .unwrap()
      .register("/foo", move |_, _| {
        *second.lock().unwrap() = true;
      })
      .unwrap();

    // Only one callback at /foo — the second registration wins
    assert_eq!(builder.root.path_count(), 1);
    let foo = builder.root.children.get("foo").unwrap();
    let cb = foo.callback.as_ref().unwrap();
    cb(&[], true);

    assert!(!*first_called.lock().unwrap());
    assert!(*second_called.lock().unwrap());
  }
}
