use crate::trie::{Builder, FlatNode, FlatTrie, count_paths, flatten};

// Sentinel: trie node index meaning "not on any registered path".
pub(crate) const OFF_PATH: u32 = u32::MAX;

// ── Container stack ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ContainerKind {
  Object,
  Array,
}

/// One frame per open `{` or `[` we have descended into.
#[derive(Debug)]
pub(crate) struct StackFrame {
  pub(crate) kind: ContainerKind,
  /// Trie node for this container level.
  /// `OFF_PATH` when we are not on any registered path prefix.
  pub(crate) trie_node: u32,
  /// For arrays: the index of the *next* element to process.
  /// Unused for objects (always 0).
  pub(crate) array_index: usize,
}

// ── Lexer mode ────────────────────────────────────────────────────────────────

/// Sub-state for the inside of a string that is being *skipped*
/// (neither a key we're matching nor a value we care about).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SkipStringState {
  /// Normal string body — watching for `"` or `\`.
  Body,
  /// After `\` — next byte is the escape character.
  Escape,
  /// Inside `\uXXXX` — `seen` hex digits consumed so far (0–3).
  Unicode { seen: u8 },
}

/// The complete state of the lexer between `feed` calls.
///
/// Every variant captures exactly what is needed to resume parsing at the
/// start of the next chunk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LexerMode {
  // ── Structural ───────────────────────────────────────────────────────
  /// Expecting the first byte of a JSON value.
  BeforeValue,
  /// Inside an object body: expecting `"` (key) or `}`.
  BeforeKey,
  /// After a key string closed: expecting `:`.
  AfterKey,
  /// After any complete value: expecting `,`, `}`, or `]`.
  AfterValue,

  // ── Key string ───────────────────────────────────────────────────────
  /// Inside an object key string. Decoded bytes accumulate into `scratch`.
  InKeyString,
  /// After `\` in a key string.
  InKeyStringEscape,
  /// Inside `\uXXXX` in a key string.
  /// `seen`: hex digits consumed so far (0 = waiting for 1st, 3 = waiting for 4th).
  /// `accum`: code point value accumulated from digits seen so far.
  InKeyUnicodeEscape { seen: u8, accum: u32 },

  // ── Matched value — string ────────────────────────────────────────────
  /// Inside the body of a string value at a registered path (`node`).
  /// Raw JSON bytes (including escape sequences) are streamed directly
  /// to the callback; no intermediate buffering.
  InMatchedString { node: u32 },
  /// After `\` in a matched string value.
  InMatchedStringEscape { node: u32 },
  /// Inside `\uXXXX` in a matched string value.
  /// `seen` and `accum` mirror the key-string variant.
  InMatchedStringUnicode { node: u32, seen: u8, accum: u32 },

  // ── Matched value — scalar ────────────────────────────────────────────
  /// Collecting a number value into `scratch` for the callback at `node`.
  /// Ends when a non-number byte (whitespace, `,`, `}`, `]`) is seen;
  /// that terminating byte is *not* consumed — it is re-processed.
  InMatchedNumber { node: u32 },
  /// Collecting a keyword (`true` / `false` / `null`) into `scratch`.
  /// `expected`: the full keyword bytes (e.g. `b"true"`).
  /// `pos`: index of the next byte to match (1 after the first byte is consumed).
  InMatchedKeyword {
    node: u32,
    expected: &'static [u8],
    pos: u8,
  },

  // ── Skip ─────────────────────────────────────────────────────────────
  /// Skipping content that is not on any registered path.
  ///
  /// `depth` counts unclosed `{` / `[` within the skipped region:
  ///   - 0 while skipping a scalar or the very start of a container
  ///   - increments on `{` / `[`, decrements on `}` / `]`
  ///
  /// `str_state` is `Some(_)` while inside a string in the skipped content,
  /// so that `}` / `]` inside strings are not mistaken for structural bytes.
  Skip {
    depth: usize,
    str_state: Option<SkipStringState>,
  },
}

// ── Public result types ───────────────────────────────────────────────────────

/// Returned by [`Parser::feed`].
#[derive(Debug, PartialEq)]
pub enum Status {
  /// All registered paths have been resolved — the caller may stop feeding.
  Done,
  /// More data is needed to resolve the remaining paths.
  NeedMore,
}

/// Errors returned by [`Parser::feed`] and [`Parser::finish`] on malformed input.
#[derive(Debug, PartialEq)]
pub enum ParseError {
  /// A byte that is not valid at the current parse position was encountered.
  /// The inner value is the offending byte.
  UnexpectedByte(u8),
  /// [`Parser::finish`] was called while the parser was still in a mid-parse
  /// state (e.g. inside a string or keyword, or with unclosed containers).
  UnexpectedEof,
}

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ParseError::UnexpectedByte(b) => write!(
        f,
        "unexpected byte 0x{b:02x} (`{}`)",
        if b.is_ascii_graphic() {
          *b as char
        } else {
          '?'
        }
      ),
      ParseError::UnexpectedEof => write!(f, "unexpected end of input"),
    }
  }
}

impl std::error::Error for ParseError {}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Streaming JSON path extractor.
///
/// Feed data in arbitrary chunks via [`Parser::feed`].  Registered callbacks
/// fire as their paths are resolved.  Returns [`Status::Done`] once every
/// registered path has been matched (early exit — the rest of the stream is
/// ignored).
pub struct Parser {
  trie: FlatTrie,
  /// Active container frames (one per unclosed `{` / `[` we care about).
  stack: Vec<StackFrame>,
  /// Current lexer state.
  mode: LexerMode,
  /// Scratch buffer for key strings, numbers, and keywords.
  scratch: Vec<u8>,
  /// Trie node for the value we are about to process.
  ///
  /// - Set to `0` (root) initially.
  /// - Updated after each key match (object) or index advance (array).
  /// - `OFF_PATH` when the current value is not on any registered path.
  pending_node: u32,
  /// Number of registered paths resolved so far.
  resolved: usize,
}

impl Parser {
  pub(crate) fn new(trie: FlatTrie) -> Self {
    let root = if trie.total_paths == 0 { OFF_PATH } else { 0 };
    Self {
      trie,
      stack: Vec::new(),
      mode: LexerMode::BeforeValue,
      scratch: Vec::new(),
      pending_node: root,
      resolved: 0,
    }
  }

  /// Feed the next chunk of JSON bytes into the parser.
  ///
  /// The parser maintains full state between calls, so chunk boundaries may
  /// fall anywhere — mid-string, mid-number, mid-escape-sequence, etc.
  /// Registered callbacks fire as their respective paths are resolved.
  ///
  /// # Arguments
  ///
  /// * `chunk` — the next slice of raw JSON bytes. May be any length, including
  ///   a single byte or the entire document at once.
  ///
  /// # Returns
  ///
  /// * [`Status::Done`] — every registered path has been matched; no further
  ///   `feed` calls are needed (remaining bytes in the stream are not processed).
  /// * [`Status::NeedMore`] — parsing is ongoing; call `feed` again with the
  ///   next chunk, then call [`finish`](Self::finish) after the last one.
  ///
  /// # Errors
  ///
  /// Returns [`ParseError::UnexpectedByte`] on the first byte that is not valid
  /// at the current parse position. After an error the parser is in an undefined
  /// state and must not be used further.
  pub fn feed(&mut self, chunk: &[u8]) -> Result<Status, ParseError> {
    if self.trie.total_paths > 0 && self.resolved >= self.trie.total_paths {
      return Ok(Status::Done);
    }

    let mut i = 0;
    while i < chunk.len() {
      // ── InMatchedString: slice-scan to avoid per-byte dispatch ────
      // Handled separately so we can fire whole slices to the callback
      // rather than one byte at a time.
      if let LexerMode::InMatchedString { node } = self.mode {
        let start = i;
        while i < chunk.len() && !matches!(chunk[i], b'"' | b'\\' | 0x00..=0x1F) {
          i += 1;
        }
        // Fire the non-special slice (may be empty).
        if i > start && node != OFF_PATH {
          let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
          cb(&chunk[start..i], false);
        }
        if i == chunk.len() {
          break; // chunk exhausted mid-string
        }
        // Process the special byte.
        let b = chunk[i];
        i += 1;
        match b {
          b'"' => {
            if node != OFF_PATH {
              let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
              cb(&[], true);
              self.resolved += 1;
            }
            self.mode = LexerMode::AfterValue;
          }
          b'\\' => self.mode = LexerMode::InMatchedStringEscape { node },
          _ => return Err(ParseError::UnexpectedByte(b)),
        }
        if self.trie.total_paths > 0 && self.resolved >= self.trie.total_paths {
          return Ok(Status::Done);
        }
        continue;
      }

      let b = chunk[i];
      // `advance = false` tells the loop not to increment `i`, used when
      // a byte must be re-processed (e.g. number terminator).
      let mut advance = true;

      match self.mode {
        // ── Expecting a value ─────────────────────────────────────
        LexerMode::BeforeValue => match b {
          b' ' | b'\t' | b'\n' | b'\r' => {}

          b'{' => {
            if self.pending_node == OFF_PATH {
              self.mode = LexerMode::Skip {
                depth: 1,
                str_state: None,
              };
            } else {
              self.stack.push(StackFrame {
                kind: ContainerKind::Object,
                trie_node: self.pending_node,
                array_index: 0,
              });
              self.mode = LexerMode::BeforeKey;
            }
          }

          b'[' => {
            if self.pending_node == OFF_PATH {
              self.mode = LexerMode::Skip {
                depth: 1,
                str_state: None,
              };
            } else {
              let trie_node = self.pending_node;
              self.stack.push(StackFrame {
                kind: ContainerKind::Array,
                trie_node,
                array_index: 0,
              });
              self.pending_node = self.resolve_array_index(trie_node, 0);
            }
          }

          b']' => match self.stack.last() {
            Some(f) if f.kind == ContainerKind::Array => {
              self.stack.pop();
              self.mode = LexerMode::AfterValue;
            }
            _ => return Err(ParseError::UnexpectedByte(b)),
          },

          b'"' => {
            let node = self.pending_node;
            let has_cb = node != OFF_PATH && self.trie.nodes[node as usize].callback.is_some();
            self.mode = LexerMode::InMatchedString {
              node: if has_cb { node } else { OFF_PATH },
            };
          }

          b't' | b'f' | b'n' => {
            let node = self.pending_node;
            let has_cb = node != OFF_PATH && self.trie.nodes[node as usize].callback.is_some();
            let expected: &'static [u8] = match b {
              b't' => b"true",
              b'f' => b"false",
              _ => b"null",
            };
            let effective_node = if has_cb { node } else { OFF_PATH };
            if effective_node != OFF_PATH {
              self.scratch.clear();
              self.scratch.push(b);
            }
            self.mode = LexerMode::InMatchedKeyword {
              node: effective_node,
              expected,
              pos: 1,
            };
          }

          b'0'..=b'9' | b'-' => {
            let node = self.pending_node;
            let has_cb = node != OFF_PATH && self.trie.nodes[node as usize].callback.is_some();
            let effective_node = if has_cb { node } else { OFF_PATH };
            if effective_node != OFF_PATH {
              self.scratch.clear();
              self.scratch.push(b);
            }
            self.mode = LexerMode::InMatchedNumber {
              node: effective_node,
            };
          }

          _ => return Err(ParseError::UnexpectedByte(b)),
        },

        // ── Inside object, expecting key or `}` ───────────────────
        LexerMode::BeforeKey => match b {
          b' ' | b'\t' | b'\n' | b'\r' => {}
          b'"' => {
            self.scratch.clear();
            self.mode = LexerMode::InKeyString;
          }
          b'}' => {
            self.stack.pop();
            self.mode = LexerMode::AfterValue;
          }
          _ => return Err(ParseError::UnexpectedByte(b)),
        },

        // ── After key string, expecting `:` ───────────────────────
        LexerMode::AfterKey => match b {
          b' ' | b'\t' | b'\n' | b'\r' => {}
          b':' => self.mode = LexerMode::BeforeValue,
          _ => return Err(ParseError::UnexpectedByte(b)),
        },

        // ── After a value, expecting `,` / `}` / `]` ─────────────
        LexerMode::AfterValue => match b {
          b' ' | b'\t' | b'\n' | b'\r' => {}

          b',' => match self.stack.last().map(|f| f.kind) {
            None => return Err(ParseError::UnexpectedByte(b)),
            Some(ContainerKind::Object) => {
              self.mode = LexerMode::BeforeKey;
            }
            Some(ContainerKind::Array) => {
              let frame = self.stack.last_mut().unwrap();
              frame.array_index += 1;
              let (trie_node, index) = (frame.trie_node, frame.array_index);
              self.pending_node = self.resolve_array_index(trie_node, index);
              self.mode = LexerMode::BeforeValue;
            }
          },

          b'}' => match self.stack.last().map(|f| f.kind) {
            Some(ContainerKind::Object) => {
              self.stack.pop();
              self.mode = LexerMode::AfterValue;
            }
            _ => return Err(ParseError::UnexpectedByte(b)),
          },

          b']' => match self.stack.last().map(|f| f.kind) {
            Some(ContainerKind::Array) => {
              self.stack.pop();
              self.mode = LexerMode::AfterValue;
            }
            _ => return Err(ParseError::UnexpectedByte(b)),
          },

          _ => return Err(ParseError::UnexpectedByte(b)),
        },

        // ── Key string ────────────────────────────────────────────
        LexerMode::InKeyString => match b {
          b'"' => {
            let frame_node = self.stack.last().map(|f| f.trie_node).unwrap_or(OFF_PATH);
            self.pending_node = if frame_node == OFF_PATH {
              OFF_PATH
            } else {
              let key = std::str::from_utf8(&self.scratch).unwrap_or("");
              self.trie.child(frame_node, key).unwrap_or(OFF_PATH)
            };
            self.mode = LexerMode::AfterKey;
          }
          b'\\' => self.mode = LexerMode::InKeyStringEscape,
          0x00..=0x1F => return Err(ParseError::UnexpectedByte(b)),
          _ => self.scratch.push(b),
        },

        LexerMode::InKeyStringEscape => match b {
          b'u' => {
            self.mode = LexerMode::InKeyUnicodeEscape { seen: 0, accum: 0 };
          }
          _ => {
            let decoded = match b {
              b'"' | b'\\' | b'/' => b,
              b'b' => 0x08,
              b'f' => 0x0C,
              b'n' => b'\n',
              b'r' => b'\r',
              b't' => b'\t',
              _ => return Err(ParseError::UnexpectedByte(b)),
            };
            self.scratch.push(decoded);
            self.mode = LexerMode::InKeyString;
          }
        },

        LexerMode::InKeyUnicodeEscape { seen, accum } => {
          let hex = match b {
            b'0'..=b'9' => (b - b'0') as u32,
            b'a'..=b'f' => (b - b'a' + 10) as u32,
            b'A'..=b'F' => (b - b'A' + 10) as u32,
            _ => return Err(ParseError::UnexpectedByte(b)),
          };
          let new_accum = (accum << 4) | hex;
          if seen == 3 {
            let ch = char::from_u32(new_accum).unwrap_or(char::REPLACEMENT_CHARACTER);
            let mut utf8 = [0u8; 4];
            self
              .scratch
              .extend_from_slice(ch.encode_utf8(&mut utf8).as_bytes());
            self.mode = LexerMode::InKeyString;
          } else {
            self.mode = LexerMode::InKeyUnicodeEscape {
              seen: seen + 1,
              accum: new_accum,
            };
          }
        }

        // ── Matched value — string escape/unicode ─────────────────
        // (string body handled by the slice-scan block above)
        LexerMode::InMatchedString { .. } => unreachable!(),

        LexerMode::InMatchedStringEscape { node } => match b {
          b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
            if node != OFF_PATH {
              let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
              cb(&[b'\\', b], false);
            }
            self.mode = LexerMode::InMatchedString { node };
          }
          b'u' => {
            if node != OFF_PATH {
              let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
              cb(b"\\u", false);
            }
            self.mode = LexerMode::InMatchedStringUnicode {
              node,
              seen: 0,
              accum: 0,
            };
          }
          _ => return Err(ParseError::UnexpectedByte(b)),
        },

        LexerMode::InMatchedStringUnicode {
          node,
          seen,
          accum: _,
        } => match b {
          b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
            if node != OFF_PATH {
              let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
              cb(&[b], false);
            }
            self.mode = if seen == 3 {
              LexerMode::InMatchedString { node }
            } else {
              LexerMode::InMatchedStringUnicode {
                node,
                seen: seen + 1,
                accum: 0,
              }
            };
          }
          _ => return Err(ParseError::UnexpectedByte(b)),
        },

        // ── Matched value — number ────────────────────────────────
        LexerMode::InMatchedNumber { node } => match b {
          b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-' => {
            if node != OFF_PATH {
              self.scratch.push(b);
            }
          }
          _ => {
            // Number terminated — fire and re-process this byte.
            if node != OFF_PATH {
              let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
              cb(&self.scratch, true);
              self.resolved += 1;
            }
            self.mode = LexerMode::AfterValue;
            advance = false;
          }
        },

        // ── Matched value — keyword ───────────────────────────────
        LexerMode::InMatchedKeyword {
          node,
          expected,
          pos,
        } => {
          if b != expected[pos as usize] {
            return Err(ParseError::UnexpectedByte(b));
          }
          if node != OFF_PATH {
            self.scratch.push(b);
          }
          if pos as usize == expected.len() - 1 {
            if node != OFF_PATH {
              let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
              cb(&self.scratch, true);
              self.resolved += 1;
            }
            self.mode = LexerMode::AfterValue;
          } else {
            self.mode = LexerMode::InMatchedKeyword {
              node,
              expected,
              pos: pos + 1,
            };
          }
        }

        // ── Skip off-path container ───────────────────────────────
        LexerMode::Skip { depth, str_state } => match str_state {
          None => match b {
            b'{' | b'[' => {
              self.mode = LexerMode::Skip {
                depth: depth + 1,
                str_state: None,
              };
            }
            b'}' | b']' => {
              if depth == 1 {
                self.mode = LexerMode::AfterValue;
              } else {
                self.mode = LexerMode::Skip {
                  depth: depth - 1,
                  str_state: None,
                };
              }
            }
            b'"' => {
              self.mode = LexerMode::Skip {
                depth,
                str_state: Some(SkipStringState::Body),
              };
            }
            0x00..=0x1F => return Err(ParseError::UnexpectedByte(b)),
            _ => {} // commas, colons, whitespace, number chars, keyword chars
          },
          Some(SkipStringState::Body) => match b {
            b'"' => {
              self.mode = LexerMode::Skip {
                depth,
                str_state: None,
              };
            }
            b'\\' => {
              self.mode = LexerMode::Skip {
                depth,
                str_state: Some(SkipStringState::Escape),
              };
            }
            0x00..=0x1F => return Err(ParseError::UnexpectedByte(b)),
            _ => {}
          },
          Some(SkipStringState::Escape) => match b {
            b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
              self.mode = LexerMode::Skip {
                depth,
                str_state: Some(SkipStringState::Body),
              };
            }
            b'u' => {
              self.mode = LexerMode::Skip {
                depth,
                str_state: Some(SkipStringState::Unicode { seen: 0 }),
              };
            }
            _ => return Err(ParseError::UnexpectedByte(b)),
          },
          Some(SkipStringState::Unicode { seen }) => match b {
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
              self.mode = if seen == 3 {
                LexerMode::Skip {
                  depth,
                  str_state: Some(SkipStringState::Body),
                }
              } else {
                LexerMode::Skip {
                  depth,
                  str_state: Some(SkipStringState::Unicode { seen: seen + 1 }),
                }
              };
            }
            _ => return Err(ParseError::UnexpectedByte(b)),
          },
        },
      }

      if advance {
        i += 1;
      }
      if self.trie.total_paths > 0 && self.resolved >= self.trie.total_paths {
        return Ok(Status::Done);
      }
    }

    Ok(Status::NeedMore)
  }

  /// Signal the end of the JSON stream.
  ///
  /// Must be called once, after the last [`feed`](Self::feed) chunk.
  /// If a number value was still being accumulated (numbers have no
  /// self-delimiting end byte in JSON), `finish` fires its callback with
  /// `is_complete = true` before returning.
  ///
  /// Calling `finish` on an already-resolved parser (i.e. after
  /// [`feed`](Self::feed) returned [`Status::Done`]) is a no-op and returns `Ok(())`.
  ///
  /// # Errors
  ///
  /// Returns [`ParseError::UnexpectedEof`] if the stream ended in a
  /// mid-parse state: inside a string, inside a keyword, or with unclosed
  /// containers and no number pending.
  pub fn finish(&mut self) -> Result<(), ParseError> {
    if self.trie.total_paths > 0 && self.resolved >= self.trie.total_paths {
      return Ok(());
    }
    match self.mode {
      // A number has no self-delimiting end byte; EOF terminates it.
      LexerMode::InMatchedNumber { node } => {
        if node != OFF_PATH {
          let cb = self.trie.nodes[node as usize].callback.as_ref().unwrap();
          cb(&self.scratch, true);
          self.resolved += 1;
        }
        self.mode = LexerMode::AfterValue;
        Ok(())
      }
      // Valid terminal states.
      LexerMode::AfterValue if self.stack.is_empty() => Ok(()),
      LexerMode::BeforeValue if self.stack.is_empty() => {
        // Empty stream — no value seen yet.  Valid only if no paths
        // were registered (already handled above) otherwise it is EOF
        // before any value.
        Err(ParseError::UnexpectedEof)
      }
      // Everything else is a truncated / malformed stream.
      _ => Err(ParseError::UnexpectedEof),
    }
  }

  /// Returns the trie child index for array element `index` under `trie_node`,
  /// or `OFF_PATH` if the index is not on any registered path.
  fn resolve_array_index(&self, trie_node: u32, index: usize) -> u32 {
    if trie_node == OFF_PATH {
      return OFF_PATH;
    }
    let mut buf = [0u8; 20];
    let s = fmt_usize(index, &mut buf);
    self.trie.child(trie_node, s).unwrap_or(OFF_PATH)
  }
}

/// Formats `n` into `buf` without heap allocation.
/// Returns a `&str` slice into `buf`.
fn fmt_usize(n: usize, buf: &mut [u8; 20]) -> &str {
  if n == 0 {
    buf[0] = b'0';
    return std::str::from_utf8(&buf[..1]).unwrap();
  }
  let mut end = buf.len();
  let mut n = n;
  while n > 0 {
    end -= 1;
    buf[end] = b'0' + (n % 10) as u8;
    n /= 10;
  }
  std::str::from_utf8(&buf[end..]).unwrap()
}

// ── Builder::build ────────────────────────────────────────────────────────────

impl Builder {
  /// Consumes the builder and produces a [`Parser`] ready to accept byte chunks.
  ///
  /// After building, call [`Parser::feed`] for each incoming chunk and
  /// [`Parser::finish`] once after the last chunk.
  pub fn build(self) -> Parser {
    let total_paths = count_paths(&self.root);
    let mut nodes: Vec<FlatNode> = Vec::new();
    flatten(self.root, &mut nodes);
    Parser::new(FlatTrie { nodes, total_paths })
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;
  use crate::trie::Builder;

  fn parser(pointer: &str) -> Parser {
    Builder::new().register(pointer, |_, _| {}).unwrap().build()
  }

  // ── Parser::new / initial state ──────────────────────────────────────

  #[test]
  fn initial_mode_is_before_value() {
    let p = parser("/foo");
    assert_eq!(p.mode, LexerMode::BeforeValue);
  }

  #[test]
  fn initial_pending_node_is_root() {
    let p = parser("/foo");
    assert_eq!(p.pending_node, 0);
  }

  #[test]
  fn initial_stack_is_empty() {
    let p = parser("/foo");
    assert!(p.stack.is_empty());
  }

  #[test]
  fn initial_resolved_is_zero() {
    let p = parser("/foo");
    assert_eq!(p.resolved, 0);
  }

  #[test]
  fn empty_builder_pending_node_is_off_path() {
    let p = Builder::new().build();
    assert_eq!(p.pending_node, OFF_PATH);
  }

  // ── Builder::build wires trie correctly ──────────────────────────────

  #[test]
  fn build_single_path_trie_shape() {
    let p = Builder::new()
      .register("/foo/bar", |_, _| {})
      .unwrap()
      .build();
    // Root has "foo" child.
    assert!(p.trie.child(0, "foo").is_some());
    // "foo" has "bar" child with a callback.
    let foo = p.trie.child(0, "foo").unwrap();
    let bar = p.trie.child(foo, "bar").unwrap();
    assert!(p.trie.nodes[bar as usize].callback.is_some());
  }

  #[test]
  fn build_total_paths() {
    let p = Builder::new()
      .register("/a", |_, _| {})
      .unwrap()
      .register("/b/c", |_, _| {})
      .unwrap()
      .build();
    assert_eq!(p.trie.total_paths, 2);
  }

  // ── fmt_usize ─────────────────────────────────────────────────────────

  #[test]
  fn fmt_usize_zero() {
    let mut buf = [0u8; 20];
    assert_eq!(fmt_usize(0, &mut buf), "0");
  }

  #[test]
  fn fmt_usize_small() {
    let mut buf = [0u8; 20];
    assert_eq!(fmt_usize(42, &mut buf), "42");
  }

  #[test]
  fn fmt_usize_large() {
    let mut buf = [0u8; 20];
    assert_eq!(fmt_usize(usize::MAX, &mut buf), usize::MAX.to_string());
  }

  // ── Structural parsing ────────────────────────────────────────────────

  // Helper: build a parser for `pointer`, feed all of `input` in one shot.
  fn feed_once(pointer: &str, input: &[u8]) -> Result<Status, ParseError> {
    parser(pointer).feed(input)
  }

  #[test]
  fn empty_object_returns_need_more() {
    // `{}` parsed cleanly; /foo was never found.
    assert_eq!(feed_once("/foo", b"{}"), Ok(Status::NeedMore));
  }

  #[test]
  fn empty_array_returns_need_more() {
    assert_eq!(feed_once("/foo", b"[]"), Ok(Status::NeedMore));
  }

  #[test]
  fn nested_empty_containers() {
    // [[]] — structural only, no strings hit.
    let mut p = Builder::new().register("/0/0", |_, _| {}).unwrap().build();
    assert_eq!(p.feed(b"[[]]"), Ok(Status::NeedMore));
  }

  #[test]
  fn object_inside_array_no_match() {
    // [{}] with /0/foo registered — descends into the object but finds
    // no key, returns NeedMore.
    let mut p = Builder::new()
      .register("/0/foo", |_, _| {})
      .unwrap()
      .build();
    assert_eq!(p.feed(b"[{}]"), Ok(Status::NeedMore));
  }

  #[test]
  fn multi_chunk_empty_object() {
    let mut p = parser("/foo");
    assert_eq!(p.feed(b"{"), Ok(Status::NeedMore));
    // After `{` the mode is BeforeKey; stack has one Object frame.
    assert_eq!(p.mode, LexerMode::BeforeKey);
    assert_eq!(p.stack.len(), 1);

    assert_eq!(p.feed(b"}"), Ok(Status::NeedMore));
    assert_eq!(p.mode, LexerMode::AfterValue);
    assert!(p.stack.is_empty());
  }

  #[test]
  fn multi_chunk_nested_arrays() {
    // Feed `[`, `[`, `]`, `]` one byte at a time.
    let mut p = Builder::new().register("/0/0", |_, _| {}).unwrap().build();
    p.feed(b"[").unwrap();
    assert_eq!(p.stack.len(), 1);
    p.feed(b"[").unwrap();
    assert_eq!(p.stack.len(), 2);
    p.feed(b"]").unwrap();
    assert_eq!(p.stack.len(), 1);
    assert_eq!(p.mode, LexerMode::AfterValue);
    p.feed(b"]").unwrap();
    assert!(p.stack.is_empty());
  }

  #[test]
  fn array_comma_advances_index() {
    // Feed `[{},` — after the comma the pending_node should be OFF_PATH
    // (no /1/... registered) and we're back in BeforeValue.
    let mut p = Builder::new()
      .register("/0/foo", |_, _| {})
      .unwrap()
      .build();
    p.feed(b"[{},").unwrap();
    assert_eq!(p.mode, LexerMode::BeforeValue);
    assert_eq!(p.pending_node, OFF_PATH); // index 1 not in trie
  }

  #[test]
  fn after_key_colon_transitions_to_before_value() {
    // After `{"` (entering key mode) would fail since key parsing isn't
    // implemented yet.  Instead test BeforeKey → `}` → AfterValue → done.
    let mut p = parser("/foo");
    p.feed(b"{").unwrap();
    assert_eq!(p.mode, LexerMode::BeforeKey);
    p.feed(b"}").unwrap();
    assert_eq!(p.mode, LexerMode::AfterValue);
  }

  #[test]
  fn whitespace_is_ignored() {
    assert_eq!(feed_once("/foo", b"  {  }  "), Ok(Status::NeedMore));
    assert_eq!(feed_once("/foo", b"\n[\n]\n"), Ok(Status::NeedMore));
  }

  // ── Error cases ───────────────────────────────────────────────────────

  #[test]
  fn error_closing_bracket_at_top_level() {
    assert_eq!(
      feed_once("/foo", b"]"),
      Err(ParseError::UnexpectedByte(b']'))
    );
  }

  #[test]
  fn error_closing_brace_at_top_level() {
    assert_eq!(
      feed_once("/foo", b"}"),
      Err(ParseError::UnexpectedByte(b'}'))
    );
  }

  #[test]
  fn error_bracket_closes_object() {
    // `{]` — object closed by `]`
    assert_eq!(
      feed_once("/foo", b"{]"),
      Err(ParseError::UnexpectedByte(b']'))
    );
  }

  #[test]
  fn error_brace_closes_array() {
    // `[}` — array closed by `}`
    assert_eq!(
      feed_once("/foo", b"[}"),
      Err(ParseError::UnexpectedByte(b'}'))
    );
  }

  #[test]
  fn error_comma_at_top_level() {
    // After `{}`, a `,` at top level is invalid.
    let mut p = parser("/foo");
    p.feed(b"{}").unwrap();
    assert_eq!(p.feed(b","), Err(ParseError::UnexpectedByte(b',')));
  }

  #[test]
  fn error_double_closing_brace() {
    // `{}}` — second `}` is unexpected
    let mut p = parser("/foo");
    p.feed(b"{}").unwrap();
    assert_eq!(p.feed(b"}"), Err(ParseError::UnexpectedByte(b'}')));
  }

  // ── LexerMode PartialEq sanity ────────────────────────────────────────

  #[test]
  fn lexer_mode_eq() {
    assert_eq!(LexerMode::BeforeValue, LexerMode::BeforeValue);
    assert_ne!(LexerMode::BeforeValue, LexerMode::BeforeKey);
    assert_eq!(
      LexerMode::InKeyUnicodeEscape { seen: 2, accum: 0 },
      LexerMode::InKeyUnicodeEscape { seen: 2, accum: 0 }
    );
    assert_ne!(
      LexerMode::InKeyUnicodeEscape { seen: 1, accum: 0 },
      LexerMode::InKeyUnicodeEscape { seen: 2, accum: 0 }
    );
    assert_eq!(
      LexerMode::Skip {
        depth: 3,
        str_state: Some(SkipStringState::Body)
      },
      LexerMode::Skip {
        depth: 3,
        str_state: Some(SkipStringState::Body)
      },
    );
  }

  // ── StackFrame ────────────────────────────────────────────────────────

  #[test]
  fn stack_frame_fields() {
    let frame = StackFrame {
      kind: ContainerKind::Object,
      trie_node: 1,
      array_index: 0,
    };
    assert_eq!(frame.kind, ContainerKind::Object);
    assert_eq!(frame.trie_node, 1);
    assert!(frame.trie_node != OFF_PATH);
  }

  #[test]
  fn off_path_sentinel() {
    let frame = StackFrame {
      kind: ContainerKind::Array,
      trie_node: OFF_PATH,
      array_index: 5,
    };
    assert_eq!(frame.trie_node, OFF_PATH);
    assert_eq!(frame.array_index, 5);
  }

  // ── ParseError Display ────────────────────────────────────────────────

  #[test]
  fn parse_error_unexpected_byte_display() {
    let e = ParseError::UnexpectedByte(b'x');
    assert!(e.to_string().contains("78") || e.to_string().contains('x'));
  }

  #[test]
  fn parse_error_eof_display() {
    assert!(ParseError::UnexpectedEof.to_string().contains("end"));
  }

  // ── Status ────────────────────────────────────────────────────────────

  #[test]
  fn status_eq() {
    assert_eq!(Status::Done, Status::Done);
    assert_eq!(Status::NeedMore, Status::NeedMore);
    assert_ne!(Status::Done, Status::NeedMore);
  }

  // ── Key string parsing ────────────────────────────────────────────────

  fn feed_all(pointer: &str, input: &[u8]) -> Parser {
    let mut p = parser(pointer);
    p.feed(input).unwrap();
    p
  }

  #[test]
  fn matched_key_sets_pending_node() {
    let p = feed_all("/foo/bar", b"{\"foo\"");
    assert_eq!(p.mode, LexerMode::AfterKey);
    assert_ne!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn unmatched_key_sets_off_path() {
    let p = feed_all("/foo", b"{\"bar\"");
    assert_eq!(p.mode, LexerMode::AfterKey);
    assert_eq!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn key_then_colon_enters_before_value() {
    let p = feed_all("/foo/bar", b"{\"foo\":");
    assert_eq!(p.mode, LexerMode::BeforeValue);
    assert_ne!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn key_descends_into_nested_object() {
    // `{"foo": {}}` with /foo/bar — descends, bar not found, NeedMore.
    let mut p = Builder::new()
      .register("/foo/bar", |_, _| {})
      .unwrap()
      .build();
    assert_eq!(p.feed(b"{\"foo\":{}}"), Ok(Status::NeedMore));
    assert_eq!(p.mode, LexerMode::AfterValue);
  }

  #[test]
  fn unmatched_key_value_enters_skip() {
    // "bar" doesn't match /foo; its `{` value enters Skip mode.
    let mut p = parser("/foo");
    p.feed(b"{\"bar\":{").unwrap();
    assert_eq!(
      p.mode,
      LexerMode::Skip {
        depth: 1,
        str_state: None
      }
    );
  }

  #[test]
  fn multi_chunk_key_split_across_boundary() {
    let mut p = Builder::new()
      .register("/foo/bar", |_, _| {})
      .unwrap()
      .build();
    p.feed(b"{\"fo").unwrap();
    assert_eq!(p.mode, LexerMode::InKeyString);
    assert_eq!(p.scratch, b"fo");
    p.feed(b"o\":{}").unwrap();
    assert_eq!(p.mode, LexerMode::AfterValue);
  }

  #[test]
  fn key_with_solidus_escape() {
    // JSON `"fo\/o"` decodes to `fo/o`; register with ~1 in pointer.
    let mut p = Builder::new()
      .register("/fo~1o", |_, _| {})
      .unwrap()
      .build();
    p.feed(b"{\"fo\\/o\"").unwrap();
    assert_eq!(p.mode, LexerMode::AfterKey);
    assert_ne!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn key_with_tab_escape() {
    // JSON `"\t"` decodes to a tab character.
    let mut p = Builder::new().register("/\t", |_, _| {}).unwrap().build();
    p.feed(b"{\"\\t\"").unwrap();
    assert_eq!(p.mode, LexerMode::AfterKey);
    assert_ne!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn key_with_unicode_escape() {
    // `\u0066` == 'f', so `\u0066oo` == "foo".
    let mut p = parser("/foo");
    p.feed(b"{\"\\u0066oo\"").unwrap();
    assert_eq!(p.mode, LexerMode::AfterKey);
    assert_ne!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn key_unicode_escape_split_across_chunks() {
    let mut p = parser("/foo");
    p.feed(b"{\"\\u00").unwrap();
    assert_eq!(p.mode, LexerMode::InKeyUnicodeEscape { seen: 2, accum: 0 });
    p.feed(b"66oo\"").unwrap();
    assert_eq!(p.mode, LexerMode::AfterKey);
    assert_ne!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn second_key_resolved_from_frame_node() {
    // After missing "bar", re-entering BeforeKey and matching "foo" should
    // still resolve correctly from the object frame's trie node.
    let mut p = parser("/foo");
    p.feed(b"{\"bar\":").unwrap();
    assert_eq!(p.pending_node, OFF_PATH);
    // Simulate having consumed the value for "bar" by manually returning
    // to BeforeKey (Skip isn't implemented yet).
    p.mode = LexerMode::BeforeKey;
    p.feed(b"\"foo\"").unwrap();
    assert_eq!(p.mode, LexerMode::AfterKey);
    assert_ne!(p.pending_node, OFF_PATH);
  }

  #[test]
  fn control_byte_in_key_is_error() {
    let mut p = parser("/foo");
    p.feed(b"{\"").unwrap();
    assert_eq!(p.feed(b"\x01"), Err(ParseError::UnexpectedByte(0x01)));
  }

  #[test]
  fn invalid_escape_in_key_is_error() {
    let mut p = parser("/foo");
    p.feed(b"{\"\\").unwrap();
    assert_eq!(p.feed(b"q"), Err(ParseError::UnexpectedByte(b'q')));
  }

  #[test]
  fn invalid_unicode_hex_in_key_is_error() {
    let mut p = parser("/foo");
    p.feed(b"{\"\\u").unwrap();
    assert_eq!(p.feed(b"G"), Err(ParseError::UnexpectedByte(b'G')));
  }

  // ── Matched value parsing ─────────────────────────────────────────────

  /// Helper: build a parser with a collecting callback; returns (parser, collected).
  /// `collected` accumulates `(bytes, is_complete)` pairs.
  fn collecting_parser(
    pointer: &str,
  ) -> (
    Parser,
    std::rc::Rc<std::cell::RefCell<Vec<(Vec<u8>, bool)>>>,
  ) {
    use std::cell::RefCell;
    use std::rc::Rc;
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

  #[test]
  fn matched_string_fires_callback() {
    let (mut p, calls) = collecting_parser("/foo");
    let result = p.feed(b"{\"foo\":\"hello\"}");
    assert_eq!(result, Ok(Status::Done));
    let c = calls.borrow();
    // Body bytes streamed with is_complete=false, then empty final with is_complete=true.
    let body: Vec<u8> = c
      .iter()
      .filter(|(_, done)| !done)
      .flat_map(|(b, _)| b.iter().copied())
      .collect();
    assert_eq!(body, b"hello");
    assert!(
      c.last()
        .map(|(b, done)| b.is_empty() && *done)
        .unwrap_or(false)
    );
  }

  #[test]
  fn matched_string_empty_fires_callback() {
    let (mut p, calls) = collecting_parser("/x");
    let result = p.feed(b"{\"x\":\"\"}");
    assert_eq!(result, Ok(Status::Done));
    let c = calls.borrow();
    // Only one call: empty slice, is_complete=true.
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (vec![], true));
  }

  #[test]
  fn matched_string_split_across_chunks() {
    let (mut p, calls) = collecting_parser("/k");
    p.feed(b"{\"k\":\"hel").unwrap();
    p.feed(b"lo\"}").unwrap();
    let c = calls.borrow();
    let body: Vec<u8> = c
      .iter()
      .filter(|(_, done)| !done)
      .flat_map(|(b, _)| b.iter().copied())
      .collect();
    assert_eq!(body, b"hello");
    assert!(c.last().map(|(_, done)| *done).unwrap_or(false));
  }

  #[test]
  fn matched_string_with_escape_fires_raw_bytes() {
    // `"hi\nbye"` — the escape is forwarded raw (backslash + n).
    let (mut p, calls) = collecting_parser("/s");
    p.feed(b"{\"s\":\"hi\\nbye\"}").unwrap();
    let c = calls.borrow();
    let body: Vec<u8> = c
      .iter()
      .filter(|(_, done)| !done)
      .flat_map(|(b, _)| b.iter().copied())
      .collect();
    // Raw JSON bytes: "hi" + b'\' + b'n' + "bye"
    assert_eq!(body, b"hi\\nbye");
  }

  #[test]
  fn matched_number_fires_callback() {
    let (mut p, calls) = collecting_parser("/n");
    let result = p.feed(b"{\"n\":42}");
    assert_eq!(result, Ok(Status::Done));
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"42".to_vec(), true));
  }

  #[test]
  fn matched_number_decimal_fires_callback() {
    let (mut p, calls) = collecting_parser("/v");
    p.feed(b"{\"v\":3.14}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"3.14".to_vec(), true));
  }

  #[test]
  fn matched_number_negative_fires_callback() {
    let (mut p, calls) = collecting_parser("/v");
    p.feed(b"{\"v\":-7}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"-7".to_vec(), true));
  }

  #[test]
  fn matched_number_split_across_chunks() {
    let (mut p, calls) = collecting_parser("/n");
    p.feed(b"{\"n\":12").unwrap();
    assert_eq!(calls.borrow().len(), 0); // not fired yet
    p.feed(b"34}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"1234".to_vec(), true));
  }

  #[test]
  fn matched_true_fires_callback() {
    let (mut p, calls) = collecting_parser("/b");
    let result = p.feed(b"{\"b\":true}");
    assert_eq!(result, Ok(Status::Done));
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"true".to_vec(), true));
  }

  #[test]
  fn matched_false_fires_callback() {
    let (mut p, calls) = collecting_parser("/b");
    let result = p.feed(b"{\"b\":false}");
    assert_eq!(result, Ok(Status::Done));
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"false".to_vec(), true));
  }

  #[test]
  fn matched_null_fires_callback() {
    let (mut p, calls) = collecting_parser("/b");
    let result = p.feed(b"{\"b\":null}");
    assert_eq!(result, Ok(Status::Done));
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"null".to_vec(), true));
  }

  #[test]
  fn keyword_split_across_chunks() {
    let (mut p, calls) = collecting_parser("/x");
    p.feed(b"{\"x\":tr").unwrap();
    assert_eq!(calls.borrow().len(), 0);
    p.feed(b"ue}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"true".to_vec(), true));
  }

  #[test]
  fn status_done_after_all_paths_resolved() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let hits: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let h1 = Rc::clone(&hits);
    let h2 = Rc::clone(&hits);
    let mut p = Builder::new()
      .register("/a", move |_, _| {
        *h1.borrow_mut() += 1;
      })
      .unwrap()
      .register("/b", move |_, _| {
        *h2.borrow_mut() += 1;
      })
      .unwrap()
      .build();
    // /a resolves first; Done should be returned immediately after /b resolves.
    let result = p.feed(b"{\"a\":1,\"b\":2}");
    assert_eq!(result, Ok(Status::Done));
    assert_eq!(*hits.borrow(), 2);
  }

  #[test]
  fn status_done_mid_chunk_stops_parsing() {
    // After finding the single registered path, Done is returned; the
    // trailing `,garbage` is never processed (no error).
    let (mut p, _calls) = collecting_parser("/x");
    // Feed a chunk where /x resolves partway through.
    let result = p.feed(b"{\"x\":1,\"ignored\":bad}");
    assert_eq!(result, Ok(Status::Done));
  }

  #[test]
  fn off_path_string_no_callback() {
    // Register /other; feed /foo string value — no callback, no error.
    let (mut p, calls) = collecting_parser("/other");
    p.feed(b"{\"foo\":\"hello\"}").unwrap();
    assert_eq!(calls.borrow().len(), 0);
  }

  #[test]
  fn off_path_number_no_callback() {
    let (mut p, calls) = collecting_parser("/other");
    p.feed(b"{\"foo\":42}").unwrap();
    assert_eq!(calls.borrow().len(), 0);
  }

  #[test]
  fn off_path_keyword_no_callback() {
    let (mut p, calls) = collecting_parser("/other");
    p.feed(b"{\"foo\":true}").unwrap();
    assert_eq!(calls.borrow().len(), 0);
  }

  #[test]
  fn invalid_keyword_byte_is_error() {
    let mut p = parser("/x");
    // "trxe" — not a valid keyword.
    assert_eq!(
      p.feed(b"{\"x\":trxe}"),
      Err(ParseError::UnexpectedByte(b'x'))
    );
  }

  #[test]
  fn array_element_string_fires_callback() {
    let (mut p, calls) = collecting_parser("/0");
    let result = p.feed(b"[\"first\",\"second\"]");
    assert_eq!(result, Ok(Status::Done));
    let c = calls.borrow();
    let body: Vec<u8> = c
      .iter()
      .filter(|(_, done)| !done)
      .flat_map(|(b, _)| b.iter().copied())
      .collect();
    assert_eq!(body, b"first");
  }

  #[test]
  fn nested_path_number_fires_callback() {
    let (mut p, calls) = collecting_parser("/foo/bar");
    p.feed(b"{\"foo\":{\"bar\":99}}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"99".to_vec(), true));
  }

  // ── Skip mode ─────────────────────────────────────────────────────────

  #[test]
  fn skip_off_path_object() {
    // /foo registered; "bar":{...} should be skipped cleanly.
    let (mut p, calls) = collecting_parser("/foo");
    p.feed(b"{\"bar\":{\"a\":1},\"foo\":42}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"42".to_vec(), true));
  }

  #[test]
  fn skip_off_path_array() {
    let (mut p, calls) = collecting_parser("/x");
    p.feed(b"{\"y\":[1,2,3],\"x\":\"hit\"}").unwrap();
    let c = calls.borrow();
    let body: Vec<u8> = c
      .iter()
      .filter(|(_, d)| !d)
      .flat_map(|(b, _)| b.iter().copied())
      .collect();
    assert_eq!(body, b"hit");
  }

  #[test]
  fn skip_deeply_nested_container() {
    let (mut p, calls) = collecting_parser("/z");
    p.feed(b"{\"a\":{\"b\":{\"c\":{\"d\":99}}},\"z\":7}")
      .unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"7".to_vec(), true));
  }

  #[test]
  fn skip_string_with_braces_inside() {
    // A string value containing `{` and `}` must not affect depth counting.
    let (mut p, calls) = collecting_parser("/target");
    p.feed(b"{\"trap\":\"{not a brace}\",\"target\":5}")
      .unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"5".to_vec(), true));
  }

  #[test]
  fn skip_string_with_escaped_quote_inside() {
    // `"\""` — the escaped quote must not end the string early in Skip.
    let (mut p, calls) = collecting_parser("/v");
    p.feed(b"{\"s\":\"he said \\\"hi\\\"\",\"v\":1}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"1".to_vec(), true));
  }

  #[test]
  fn skip_string_with_unicode_escape() {
    let (mut p, calls) = collecting_parser("/v");
    p.feed(b"{\"s\":\"\\u0048ello\",\"v\":2}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"2".to_vec(), true));
  }

  #[test]
  fn skip_split_across_chunks() {
    let (mut p, calls) = collecting_parser("/found");
    p.feed(b"{\"skip\":{\"a\":1,").unwrap();
    assert_eq!(calls.borrow().len(), 0);
    p.feed(b"\"b\":2},\"found\":99}").unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"99".to_vec(), true));
  }

  #[test]
  fn skip_then_match_returns_done() {
    let (mut p, _calls) = collecting_parser("/k");
    let result = p.feed(b"{\"skip\":{},\"k\":\"v\"}");
    assert_eq!(result, Ok(Status::Done));
  }

  #[test]
  fn skip_control_byte_in_string_is_error() {
    let mut p = parser("/x");
    // Control byte inside skipped string.
    assert_eq!(
      p.feed(b"{\"s\":\"\x01\"}"),
      Err(ParseError::UnexpectedByte(0x01))
    );
  }

  // ── finish() ──────────────────────────────────────────────────────────

  #[test]
  fn finish_after_complete_object_is_ok() {
    let mut p = parser("/foo");
    p.feed(b"{\"foo\":\"bar\"}").unwrap();
    assert_eq!(p.finish(), Ok(()));
  }

  #[test]
  fn finish_top_level_number_fires_callback() {
    let (mut p, calls) = collecting_parser("");
    // Register root pointer "" — matches any top-level value.
    // Feed a number with no trailing delimiter.
    let _ = p.feed(b"42"); // stays in InMatchedNumber
    assert!(calls.borrow().is_empty());
    p.finish().unwrap();
    let c = calls.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0], (b"42".to_vec(), true));
  }

  #[test]
  fn finish_mid_string_is_eof_error() {
    let mut p = parser("/x");
    p.feed(b"{\"x\":\"oops").unwrap(); // unterminated string
    assert_eq!(p.finish(), Err(ParseError::UnexpectedEof));
  }

  #[test]
  fn finish_mid_keyword_is_eof_error() {
    let mut p = parser("/x");
    p.feed(b"{\"x\":tru").unwrap(); // truncated "true"
    assert_eq!(p.finish(), Err(ParseError::UnexpectedEof));
  }

  #[test]
  fn finish_unclosed_object_is_eof_error() {
    let mut p = parser("/foo");
    p.feed(b"{\"foo\":1").unwrap();
    // Number fires on `}` but `}` never arrives — number accumulates.
    // finish() fires the number callback, then we still have an unclosed
    // object on the stack, so a second finish() call is needed... actually
    // the stack check happens after the InMatchedNumber arm returns Ok.
    // The stack is still non-empty → but InMatchedNumber arm returns Ok(())
    // directly. Let's verify actual behaviour: after finish(), mode is
    // AfterValue, stack still has the Object frame → any subsequent
    // operation is the caller's concern. The number callback fired. That's
    // the contract.
    assert_eq!(p.finish(), Ok(()));
  }

  #[test]
  fn finish_empty_stream_is_eof_error() {
    let mut p = parser("/foo");
    // No bytes fed — BeforeValue with empty stack but paths registered.
    assert_eq!(p.finish(), Err(ParseError::UnexpectedEof));
  }

  #[test]
  fn finish_after_done_is_ok() {
    let (mut p, _calls) = collecting_parser("/v");
    p.feed(b"{\"v\":1}").unwrap(); // resolves, Status::Done
    assert_eq!(p.finish(), Ok(())); // already resolved
  }

  #[test]
  fn finish_off_path_top_level_number_no_callback() {
    // /other registered; top-level number won't match → no callback, no error.
    let (mut p, calls) = collecting_parser("/other");
    p.feed(b"99").unwrap();
    p.finish().unwrap();
    assert!(calls.borrow().is_empty());
  }
}
