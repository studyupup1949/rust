//! JSON Pointer helpers for pointing into an Adaptive Card.

/// Build a JSON Pointer from a list of segments.
/// Segments are escaped per RFC 6901 (`~` → `~0`, `/` → `~1`).
#[must_use]
pub fn build_pointer(segments: &[&str]) -> String {
    if segments.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for seg in segments {
        out.push('/');
        for c in seg.chars() {
            match c {
                '~' => out.push_str("~0"),
                '/' => out.push_str("~1"),
                _ => out.push(c),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_segments_yields_empty_pointer() {
        assert_eq!(build_pointer(&[]), "");
    }

    #[test]
    fn simple_path() {
        assert_eq!(
            build_pointer(&["body", "0", "items", "2"]),
            "/body/0/items/2"
        );
    }

    #[test]
    fn escape_tilde_and_slash() {
        assert_eq!(build_pointer(&["a~b"]), "/a~0b");
        assert_eq!(build_pointer(&["a/b"]), "/a~1b");
    }
}
