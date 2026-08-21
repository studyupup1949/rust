use super::{RsRule, Rule};

pub struct NoVagueComment;

impl Rule for NoVagueComment {
    fn name(&self) -> &'static str {
        "no-vague-comment"
    }

    fn description(&self) -> &'static str {
        "comment blocks must attach to the next syntax item with no blank line in between"
    }
}

/// A contiguous block of comment lines (start..end are 0-based indices).
struct CommentBlock {
    start: usize,
    end: usize,
    is_inner_doc: bool,
}

fn is_comment_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//")
}

fn is_inner_doc_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//!")
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// Find all contiguous comment blocks in the source.
fn find_comment_blocks(lines: &[&str]) -> Vec<CommentBlock> {
    let mut blocks = vec![];
    let mut i = 0;
    while i < lines.len() {
        if is_comment_line(lines[i]) {
            let start = i;
            let mut all_inner_doc = true;
            while i < lines.len() && is_comment_line(lines[i]) {
                if !is_inner_doc_comment(lines[i]) {
                    all_inner_doc = false;
                }
                i += 1;
            }
            blocks.push(CommentBlock {
                start,
                end: i,
                is_inner_doc: all_inner_doc,
            });
        } else {
            i += 1;
        }
    }
    blocks
}

/// Check if a comment block at the top of the file is a file-level inner doc block.
fn is_top_level_doc_block(block: &CommentBlock, lines: &[&str]) -> bool {
    if !block.is_inner_doc {
        return false;
    }
    // All lines before the block must be blank or shebang.
    for line in lines.iter().take(block.start) {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with("#!") {
            return false;
        }
    }
    true
}

impl RsRule for NoVagueComment {
    fn check(&self, _file: &syn::File, source: &str) -> Vec<(usize, String)> {
        let lines: Vec<&str> = source.lines().collect();
        let blocks = find_comment_blocks(&lines);
        let mut violations = vec![];

        for block in &blocks {
            if is_top_level_doc_block(block, &lines) {
                continue;
            }

            // Find the next non-blank line after the comment block.
            let mut next_non_blank = None;
            let mut has_blank_gap = false;
            for (i, line) in lines.iter().enumerate().skip(block.end) {
                if is_blank(line) {
                    has_blank_gap = true;
                } else {
                    next_non_blank = Some(i);
                    break;
                }
            }

            match next_non_blank {
                None => {
                    // Free-floating comment: no syntax item follows.
                    violations.push((
                        block.start + 1,
                        "free-floating comment block is not attached to any syntax item".into(),
                    ));
                }
                Some(next) => {
                    // Check if next line is a closing brace only.
                    let next_trimmed = lines[next].trim();
                    if next_trimmed == "}" || next_trimmed == "};" {
                        violations.push((
                            block.start + 1,
                            "free-floating comment block is not attached to any syntax item".into(),
                        ));
                    } else if has_blank_gap {
                        violations.push((
                            block.start + 1,
                            "blank line between comment block and its attached item".into(),
                        ));
                    }
                }
            }
        }

        violations
    }

    fn has_fixer(&self) -> bool {
        true
    }

    fn try_fix(&self, source: &str, _file: syn::File) -> Result<String, String> {
        let lines: Vec<&str> = source.lines().collect();
        let blocks = find_comment_blocks(&lines);
        let mut violations_found = false;

        // Collect line indices to remove (blank lines between comment blocks and items).
        let mut remove: Vec<bool> = vec![false; lines.len()];

        for block in &blocks {
            if is_top_level_doc_block(block, &lines) {
                continue;
            }

            let mut next_non_blank = None;
            let mut blank_start = None;
            for (i, line) in lines.iter().enumerate().skip(block.end) {
                if is_blank(line) {
                    if blank_start.is_none() {
                        blank_start = Some(i);
                    }
                } else {
                    next_non_blank = Some(i);
                    break;
                }
            }

            match next_non_blank {
                None => {
                    // Free-floating comment at end — can't auto-fix.
                    violations_found = true;
                }
                Some(next) => {
                    let next_trimmed = lines[next].trim();
                    if next_trimmed == "}" || next_trimmed == "};" {
                        // Free-floating before closing brace — can't auto-fix.
                        violations_found = true;
                    } else if let Some(bs) = blank_start {
                        // Remove blank lines between comment block and item.
                        for line_remove in remove.iter_mut().take(next).skip(bs) {
                            *line_remove = true;
                        }
                        violations_found = true;
                    }
                }
            }
        }

        if !violations_found {
            return Err("no violations to fix".into());
        }

        let result: Vec<&str> = lines
            .iter()
            .enumerate()
            .filter(|(i, _)| !remove[*i])
            .map(|(_, l)| *l)
            .collect();

        let mut out = result.join("\n");
        if source.ends_with('\n') {
            out.push('\n');
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        NoVagueComment.check(&file, src)
    }

    #[test]
    fn no_violations_attached_comment() {
        let src = "// This is a doc\nfn foo() {}\n";
        assert!(check(src).is_empty());
    }

    #[test]
    fn no_violations_inner_doc_at_top() {
        let src = "//! Module docs\n//! More docs\n\nuse std::io;\n";
        assert!(check(src).is_empty());
    }

    #[test]
    fn blank_line_between_comment_and_item() {
        let src = "// comment\n\nfn foo() {}\n";
        let v = check(src);
        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("blank line"));
    }

    #[test]
    fn free_floating_comment_at_eof() {
        let src = "fn foo() {}\n// orphan\n";
        let v = check(src);
        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("free-floating"));
    }

    #[test]
    fn free_floating_comment_before_closing_brace() {
        let src = "fn foo() {\n    // orphan\n}\n";
        let v = check(src);
        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("free-floating"));
    }

    #[test]
    fn comment_not_attached_to_item_is_violation() {
        let src = "use std::io;\n// detached\n\nfn foo() {}\n";
        let v = check(src);
        assert!(!v.is_empty());
    }

    #[test]
    fn fixer_removes_blank_lines() {
        let src = "// comment\n\nfn foo() {}\n";
        let file = syn::parse_file(src).unwrap();
        let fixed = NoVagueComment.try_fix(src, file).unwrap();
        assert_eq!(fixed, "// comment\nfn foo() {}\n");
    }

    #[test]
    fn fixer_preserves_attached_comments() {
        let src = "// attached\nfn foo() {}\n";
        let file = syn::parse_file(src).unwrap();
        let result = NoVagueComment.try_fix(src, file);
        assert!(result.is_err()); // nothing to fix
    }

    #[test]
    fn multiple_blank_lines_removed() {
        let src = "// comment\n\n\n\nfn foo() {}\n";
        let file = syn::parse_file(src).unwrap();
        let fixed = NoVagueComment.try_fix(src, file).unwrap();
        assert_eq!(fixed, "// comment\nfn foo() {}\n");
    }
}
