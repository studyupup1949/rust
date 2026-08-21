use std::collections::HashMap;

pub enum Strategy {
    Collapse,
    Suffix,
    Truncate(usize),
}

/// Shortens a set of segment sequences according to the chosen strategy.
/// Returns one shortened string per input.
pub fn shorten(segments: &[Vec<&str>], sep: &str, ellipsis: &str, strategy: &Strategy) -> Vec<String> {
    if segments.is_empty() {
        return vec![];
    }
    if segments.len() == 1 {
        return vec![segments[0].join(sep)];
    }

    // Strip trailing empty segments (from trailing separators like /foo/bar/)
    // before shortening, then re-append them.
    let trailing: Vec<usize> = segments
        .iter()
        .map(|s| {
            let n = s.iter().rev().take_while(|seg| seg.is_empty()).count();
            // Don't strip everything — keep at least 1 segment
            if n >= s.len() { 0 } else { n }
        })
        .collect();

    let stripped: Vec<Vec<&str>> = segments
        .iter()
        .zip(trailing.iter())
        .map(|(s, &t)| if t > 0 { s[..s.len() - t].to_vec() } else { s.clone() })
        .collect();

    let mut results = match strategy {
        Strategy::Collapse => collapse(&stripped, sep, ellipsis),
        Strategy::Suffix => suffix(&stripped, sep, ellipsis),
        Strategy::Truncate(n) => truncate(&stripped, sep, *n),
    };

    // Re-append trailing separators
    for (r, &t) in results.iter_mut().zip(trailing.iter()) {
        for _ in 0..t {
            r.push_str(sep);
        }
    }

    results
}

// --- Collapse: trie-based ---
// Build a trie from all segment sequences. For each line, a segment is
// "needed" if the trie node it sits under has multiple children (branching).
// The last segment is always kept. Empty segments (leading separator) are
// always kept. Runs of 1 collapsed segment are kept for readability —
// only runs of 2+ get replaced with ellipsis.

struct TrieNode {
    children: HashMap<String, TrieNode>,
}

impl TrieNode {
    fn new() -> Self {
        Self { children: HashMap::new() }
    }

    fn insert(&mut self, segments: &[&str]) {
        if let Some((first, rest)) = segments.split_first() {
            self.children
                .entry(first.to_string())
                .or_insert_with(TrieNode::new)
                .insert(rest);
        }
    }
}

fn collapse(segments: &[Vec<&str>], sep: &str, ellipsis: &str) -> Vec<String> {
    let mut root = TrieNode::new();
    for segs in segments {
        root.insert(segs);
    }

    // If trie has no branching at all, all lines are identical → nothing to collapse
    if !has_branching(&root) {
        return segments.iter().map(|segs| segs.join(sep)).collect();
    }

    segments
        .iter()
        .map(|segs| collapse_one(segs, &root, sep, ellipsis))
        .collect()
}

fn has_branching(node: &TrieNode) -> bool {
    if node.children.len() > 1 {
        return true;
    }
    node.children.values().any(|child| has_branching(child))
}

fn collapse_one(segs: &[&str], root: &TrieNode, sep: &str, ellipsis: &str) -> String {
    if segs.is_empty() {
        return String::new();
    }

    // Walk the trie and decide keep/collapse per segment.
    // Keep if: branching (parent has >1 children), last segment, or empty (structural).
    let mut kept: Vec<Option<&str>> = Vec::with_capacity(segs.len());
    let mut node = root;

    for (i, seg) in segs.iter().enumerate() {
        let is_last = i == segs.len() - 1;
        let branching = node.children.len() > 1;

        if branching || is_last || seg.is_empty() {
            kept.push(Some(seg));
        } else {
            kept.push(None);
        }

        if let Some(child) = node.children.get(*seg) {
            node = child;
        } else {
            break;
        }
    }

    // Build output: replace runs of collapsed (None) segments with ellipsis.
    // But keep single-segment runs as-is (not worth replacing 1 segment with …).
    let mut parts: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < kept.len() {
        if let Some(s) = kept[i] {
            parts.push(s);
            i += 1;
        } else {
            let start = i;
            while i < kept.len() && kept[i].is_none() {
                i += 1;
            }
            if i - start == 1 {
                // Single collapsed segment — keep original for readability
                parts.push(segs[start]);
            } else {
                parts.push(ellipsis);
            }
        }
    }

    parts.join(sep)
}

// --- Suffix: per-line shortest unique suffix ---

fn suffix(segments: &[Vec<&str>], sep: &str, ellipsis: &str) -> Vec<String> {
    segments
        .iter()
        .map(|segs| {
            let needed = suffix_len(segs, segments);
            if needed >= segs.len() {
                segs.join(sep)
            } else {
                let tail = &segs[segs.len() - needed..];
                // Preserve leading separator
                if segs.first() == Some(&"") {
                    format!("{sep}{ellipsis}{sep}{}", tail.join(sep))
                } else {
                    format!("{ellipsis}{sep}{}", tail.join(sep))
                }
            }
        })
        .collect()
}

/// Find minimum N such that the last N segments of `target` are unique among all `lines`.
fn suffix_len(target: &[&str], all: &[Vec<&str>]) -> usize {
    for n in 1..=target.len() {
        let tail: &[&str] = &target[target.len() - n..];
        let count = all.iter().filter(|s| s.ends_with(tail)).count();
        if count <= 1 {
            return n;
        }
    }
    target.len()
}

// --- Truncate: shorten common prefix segments to N chars ---

fn truncate(segments: &[Vec<&str>], sep: &str, n: usize) -> Vec<String> {
    let prefix_len = common_prefix_len(segments);

    segments
        .iter()
        .map(|segs| {
            let mut parts: Vec<String> = Vec::with_capacity(segs.len());
            for (i, seg) in segs.iter().enumerate() {
                if i < prefix_len && !seg.is_empty() {
                    let truncated: String = seg.chars().take(n).collect();
                    parts.push(truncated);
                } else {
                    parts.push(seg.to_string());
                }
            }
            parts.join(sep)
        })
        .collect()
}

fn common_prefix_len(segments: &[Vec<&str>]) -> usize {
    if segments.is_empty() {
        return 0;
    }
    let first = &segments[0];
    let min_len = segments.iter().map(|s| s.len()).min().unwrap_or(0);

    for i in 0..min_len {
        if !segments.iter().all(|s| s[i] == first[i]) {
            return i;
        }
    }
    min_len
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(s: &str) -> Vec<&str> {
        s.split('/').collect()
    }

    #[test]
    fn test_collapse_paths() {
        let lines = [
            "/home/user/proj/foo/src/main.rs",
            "/home/user/proj/bar/src/main.rs",
            "/home/user/proj/bar/src/lib.rs",
        ];
        let segs: Vec<Vec<&str>> = lines.iter().map(|l| split(l)).collect();
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        // home/user/proj = 3 non-branching segments → collapsed
        // foo vs bar = branching → kept
        // src = single non-branching → kept (run of 1)
        assert_eq!(result[0], "/…/foo/src/main.rs");
        assert_eq!(result[1], "/…/bar/src/main.rs");
        assert_eq!(result[2], "/…/bar/src/lib.rs");
    }

    #[test]
    fn test_collapse_no_common() {
        let segs = vec![split("foo/bar"), split("baz/qux")];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        assert_eq!(result[0], "foo/bar");
        assert_eq!(result[1], "baz/qux");
    }

    #[test]
    fn test_collapse_mid_path() {
        let segs: Vec<Vec<&str>> = vec![
            split("/org/frontend/issues/42"),
            split("/org/frontend/pulls/15"),
            split("/org/backend/actions/runs/99"),
            split("/team/infra/merge_requests/7"),
        ];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        // All segments at branching points → kept
        // actions/runs (2 non-branching) → collapsed
        // infra/merge_requests (2 non-branching) → collapsed
        assert_eq!(result[0], "/org/frontend/issues/42");
        assert_eq!(result[1], "/org/frontend/pulls/15");
        assert_eq!(result[2], "/org/backend/…/99");
        assert_eq!(result[3], "/team/…/7");
    }

    #[test]
    fn test_suffix_basic() {
        let lines = [
            "/home/user/proj/foo/src/main.rs",
            "/home/user/proj/bar/src/main.rs",
            "/home/user/proj/bar/src/lib.rs",
        ];
        let segs: Vec<Vec<&str>> = lines.iter().map(|l| split(l)).collect();
        let result = shorten(&segs, "/", "…", &Strategy::Suffix);
        // main.rs collides → need src/main.rs → still collides → foo/src/main.rs unique
        assert_eq!(result[0], "/…/foo/src/main.rs");
        assert_eq!(result[1], "/…/bar/src/main.rs");
        // lib.rs is unique at 1 segment
        assert_eq!(result[2], "/…/lib.rs");
    }

    #[test]
    fn test_truncate_basic() {
        let lines = [
            "/home/user/proj/foo/src/main.rs",
            "/home/user/proj/bar/src/main.rs",
        ];
        let segs: Vec<Vec<&str>> = lines.iter().map(|l| split(l)).collect();
        let result = shorten(&segs, "/", "…", &Strategy::Truncate(1));
        assert_eq!(result[0], "/h/u/p/foo/src/main.rs");
        assert_eq!(result[1], "/h/u/p/bar/src/main.rs");
    }

    #[test]
    fn test_single_line() {
        let segs = vec![split("foo/bar/baz")];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        assert_eq!(result[0], "foo/bar/baz");
    }

    #[test]
    fn test_dot_separator() {
        let segs: Vec<Vec<&str>> = vec![
            "com.example.foo.Bar".split('.').collect(),
            "com.example.bar.Baz".split('.').collect(),
        ];
        let result = shorten(&segs, ".", "…", &Strategy::Collapse);
        assert_eq!(result[0], "….foo.Bar");
        assert_eq!(result[1], "….bar.Baz");
    }

    #[test]
    fn test_empty_input() {
        let segs: Vec<Vec<&str>> = vec![];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        assert!(result.is_empty());
    }

    #[test]
    fn test_identical_lines() {
        let segs = vec![split("foo/bar/baz"), split("foo/bar/baz")];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        assert_eq!(result[0], "foo/bar/baz");
        assert_eq!(result[1], "foo/bar/baz");
    }

    #[test]
    fn test_collapse_two_segments_only() {
        let segs = vec![split("a/x"), split("a/y")];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        // 'a' is single non-branching → kept (run of 1)
        assert_eq!(result[0], "a/x");
        assert_eq!(result[1], "a/y");
    }

    #[test]
    fn test_collapse_deep_shared_prefix() {
        let segs = vec![
            split("a/b/c/d/e/x"),
            split("a/b/c/d/e/y"),
        ];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        assert_eq!(result[0], "…/x");
        assert_eq!(result[1], "…/y");
    }

    #[test]
    fn test_collapse_preserves_leading_separator() {
        let segs = vec![split("/a/b/x"), split("/a/b/y")];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        assert_eq!(result[0], "/…/x");
        assert_eq!(result[1], "/…/y");
    }

    #[test]
    fn test_suffix_identical_lines() {
        let segs = vec![split("a/b/c"), split("a/b/c")];
        let result = shorten(&segs, "/", "…", &Strategy::Suffix);
        // Can't disambiguate — keep full
        assert_eq!(result[0], "a/b/c");
        assert_eq!(result[1], "a/b/c");
    }

    #[test]
    fn test_suffix_single_line() {
        let segs = vec![split("a/b/c")];
        let result = shorten(&segs, "/", "…", &Strategy::Suffix);
        assert_eq!(result[0], "a/b/c");
    }

    #[test]
    fn test_suffix_unique_at_different_depths() {
        let segs = vec![
            split("a/b/c/file.txt"),   // needs c/file.txt to be unique
            split("a/b/d/file.txt"),   // needs d/file.txt
            split("a/b/d/other.txt"),  // other.txt is unique at depth 1
        ];
        let result = shorten(&segs, "/", "…", &Strategy::Suffix);
        assert_eq!(result[0], "…/c/file.txt");
        assert_eq!(result[1], "…/d/file.txt");
        assert_eq!(result[2], "…/other.txt");
    }

    #[test]
    fn test_truncate_no_common_prefix() {
        let segs = vec![split("foo/bar"), split("baz/qux")];
        let result = shorten(&segs, "/", "…", &Strategy::Truncate(1));
        // No common prefix → nothing truncated
        assert_eq!(result[0], "foo/bar");
        assert_eq!(result[1], "baz/qux");
    }

    #[test]
    fn test_truncate_n3() {
        let segs = vec![
            split("something/long/prefix/x"),
            split("something/long/prefix/y"),
        ];
        let result = shorten(&segs, "/", "…", &Strategy::Truncate(3));
        assert_eq!(result[0], "som/lon/pre/x");
        assert_eq!(result[1], "som/lon/pre/y");
    }

    #[test]
    fn test_truncate_preserves_leading_separator() {
        let segs = vec![split("/home/user/x"), split("/home/user/y")];
        let result = shorten(&segs, "/", "…", &Strategy::Truncate(1));
        assert_eq!(result[0], "/h/u/x");
        assert_eq!(result[1], "/h/u/y");
    }

    #[test]
    fn test_collapse_custom_ellipsis() {
        let segs = vec![
            split("a/b/c/d/x"),
            split("a/b/c/d/y"),
        ];
        let result = shorten(&segs, "/", "..", &Strategy::Collapse);
        assert_eq!(result[0], "../x");
        assert_eq!(result[1], "../y");
    }

    #[test]
    fn test_collapse_trailing_separator() {
        let segs = vec![
            split("/a/b/c/x/"),
            split("/a/b/c/y/"),
        ];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        assert_eq!(result[0], "/…/x/");
        assert_eq!(result[1], "/…/y/");
    }

    #[test]
    fn test_collapse_many_branches() {
        let segs = vec![
            split("root/a/leaf1"),
            split("root/b/leaf2"),
            split("root/c/leaf3"),
        ];
        let result = shorten(&segs, "/", "…", &Strategy::Collapse);
        // root is single non-branching (run of 1) → kept
        assert_eq!(result[0], "root/a/leaf1");
        assert_eq!(result[1], "root/b/leaf2");
        assert_eq!(result[2], "root/c/leaf3");
    }
}
