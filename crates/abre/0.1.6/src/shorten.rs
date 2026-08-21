use std::collections::HashMap;

pub enum Strategy {
    Collapse,
    Suffix,
    Truncate(usize),
}

/// Split a string by any character in `seps`.
/// Returns (segments, inter_seps) where inter_seps.len() == segments.len() - 1.
pub fn split_multi<'a>(s: &'a str, seps: &[char]) -> (Vec<&'a str>, Vec<char>) {
    if seps.is_empty() {
        return (vec![s], vec![]);
    }
    let mut segments = Vec::new();
    let mut inter_seps = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if seps.contains(&c) {
            segments.push(&s[start..i]);
            inter_seps.push(c);
            start = i + c.len_utf8();
        }
    }
    segments.push(&s[start..]);
    (segments, inter_seps)
}

fn join_tracked(segs: &[&str], seps: &[char]) -> String {
    let mut result = String::new();
    for (i, seg) in segs.iter().enumerate() {
        if i > 0 {
            result.push(seps[i - 1]);
        }
        result.push_str(seg);
    }
    result
}

/// Shortens a set of lines according to the chosen strategy.
/// Each line is (segments, inter_seps) from split_multi — seps tracks which
/// separator was used between each pair of adjacent segments.
pub fn shorten(lines: &[(Vec<&str>, Vec<char>)], ellipsis: &str, strategy: &Strategy) -> Vec<String> {
    if lines.is_empty() {
        return vec![];
    }
    if lines.len() == 1 {
        let (segs, seps) = &lines[0];
        return vec![join_tracked(segs, seps)];
    }

    // Strip trailing empty segments (from trailing separators like /foo/bar/)
    // before shortening, then re-append them.
    let trailing: Vec<usize> = lines
        .iter()
        .map(|(segs, _)| {
            let n = segs.iter().rev().take_while(|s| s.is_empty()).count();
            if n >= segs.len() { 0 } else { n }
        })
        .collect();

    let trailing_seps: Vec<Vec<char>> = lines
        .iter()
        .zip(&trailing)
        .map(|((_, seps), &t)| {
            if t > 0 { seps[seps.len() - t..].to_vec() } else { vec![] }
        })
        .collect();

    let stripped: Vec<(Vec<&str>, Vec<char>)> = lines
        .iter()
        .zip(&trailing)
        .map(|((segs, seps), &t)| {
            if t > 0 {
                (segs[..segs.len() - t].to_vec(), seps[..seps.len() - t].to_vec())
            } else {
                (segs.clone(), seps.clone())
            }
        })
        .collect();

    let mut results = match strategy {
        Strategy::Collapse => collapse(&stripped, ellipsis),
        Strategy::Suffix => suffix_strategy(&stripped, ellipsis),
        Strategy::Truncate(n) => truncate(&stripped, *n),
    };

    for (r, ts) in results.iter_mut().zip(&trailing_seps) {
        for &c in ts {
            r.push(c);
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

fn collapse(lines: &[(Vec<&str>, Vec<char>)], ellipsis: &str) -> Vec<String> {
    let mut root = TrieNode::new();
    for (segs, _) in lines {
        root.insert(segs);
    }

    if !has_branching(&root) {
        return lines.iter().map(|(segs, seps)| join_tracked(segs, seps)).collect();
    }

    lines.iter().map(|(segs, seps)| collapse_one(segs, seps, &root, ellipsis)).collect()
}

fn has_branching(node: &TrieNode) -> bool {
    if node.children.len() > 1 {
        return true;
    }
    node.children.values().any(|child| has_branching(child))
}

fn collapse_one(segs: &[&str], seps: &[char], root: &TrieNode, ellipsis: &str) -> String {
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

    // Build output using tracked seps.
    // Runs of 2+ collapsed (None) → ellipsis; runs of 1 → keep original.
    let mut result = String::new();
    let mut first_output = true;
    let mut i = 0;

    while i < kept.len() {
        if let Some(s) = kept[i] {
            if !first_output && i > 0 {
                result.push(seps[i - 1]);
            }
            result.push_str(s);
            first_output = false;
            i += 1;
        } else {
            let start = i;
            while i < kept.len() && kept[i].is_none() {
                i += 1;
            }
            if !first_output && start > 0 {
                result.push(seps[start - 1]);
            }
            if i - start == 1 {
                result.push_str(segs[start]);
            } else {
                result.push_str(ellipsis);
            }
            first_output = false;
        }
    }

    result
}

// --- Suffix: per-line shortest unique suffix ---

fn suffix_strategy(lines: &[(Vec<&str>, Vec<char>)], ellipsis: &str) -> Vec<String> {
    let all_segs: Vec<&[&str]> = lines.iter().map(|(s, _)| s.as_slice()).collect();

    lines
        .iter()
        .map(|(segs, seps)| {
            let needed = suffix_len(segs, &all_segs);
            if needed >= segs.len() {
                return join_tracked(segs, seps);
            }
            let k = segs.len() - needed;

            // Build tail using original seps between tail segments
            let mut tail = String::new();
            for i in k..segs.len() {
                if i > k {
                    tail.push(seps[i - 1]);
                }
                tail.push_str(segs[i]);
            }

            let ellipsis_sep = seps[k - 1]; // sep that originally preceded the first tail segment

            if segs.first() == Some(&"") {
                // Has leading separator: "/<leading_sep>…<ellipsis_sep><tail>"
                format!("{}{ellipsis}{}{tail}", seps[0], ellipsis_sep)
            } else {
                format!("{ellipsis}{}{tail}", ellipsis_sep)
            }
        })
        .collect()
}

fn suffix_len(target: &[&str], all: &[&[&str]]) -> usize {
    for n in 1..=target.len() {
        let tail = &target[target.len() - n..];
        let count = all.iter().filter(|s| s.ends_with(tail)).count();
        if count <= 1 {
            return n;
        }
    }
    target.len()
}

// --- Truncate: shorten common prefix segments to N chars ---

fn truncate(lines: &[(Vec<&str>, Vec<char>)], n: usize) -> Vec<String> {
    let all_segs: Vec<&[&str]> = lines.iter().map(|(s, _)| s.as_slice()).collect();
    let prefix_len = common_prefix_len(&all_segs);

    lines
        .iter()
        .map(|(segs, seps)| {
            let mut result = String::new();
            for (i, seg) in segs.iter().enumerate() {
                if i > 0 {
                    result.push(seps[i - 1]);
                }
                if i < prefix_len && !seg.is_empty() {
                    let truncated: String = seg.chars().take(n).collect();
                    result.push_str(&truncated);
                } else {
                    result.push_str(seg);
                }
            }
            result
        })
        .collect()
}

fn common_prefix_len(segments: &[&[&str]]) -> usize {
    if segments.is_empty() {
        return 0;
    }
    let first = segments[0];
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

    fn lns<'a>(strs: &[&'a str]) -> Vec<(Vec<&'a str>, Vec<char>)> {
        strs.iter().map(|s| split_multi(s, &['/'])).collect()
    }

    #[test]
    fn test_split_multi_single_sep() {
        let (segs, seps) = split_multi("/foo/bar", &['/']);
        assert_eq!(segs, ["", "foo", "bar"]);
        assert_eq!(seps, ['/', '/']);
    }

    #[test]
    fn test_split_multi_mixed_seps() {
        let (segs, seps) = split_multi("foo/bar-baz", &['/', '-']);
        assert_eq!(segs, ["foo", "bar", "baz"]);
        assert_eq!(seps, ['/', '-']);
    }

    #[test]
    fn test_collapse_paths() {
        let result = shorten(&lns(&[
            "/home/user/proj/foo/src/main.rs",
            "/home/user/proj/bar/src/main.rs",
            "/home/user/proj/bar/src/lib.rs",
        ]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "/…/foo/src/main.rs");
        assert_eq!(result[1], "/…/bar/src/main.rs");
        assert_eq!(result[2], "/…/bar/src/lib.rs");
    }

    #[test]
    fn test_collapse_no_common() {
        let result = shorten(&lns(&["foo/bar", "baz/qux"]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "foo/bar");
        assert_eq!(result[1], "baz/qux");
    }

    #[test]
    fn test_collapse_mid_path() {
        let result = shorten(&lns(&[
            "/org/frontend/issues/42",
            "/org/frontend/pulls/15",
            "/org/backend/actions/runs/99",
            "/team/infra/merge_requests/7",
        ]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "/org/frontend/issues/42");
        assert_eq!(result[1], "/org/frontend/pulls/15");
        assert_eq!(result[2], "/org/backend/…/99");
        assert_eq!(result[3], "/team/…/7");
    }

    #[test]
    fn test_suffix_basic() {
        let result = shorten(&lns(&[
            "/home/user/proj/foo/src/main.rs",
            "/home/user/proj/bar/src/main.rs",
            "/home/user/proj/bar/src/lib.rs",
        ]), "…", &Strategy::Suffix);
        assert_eq!(result[0], "/…/foo/src/main.rs");
        assert_eq!(result[1], "/…/bar/src/main.rs");
        assert_eq!(result[2], "/…/lib.rs");
    }

    #[test]
    fn test_truncate_basic() {
        let result = shorten(&lns(&[
            "/home/user/proj/foo/src/main.rs",
            "/home/user/proj/bar/src/main.rs",
        ]), "…", &Strategy::Truncate(1));
        assert_eq!(result[0], "/h/u/p/foo/src/main.rs");
        assert_eq!(result[1], "/h/u/p/bar/src/main.rs");
    }

    #[test]
    fn test_single_line() {
        let result = shorten(&lns(&["foo/bar/baz"]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "foo/bar/baz");
    }

    #[test]
    fn test_dot_separator() {
        let lines: Vec<(Vec<&str>, Vec<char>)> = ["com.example.foo.Bar", "com.example.bar.Baz"]
            .iter()
            .map(|s| split_multi(s, &['.' ]))
            .collect();
        let result = shorten(&lines, "…", &Strategy::Collapse);
        assert_eq!(result[0], "….foo.Bar");
        assert_eq!(result[1], "….bar.Baz");
    }

    #[test]
    fn test_empty_input() {
        let result = shorten(&[], "…", &Strategy::Collapse);
        assert!(result.is_empty());
    }

    #[test]
    fn test_identical_lines() {
        let result = shorten(&lns(&["foo/bar/baz", "foo/bar/baz"]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "foo/bar/baz");
        assert_eq!(result[1], "foo/bar/baz");
    }

    #[test]
    fn test_collapse_two_segments_only() {
        let result = shorten(&lns(&["a/x", "a/y"]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "a/x");
        assert_eq!(result[1], "a/y");
    }

    #[test]
    fn test_collapse_deep_shared_prefix() {
        let result = shorten(&lns(&["a/b/c/d/e/x", "a/b/c/d/e/y"]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "…/x");
        assert_eq!(result[1], "…/y");
    }

    #[test]
    fn test_collapse_preserves_leading_separator() {
        let result = shorten(&lns(&["/a/b/x", "/a/b/y"]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "/…/x");
        assert_eq!(result[1], "/…/y");
    }

    #[test]
    fn test_suffix_identical_lines() {
        let result = shorten(&lns(&["a/b/c", "a/b/c"]), "…", &Strategy::Suffix);
        assert_eq!(result[0], "a/b/c");
        assert_eq!(result[1], "a/b/c");
    }

    #[test]
    fn test_suffix_single_line() {
        let result = shorten(&lns(&["a/b/c"]), "…", &Strategy::Suffix);
        assert_eq!(result[0], "a/b/c");
    }

    #[test]
    fn test_suffix_unique_at_different_depths() {
        let result = shorten(&lns(&[
            "a/b/c/file.txt",
            "a/b/d/file.txt",
            "a/b/d/other.txt",
        ]), "…", &Strategy::Suffix);
        assert_eq!(result[0], "…/c/file.txt");
        assert_eq!(result[1], "…/d/file.txt");
        assert_eq!(result[2], "…/other.txt");
    }

    #[test]
    fn test_truncate_no_common_prefix() {
        let result = shorten(&lns(&["foo/bar", "baz/qux"]), "…", &Strategy::Truncate(1));
        assert_eq!(result[0], "foo/bar");
        assert_eq!(result[1], "baz/qux");
    }

    #[test]
    fn test_truncate_n3() {
        let result = shorten(&lns(&[
            "something/long/prefix/x",
            "something/long/prefix/y",
        ]), "…", &Strategy::Truncate(3));
        assert_eq!(result[0], "som/lon/pre/x");
        assert_eq!(result[1], "som/lon/pre/y");
    }

    #[test]
    fn test_truncate_preserves_leading_separator() {
        let result = shorten(&lns(&["/home/user/x", "/home/user/y"]), "…", &Strategy::Truncate(1));
        assert_eq!(result[0], "/h/u/x");
        assert_eq!(result[1], "/h/u/y");
    }

    #[test]
    fn test_collapse_custom_ellipsis() {
        let result = shorten(&lns(&["a/b/c/d/x", "a/b/c/d/y"]), "..", &Strategy::Collapse);
        assert_eq!(result[0], "../x");
        assert_eq!(result[1], "../y");
    }

    #[test]
    fn test_collapse_trailing_separator() {
        let result = shorten(&lns(&["/a/b/c/x/", "/a/b/c/y/"]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "/…/x/");
        assert_eq!(result[1], "/…/y/");
    }

    #[test]
    fn test_collapse_many_branches() {
        let result = shorten(&lns(&[
            "root/a/leaf1",
            "root/b/leaf2",
            "root/c/leaf3",
        ]), "…", &Strategy::Collapse);
        assert_eq!(result[0], "root/a/leaf1");
        assert_eq!(result[1], "root/b/leaf2");
        assert_eq!(result[2], "root/c/leaf3");
    }

    #[test]
    fn test_multi_sep_collapse() {
        // "foo/bar-baz/x" and "foo/bar-qux/y" with / and - as seps
        let lines: Vec<(Vec<&str>, Vec<char>)> = [
            "foo/bar-baz/x",
            "foo/bar-qux/y",
        ]
        .iter()
        .map(|s| split_multi(s, &['/', '-']))
        .collect();
        let result = shorten(&lines, "…", &Strategy::Collapse);
        // foo and bar are non-branching (run of 2 → collapse), baz/qux branch
        assert_eq!(result[0], "…-baz/x");
        assert_eq!(result[1], "…-qux/y");
    }

    #[test]
    fn test_multi_sep_truncate() {
        let lines: Vec<(Vec<&str>, Vec<char>)> = [
            "release/my-app_v1/lib.so",
            "release/my-app_v2/lib.so",
        ]
        .iter()
        .map(|s| split_multi(s, &['/', '-', '_']))
        .collect();
        let result = shorten(&lines, "…", &Strategy::Truncate(1));
        // common prefix: release, my, app (3 segs), unique: v1/v2
        // original seps between segments are preserved in output
        assert_eq!(result[0], "r/m-a_v1/lib.so");
        assert_eq!(result[1], "r/m-a_v2/lib.so");
    }
}
