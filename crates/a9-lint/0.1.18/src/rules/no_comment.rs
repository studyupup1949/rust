use crate::LintError;

const RULE_NAME: &str = "no-comment";
const MSG: &str = "plain comment will be removed by parse→unparse; \
                    use /// doc-comment or refactor for clarity";

pub fn check_source(source: &str) -> Vec<LintError> {
    let mut errs = Vec::new();
    let mut in_block_comment = false;

    for (i, line) in source.lines().enumerate() {
        let stripped = strip_strings(line);
        let trimmed = stripped.trim();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }

            continue;
        }

        if let Some(pos) = trimmed.find("/*") {
            let after = &trimmed[pos + 2..];
            let is_doc = after.starts_with('*') && !after.starts_with("*/");

            if !is_doc {
                if !trimmed.contains("*/") {
                    in_block_comment = true;
                }

                errs.push(LintError::RuleError {
                    rule: RULE_NAME,
                    line: i + 1,
                    message: MSG.into(),
                });

                continue;
            }
        }

        let Some(pos) = trimmed.find("//") else {
            continue;
        };

        let after = &trimmed[pos + 2..];

        if after.starts_with('/') || after.starts_with('!') {
            continue;
        }

        errs.push(LintError::RuleError {
            rule: RULE_NAME,
            line: i + 1,
            message: MSG.into(),
        });
    }

    errs
}

fn strip_strings(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut chars = line.chars();

    while let Some(c) = chars.next() {
        if c != '"' {
            result.push(c);

            continue;
        }

        result.push(' ');

        while let Some(c2) = chars.next() {
            if c2 == '\\' {
                chars.next();
            } else if c2 == '"' {
                break;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_violations_on_clean_code() {
        assert!(check_source("fn main() {}\n").is_empty());
    }

    #[test]
    fn doc_comments_are_allowed() {
        assert!(check_source("/// This is a doc comment\nfn foo() {}\n").is_empty());
    }

    #[test]
    fn inner_doc_comments_are_allowed() {
        assert!(check_source("//! Module-level doc\nfn foo() {}\n").is_empty());
    }

    #[test]
    fn plain_line_comment_detected() {
        let v = check_source("// TODO: fix this\nfn main() {}\n");

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn plain_block_comment_detected() {
        let v = check_source("/* block */\nfn main() {}\n");

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn doc_block_comment_allowed() {
        assert!(check_source("/** doc block */\nfn main() {}\n").is_empty());
    }

    #[test]
    fn multiline_block_comment_single_violation() {
        let v = check_source("/* start\n   middle\n   end */\nfn main() {}\n");

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn inline_comment_after_code() {
        let v = check_source("let x = 1; // inline\n");

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn multiple_comments_multiple_violations() {
        let v = check_source("// first\n// second\nfn main() {}\n");

        assert_eq!(v.len(), 2);
    }
}
